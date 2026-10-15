use crate::cli::Cli;
use crate::helpers::progress_helper::ProgressBar;
use clap::ArgMatches;
use lazy_static::lazy_static;
use path_slash::PathBufExt;
use std::{
  io::{BufRead, BufReader, Error, Read},
  path::PathBuf,
  process::{Child, Command, Output, Stdio},
};
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

lazy_static! {
  static ref MATCHES: ArgMatches = Cli::init().get_matches();
  static ref VERBOSE: bool = MATCHES.get_flag("verbose");
}

/// FFmpeg flags that make it write progress lines (`out_time_us=...`) to stdout
/// and keep stderr short. Used when not in verbose mode.
const PROGRESS_ARGS: [&str; 5] = ["-progress", "pipe:1", "-nostats", "-loglevel", "error"];

/// Merges the files and shows a 0–100% progress bar.
/// `total_seconds` is the combined length of all input files.
/// `extra_args` are added after `-c copy`, so they can override a codec
/// (for example `-c:a aac`).
pub fn merge(
  input: String,
  output: &str,
  chapters: &str,
  total_seconds: f64,
  extra_args: &[&str],
) -> Output {
  let mut args: Vec<&str> = vec![
    "-y",
    "-f",
    "concat",
    "-safe",
    "0",
    "-i",
    input.as_str(),
    "-i",
    chapters,
    "-map",
    "0",
    "-map_metadata",
    "1",
    "-c",
    "copy",
  ];
  args.extend(extra_args);
  if !*VERBOSE {
    args.extend(PROGRESS_ARGS);
  }
  args.push(output);

  println!(
    "🚀 Run Merger, calling: {}",
    BrightBlue.paint(describe("ffmpeg", &args))
  );

  if *VERBOSE {
    return spawn_command("ffmpeg", &args, false)
      .unwrap()
      .wait_with_output()
      .unwrap();
  }

  run_with_progress(&args, "Merging videos", total_seconds)
}

/// Runs FFmpeg (which must have the `PROGRESS_ARGS` set) and shows a 0–100% bar.
/// `total_seconds` is the length of the input. If it is 0, the bar only jumps to 100% at the end.
fn run_with_progress(args: &[&str], label: &str, total_seconds: f64) -> Output {
  let mut child = Command::new("ffmpeg")
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  // Drain stderr on its own thread, so FFmpeg never blocks on a full stderr pipe.
  let stderr_reader = child.stderr.take().map(|mut stderr| {
    std::thread::spawn(move || {
      let mut buf = Vec::new();
      let _ = stderr.read_to_end(&mut buf);
      buf
    })
  });
  let mut bar = ProgressBar::start(label);
  if let Some(stdout) = child.stdout.take() {
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
      if let Some(micros) = parse_out_time_us(&line) {
        if total_seconds > 0.0 {
          bar.set(micros as f64 / 1_000_000.0 / total_seconds);
        }
      }
    }
  }
  let status = child.wait().unwrap();
  let stderr = stderr_reader
    .and_then(|handle| handle.join().ok())
    .unwrap_or_default();
  bar.finish(status.success());
  // stdout is consumed above for progress, so the returned Output has it empty.
  Output {
    status,
    stdout: Vec::new(),
    stderr,
  }
}

/// Reads the `out_time_us=` value from FFmpeg's `-progress` output, in microseconds.
/// Returns `None` for other lines and for `N/A` values.
fn parse_out_time_us(line: &str) -> Option<i64> {
  line
    .strip_prefix("out_time_us=")?
    .trim()
    .parse::<i64>()
    .ok()
}

#[cfg(test)]
mod tests {
  use super::parse_out_time_us;

  #[test]
  fn parses_out_time_us_line() {
    assert_eq!(parse_out_time_us("out_time_us=1500000"), Some(1_500_000));
  }

  #[test]
  fn ignores_other_progress_lines() {
    assert_eq!(parse_out_time_us("out_time=00:00:01.500000"), None);
    assert_eq!(parse_out_time_us("out_time_us=N/A"), None);
    assert_eq!(parse_out_time_us("progress=continue"), None);
  }
}

pub fn run_ffmpeg_info_command(file_to_merge: &PathBuf) -> Result<Output, Error> {
  Command::new("ffmpeg")
    .args(["-i", &file_to_merge.to_slash().unwrap()])
    .output()
}

pub fn adjust_fps_by_ffmpeg(
  file_to_merge: PathBuf,
  fps_goal: &f32,
  new_file_location: PathBuf,
) -> PathBuf {
  let input = file_to_merge.to_str().unwrap();
  let fps = fps_goal.to_string();
  let output = new_file_location.to_str().unwrap();
  let mut args = vec!["-i", input, "-r", fps.as_str()];

  // Print the command without the progress flags, so the log line stays short.
  let shown: Vec<&str> = args.iter().copied().chain([output]).collect();
  println!(
    "🚀 Start FPS Changer, calling: {}",
    BrightBlue.paint(describe("ffmpeg", &shown))
  );

  if *VERBOSE {
    args.push(output);
    let res = spawn_command("ffmpeg", &args, false)
      .unwrap()
      .wait_with_output()
      .unwrap();
    println!("{res:?}");
  } else {
    args.extend(PROGRESS_ARGS);
    args.push(output);
    let total_seconds = get_media_seconds(input).unwrap_or(0.0);
    let res = run_with_progress(&args, "Changing FPS", total_seconds);
    if !res.status.success() {
      panic!(
        "❌ FPS change failed (exit code: {:?}):{}",
        res.status.code(),
        String::from_utf8_lossy(&res.stderr)
      );
    }
  }
  new_file_location
}

/// Returns the codec names of all audio streams (e.g. `aac`, `pcm_alaw`),
/// or `None` if there is no audio stream or ffprobe fails.
pub fn get_audio_codecs(media_path: &str) -> Option<Vec<String>> {
  let output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-select_streams",
      "a",
      "-show_entries",
      "stream=codec_name",
      "-of",
      "default=noprint_wrappers=1:nokey=1",
      media_path,
    ])
    .output()
    .ok()?;
  let codecs: Vec<String> = String::from_utf8(output.stdout)
    .ok()?
    .lines()
    .map(str::trim)
    .filter(|codec| !codec.is_empty())
    .map(str::to_string)
    .collect();
  if codecs.is_empty() {
    None
  } else {
    Some(codecs)
  }
}

pub fn get_media_seconds(media_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
  let args = [
    "-v",
    "error",
    "-show_entries",
    "format=duration",
    "-of",
    "default=noprint_wrappers=1:nokey=1",
    media_path,
  ];

  if *VERBOSE {
    println!(
      "📖 Getting media seconds, calling: {}",
      BrightBlue.paint(describe("ffprobe", &args))
    );
  }
  let output = spawn_command("ffprobe", &args, false)?.wait_with_output()?;
  let output = String::from_utf8(output.stdout)?;
  Ok(output.trim().parse::<f64>()?)
}

/// Starts a program with each argument passed separately (no shell).
/// This keeps file names with quotes, spaces or `$` from being split or
/// interpreted by a shell.
fn spawn_command(program: &str, args: &[&str], silent: bool) -> Result<Child, std::io::Error> {
  let mut command = Command::new(program);
  command.args(args);
  if silent {
    command.stdout(Stdio::null()).stderr(Stdio::null());
  } else {
    command.stdout(Stdio::piped());
  }
  command.spawn()
}

/// Builds a printable version of the command for log output only.
/// It is never executed.
fn describe(program: &str, args: &[&str]) -> String {
  std::iter::once(program)
    .chain(args.iter().copied())
    .map(|arg| format!("'{arg}'"))
    .collect::<Vec<_>>()
    .join(" ")
}
