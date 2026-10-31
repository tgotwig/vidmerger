use crate::cli::Cli;
use crate::helpers::io_helper::verbose;
use crate::helpers::progress_helper::ProgressBar;
use clap::ArgMatches;
use lazy_static::lazy_static;
use path_slash::PathBufExt;
use std::{
  collections::BTreeMap,
  io::{BufRead, BufReader, Error, Read},
  path::PathBuf,
  process::{Child, Command, Output, Stdio},
  sync::OnceLock,
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

/// Merges the files with stream copy and shows a 0–100% progress bar.
/// `total_seconds` is the combined length of all input files.
pub fn merge(input: String, output: &str, chapters: &str, total_seconds: f64) -> Output {
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
  if !verbose() {
    args.extend(PROGRESS_ARGS);
  }
  args.push(output);

  println!(
    "🚀 Run Merger, calling: {}",
    BrightBlue.paint(describe("ffmpeg", &args))
  );

  if verbose() {
    return spawn_command("ffmpeg", &args, false)
      .unwrap()
      .wait_with_output()
      .unwrap();
  }

  run_with_progress(&args, "Merging videos", total_seconds)
}

/// GPU H.264 encoders for Linux and Windows, in order of preference.
const GPU_H264_ENCODERS: [&str; 3] = ["h264_nvenc", "h264_qsv", "h264_amf"];

/// Returns the first candidate for which `works` is true, or `libx264` (CPU) if none is.
/// Stops at the first success, so later candidates are not tested.
fn pick_h264_encoder(candidates: &[&'static str], works: impl Fn(&str) -> bool) -> &'static str {
  candidates
    .iter()
    .copied()
    .find(|encoder| works(encoder))
    .unwrap_or("libx264")
}

/// Returns true if FFmpeg can encode one test frame with `encoder`.
/// FFmpeg can list a GPU encoder even when no GPU is present, so a real test encode is used.
fn encoder_works(encoder: &str) -> bool {
  Command::new("ffmpeg")
    .args([
      "-v",
      "error",
      "-f",
      "lavfi",
      "-i",
      "nullsrc=s=256x256",
      "-frames:v",
      "1",
      "-c:v",
      encoder,
      "-f",
      "null",
      "-",
    ])
    .output()
    .map(|output| output.status.success())
    .unwrap_or(false)
}

/// Returns the H.264 encoder to use. The result is found once per run.
/// macOS tries VideoToolbox. Linux and Windows try the GPU encoders (NVIDIA, Intel or AMD).
/// Each candidate is tested first. If none works, `libx264` (CPU) is used.
pub fn h264_encoder() -> &'static str {
  static ENCODER: OnceLock<&'static str> = OnceLock::new();
  ENCODER.get_or_init(|| {
    let candidates: &[&'static str] = if cfg!(target_os = "macos") {
      &["h264_videotoolbox"]
    } else {
      &GPU_H264_ENCODERS
    };
    pick_h264_encoder(candidates, encoder_works)
  })
}

/// Converts one input to an MP4 file that is safe to merge.
/// Only video and audio streams are mapped, so data streams (e.g. `tmcd`) are dropped.
/// Video is re-encoded to H.264. Audio is encoded to AAC with the shared `profile`.
/// If `silent_audio` is true, the input has no audio, so silence is generated and
/// `-shortest` ends it with the video. `-write_tmcd 0` stops the MP4/MOV muxer from
/// adding a `tmcd` track again.
pub fn convert_for_merge(
  input: &str,
  output: &str,
  profile: Option<AudioProfile>,
  silent_audio: bool,
  hevc: bool,
) -> Output {
  // The merge chooses one video codec for every input (see `plan_merge`), so the converter does
  // not pick its own. HEVC uses the hardware HEVC encoder, H.264 uses the H.264 encoder.
  let encoder = if hevc {
    "hevc_videotoolbox"
  } else {
    h264_encoder()
  };
  convert_with_cpu_fallback(
    encoder,
    |result: &Output| result.status.success(),
    |encoder| run_convert(input, output, profile, silent_audio, encoder),
  )
}

/// True when the input's first video stream is HEVC (H.265).
pub fn is_hevc(input: &str) -> bool {
  Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=codec_name",
      "-of",
      "csv=p=0",
      input,
    ])
    .output()
    .map(|output| is_hevc_codec_output(&String::from_utf8_lossy(&output.stdout)))
    .unwrap_or(false)
}

/// Reads ffprobe's `codec_name` output. ffprobe can print a trailing comma (e.g. "hevc,"), so
/// only the first field is the codec.
fn is_hevc_codec_output(text: &str) -> bool {
  text
    .trim()
    .split(',')
    .next()
    .is_some_and(|codec| codec == "hevc")
}

#[cfg(test)]
mod test_is_hevc_codec_output {
  use super::is_hevc_codec_output;

  #[test]
  fn accepts_plain_hevc() {
    assert!(is_hevc_codec_output("hevc\n"));
  }

  #[test]
  fn accepts_hevc_with_trailing_comma() {
    assert!(is_hevc_codec_output("hevc,\n"));
  }

  #[test]
  fn rejects_other_codecs() {
    assert!(!is_hevc_codec_output("h264,\n"));
    assert!(!is_hevc_codec_output(""));
  }
}

/// Runs `convert` with `encoder`. If that fails and `encoder` is not `libx264`, runs it again
/// with `libx264`. A GPU encoder can pass the test frame but still fail on a real input.
fn convert_with_cpu_fallback<T>(
  encoder: &str,
  succeeded: impl Fn(&T) -> bool,
  convert: impl Fn(&str) -> T,
) -> T {
  let result = convert(encoder);
  if succeeded(&result) || encoder == "libx264" {
    return result;
  }
  println!("⚠️  {encoder} failed, retrying with libx264");
  convert("libx264")
}

/// Inputs shorter than this keep the encoder's own bitrate. Forcing a bitrate on very short clips
/// is not needed to control the size of a long film, and it changed the length of short clips.
const MIN_SECONDS_FOR_SOURCE_BIT_RATE: f64 = 60.0;

/// The bitrate to pass to the encoder for `input`, or `None` to let the encoder choose. Only
/// long inputs get the source bitrate, which is what keeps a film's output size close to its source.
pub fn keep_source_bit_rate(input: &str) -> Option<u64> {
  let long_enough = get_media_seconds(input).is_ok_and(|s| s >= MIN_SECONDS_FOR_SOURCE_BIT_RATE);
  if long_enough {
    video_bit_rate(input)
  } else {
    None
  }
}

/// Returns the video stream's bitrate in bits per second, or `None` when the file does not
/// report one (e.g. some containers only report the format bitrate).
pub fn video_bit_rate(input: &str) -> Option<u64> {
  let output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=bit_rate",
      "-of",
      "csv=p=0",
      input,
    ])
    .output()
    .ok()?;
  parse_bit_rate(&String::from_utf8_lossy(&output.stdout))
}

/// Reads ffprobe's bitrate output. ffprobe can print a trailing comma (e.g. "12345,"), so only
/// the first field is the bitrate. A zero or missing value is `None`.
fn parse_bit_rate(text: &str) -> Option<u64> {
  text
    .trim()
    .split(',')
    .next()
    .and_then(|field| field.parse::<u64>().ok())
    .filter(|bit_rate| *bit_rate > 0)
}

#[cfg(test)]
mod test_parse_bit_rate {
  use super::parse_bit_rate;

  #[test]
  fn reads_a_plain_bitrate() {
    assert_eq!(parse_bit_rate("12345\n"), Some(12345));
  }

  #[test]
  fn reads_a_bitrate_with_a_trailing_comma() {
    assert_eq!(parse_bit_rate("12345,\n"), Some(12345));
  }

  #[test]
  fn ignores_missing_or_zero_bitrate() {
    assert_eq!(parse_bit_rate(""), None);
    assert_eq!(parse_bit_rate("N/A\n"), None);
    assert_eq!(parse_bit_rate("0\n"), None);
  }
}

/// Runs one conversion with the given H.264 `encoder`. See `convert_for_merge`.
fn run_convert(
  input: &str,
  output: &str,
  profile: Option<AudioProfile>,
  silent_audio: bool,
  encoder: &str,
) -> Output {
  let mut args: Vec<String> = ["-y", "-i", input].map(String::from).to_vec();
  let audio_map = if silent_audio {
    let profile = profile.expect("silent audio needs an audio profile");
    args.extend(["-f", "lavfi", "-i"].map(String::from));
    args.push(format!("anullsrc=r={}", profile.sample_rate));
    "1:a"
  } else {
    "0:a?"
  };
  // Only the first video stream is mapped. Files can carry extra video streams (e.g. a PNG
  // cover image) that MP4 cannot hold.
  args.extend(["-map", "0:v:0?", "-map", audio_map, "-c:v"].map(String::from));
  args.push(encoder.to_string());
  // Apple players only open HEVC in MP4 when it is tagged `hvc1`.
  if encoder.starts_with("hevc") {
    args.extend(["-tag:v", "hvc1"].map(String::from));
  }
  // Keep the source's video bitrate on long inputs (see `keep_source_bit_rate`).
  if let Some(bit_rate) = keep_source_bit_rate(input) {
    args.extend(["-b:v".to_string(), bit_rate.to_string()]);
  }
  args.extend(["-c:a", "aac"].map(String::from));
  if let Some(profile) = profile {
    args.extend(["-ar".to_string(), profile.sample_rate.to_string()]);
    args.extend(["-ac".to_string(), profile.channels.to_string()]);
  }
  if silent_audio {
    args.push("-shortest".to_string());
  }
  args.extend(["-write_tmcd", "0"].map(String::from));
  args.push(output.to_string());

  let shown: Vec<&str> = args.iter().map(String::as_str).collect();
  println!(
    "🔁 Converting for merge, calling: {}",
    BrightBlue.paint(describe("ffmpeg", &shown))
  );

  // `output()` captures stderr too, so a failure message can be shown.
  Command::new("ffmpeg").args(&args).output().unwrap()
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

/// Runs `ffmpeg -i <file>` and returns its output. FFmpeg prints stream info to stderr.
/// Returns `Err` if FFmpeg cannot be started. Panics if the path is not valid UTF-8.
pub fn run_ffmpeg_info_command(file_to_merge: &PathBuf) -> Result<Output, Error> {
  Command::new("ffmpeg")
    .args(["-i", &file_to_merge.to_slash().unwrap()])
    .output()
}

/// Changes the frame rate of `file_to_merge` to `fps_goal` and writes the result to
/// `new_file_location`. Returns the new file's path. In normal (non-verbose) mode it panics if
/// FFmpeg fails. In verbose mode it prints FFmpeg's result and returns the path even on failure.
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

  if verbose() {
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

/// One stream reported by ffprobe. `sample_rate` and `channels` are only set for audio.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MediaStream {
  pub kind: String,
  pub codec: String,
  pub sample_rate: Option<u32>,
  pub channels: Option<u32>,
}

/// Audio settings that all merged inputs share (sample rate in Hz, channel count).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AudioProfile {
  pub sample_rate: u32,
  pub channels: u32,
}

/// Lists the streams of a media file with one ffprobe call.
/// Returns `Err` if ffprobe fails or an audio stream lacks its sample rate or channels.
pub fn probe_streams(media_path: &str) -> Result<Vec<MediaStream>, String> {
  let output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-show_entries",
      "stream=codec_type,codec_name,sample_rate,channels",
      "-of",
      "flat",
      media_path,
    ])
    .output()
    .map_err(|err| err.to_string())?;
  if !output.status.success() {
    return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
  }
  parse_flat_streams(&String::from_utf8_lossy(&output.stdout))
}

/// Parses ffprobe `-of flat` output, e.g. `streams.stream.1.codec_type="audio"`.
/// Fields are grouped by stream index. ffprobe leaves out fields a stream does not have.
fn parse_flat_streams(text: &str) -> Result<Vec<MediaStream>, String> {
  let mut streams: BTreeMap<usize, BTreeMap<&str, &str>> = BTreeMap::new();
  for line in text.lines() {
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let parts: Vec<&str> = key.split('.').collect();
    let ["streams", "stream", index, field] = parts.as_slice() else {
      continue;
    };
    let Ok(index) = index.parse::<usize>() else {
      continue;
    };
    streams
      .entry(index)
      .or_default()
      .insert(field, value.trim_matches('"'));
  }
  streams.values().map(stream_from_fields).collect()
}

/// Builds one `MediaStream` from its ffprobe fields. Audio needs a sample rate and channels.
fn stream_from_fields(fields: &BTreeMap<&str, &str>) -> Result<MediaStream, String> {
  let kind = fields.get("codec_type").copied().unwrap_or("unknown");
  let codec = fields.get("codec_name").copied().unwrap_or("");
  let (sample_rate, channels) = if kind == "audio" {
    let sample_rate = fields
      .get("sample_rate")
      .and_then(|v| v.parse::<u32>().ok());
    let channels = fields.get("channels").and_then(|v| v.parse::<u32>().ok());
    match (sample_rate, channels) {
      (Some(rate), Some(count)) => (Some(rate), Some(count)),
      _ => {
        return Err(format!(
          "audio stream without sample rate or channels: {fields:?}"
        ))
      }
    }
  } else {
    (None, None)
  };
  Ok(MediaStream {
    kind: kind.to_string(),
    codec: codec.to_string(),
    sample_rate,
    channels,
  })
}

/// Returns the length of a media file in seconds, read with ffprobe.
/// Returns `Err` if ffprobe cannot be started or waited on, if its output is not valid UTF-8,
/// or if its output is not a number.
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

  if verbose() {
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

#[cfg(test)]
mod test_parse_flat_streams {
  use super::{parse_flat_streams, MediaStream};

  #[test]
  fn parses_audio_and_video_streams_in_index_order() {
    let text = "streams.stream.0.codec_name=\"h264\"\n\
                streams.stream.0.codec_type=\"video\"\n\
                streams.stream.1.codec_name=\"aac\"\n\
                streams.stream.1.codec_type=\"audio\"\n\
                streams.stream.1.sample_rate=\"44100\"\n\
                streams.stream.1.channels=1\n";
    assert_eq!(
      parse_flat_streams(text),
      Ok(vec![
        MediaStream {
          kind: "video".into(),
          codec: "h264".into(),
          sample_rate: None,
          channels: None,
        },
        MediaStream {
          kind: "audio".into(),
          codec: "aac".into(),
          sample_rate: Some(44100),
          channels: Some(1),
        },
      ])
    );
  }

  #[test]
  fn parses_data_stream_without_audio_fields() {
    let text = "streams.stream.0.codec_name=\"tmcd\"\nstreams.stream.0.codec_type=\"data\"\n";
    let streams = parse_flat_streams(text).unwrap();
    assert_eq!(streams[0].codec, "tmcd");
    assert_eq!((streams[0].sample_rate, streams[0].channels), (None, None));
  }

  #[test]
  fn rejects_audio_stream_without_numbers() {
    let text = "streams.stream.0.codec_name=\"aac\"\nstreams.stream.0.codec_type=\"audio\"\n";
    assert!(parse_flat_streams(text).is_err());
  }
}

#[cfg(test)]
mod test_h264_encoder {
  use super::{convert_with_cpu_fallback, pick_h264_encoder, GPU_H264_ENCODERS};
  use std::cell::RefCell;

  #[test]
  fn fallback_keeps_result_when_encoder_succeeds() {
    let calls = RefCell::new(Vec::new());
    let ok = convert_with_cpu_fallback(
      "h264_nvenc",
      |ok: &bool| *ok,
      |encoder| {
        calls.borrow_mut().push(encoder.to_string());
        true
      },
    );
    assert!(ok);
    assert_eq!(*calls.borrow(), vec!["h264_nvenc"]);
  }

  #[test]
  fn fallback_retries_gpu_failure_with_libx264() {
    let calls = RefCell::new(Vec::new());
    let ok = convert_with_cpu_fallback(
      "h264_nvenc",
      |ok: &bool| *ok,
      |encoder| {
        calls.borrow_mut().push(encoder.to_string());
        encoder == "libx264"
      },
    );
    assert!(ok);
    assert_eq!(*calls.borrow(), vec!["h264_nvenc", "libx264"]);
  }

  #[test]
  fn fallback_does_not_retry_when_libx264_fails() {
    let calls = RefCell::new(Vec::new());
    let ok = convert_with_cpu_fallback(
      "libx264",
      |ok: &bool| *ok,
      |encoder| {
        calls.borrow_mut().push(encoder.to_string());
        false
      },
    );
    assert!(!ok);
    assert_eq!(*calls.borrow(), vec!["libx264"]);
  }

  #[test]
  fn picks_nvenc_first_and_stops_testing() {
    let tested = std::cell::RefCell::new(Vec::new());
    let picked = pick_h264_encoder(&GPU_H264_ENCODERS, |encoder| {
      tested.borrow_mut().push(encoder.to_string());
      true
    });
    assert_eq!(picked, "h264_nvenc");
    assert_eq!(*tested.borrow(), vec!["h264_nvenc"]);
  }

  #[test]
  fn picks_qsv_when_only_qsv_works() {
    let picked = pick_h264_encoder(&GPU_H264_ENCODERS, |e| e == "h264_qsv");
    assert_eq!(picked, "h264_qsv");
  }

  #[test]
  fn falls_back_to_libx264_without_gpu_encoder() {
    assert_eq!(pick_h264_encoder(&GPU_H264_ENCODERS, |_| false), "libx264");
  }
}

/// Integration test: this runs real ffmpeg to test each H.264 encoder.
#[cfg(test)]
mod test_h264_encoder_integration {
  use super::h264_encoder;

  #[test]
  fn uses_videotoolbox_on_macos_and_a_known_encoder_elsewhere() {
    // The probe can fail on a supported system, so the CPU fallback is also valid.
    let encoder = h264_encoder();
    let allowed: &[&str] = if cfg!(target_os = "macos") {
      &["h264_videotoolbox", "libx264"]
    } else {
      &["h264_nvenc", "h264_qsv", "h264_amf", "libx264"]
    };
    assert!(allowed.contains(&encoder), "unexpected encoder: {encoder}");
  }

  #[test]
  fn ffmpeg_is_installed() {
    assert!(
      which::which("ffmpeg").is_ok(),
      "ffmpeg wasn't found on the PATH"
    );
  }
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

/// Integration tests: these call real ffmpeg and ffprobe on generated media files.
#[cfg(test)]
mod test_ffmpeg_integration {
  use super::AudioProfile;
  use super::{
    adjust_fps_by_ffmpeg, convert_for_merge, get_media_seconds, merge, probe_streams,
    run_ffmpeg_info_command,
  };
  use std::path::{Path, PathBuf};
  use std::process::Command;

  /// A temp folder that is removed when the test ends, even if an assert fails.
  struct TempDir(PathBuf);

  impl TempDir {
    fn new(name: &str) -> Self {
      let dir = std::env::temp_dir().join(format!("vidmerger-{name}-{}", std::process::id()));
      let _ = std::fs::remove_dir_all(&dir);
      std::fs::create_dir_all(&dir).unwrap();
      TempDir(dir)
    }

    fn path(&self, file: &str) -> PathBuf {
      self.0.join(file)
    }
  }

  impl Drop for TempDir {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.0);
    }
  }

  /// Creates a 1-second MP4 with a video stream, and an audio stream when `audio` is true.
  fn make_clip(path: &Path, audio: bool) {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
      "-v",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "testsrc=d=1:s=160x120:r=25",
    ]);
    if audio {
      cmd.args(["-f", "lavfi", "-i", "sine=f=440:d=1"]);
    }
    cmd.args(["-c:v", "libx264", "-pix_fmt", "yuv420p"]);
    if audio {
      cmd.args(["-c:a", "aac", "-shortest"]);
    }
    let status = cmd.arg(path).status().expect("ffmpeg must be installed");
    assert!(status.success(), "failed to create test clip");
  }

  #[test]
  fn run_ffmpeg_info_command_reports_stream_info() {
    let dir = TempDir::new("info");
    let clip = dir.path("clip.mp4");
    make_clip(&clip, false);

    let output = run_ffmpeg_info_command(&clip).unwrap();
    let info = String::from_utf8_lossy(&output.stderr);
    assert!(info.contains("Video:"), "no video stream in: {info}");
    assert!(info.contains("25 fps"), "wrong fps in: {info}");
  }

  #[test]
  fn probe_streams_lists_video_and_audio() {
    let dir = TempDir::new("probe");
    let clip = dir.path("clip.mp4");
    make_clip(&clip, true);

    let streams = probe_streams(clip.to_str().unwrap()).unwrap();
    let kinds: Vec<&str> = streams.iter().map(|s| s.kind.as_str()).collect();
    assert_eq!(kinds, ["video", "audio"]);
    let audio = &streams[1];
    assert_eq!(audio.codec, "aac");
    assert_eq!(audio.sample_rate, Some(44100));
    assert_eq!(audio.channels, Some(1));
  }

  #[test]
  fn probe_streams_fails_on_missing_file() {
    let dir = TempDir::new("probe-missing");
    let missing = dir.path("nope.mp4");
    assert!(probe_streams(missing.to_str().unwrap()).is_err());
  }

  #[test]
  fn get_media_seconds_returns_clip_length() {
    let dir = TempDir::new("seconds");
    let clip = dir.path("clip.mp4");
    make_clip(&clip, false);

    let seconds = get_media_seconds(clip.to_str().unwrap()).unwrap();
    assert!(
      (seconds - 1.0).abs() < 0.1,
      "expected about 1s, got {seconds}"
    );
  }

  #[test]
  fn convert_for_merge_writes_mp4_with_aac_audio() {
    let dir = TempDir::new("convert");
    let input = dir.path("input.mp4");
    let output = dir.path("converted.mp4");
    make_clip(&input, true);

    let profile = AudioProfile {
      sample_rate: 48_000,
      channels: 2,
    };
    let result = convert_for_merge(
      input.to_str().unwrap(),
      output.to_str().unwrap(),
      Some(profile),
      false,
      false,
    );
    assert!(
      result.status.success(),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );

    let streams = probe_streams(output.to_str().unwrap()).unwrap();
    let audio = streams.iter().find(|s| s.kind == "audio").unwrap();
    assert_eq!(audio.codec, "aac");
    assert_eq!(audio.sample_rate, Some(48_000));
    assert_eq!(audio.channels, Some(2));
  }

  /// Creates a 1-second 160x120 video at `fps` frames per second, without audio.
  fn make_clip_with_fps(path: &Path, fps: u32) {
    let status = Command::new("ffmpeg")
      .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
      .arg(format!("testsrc=d=1:s=160x120:r={fps}"))
      .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
      .arg(path)
      .status()
      .expect("ffmpeg must be installed");
    assert!(status.success(), "failed to create test clip");
  }

  #[test]
  fn merge_joins_two_clips_into_one_file() {
    let dir = TempDir::new("merge");
    let first = dir.path("1 - first.mp4");
    let second = dir.path("2 - second.mp4");
    make_clip(&first, false);
    make_clip(&second, false);

    // The concat list and the chapter file that `merge` reads.
    let list = dir.path("list.txt");
    std::fs::write(
      &list,
      format!(
        "file '{}'\nfile '{}'\n",
        first.to_str().unwrap(),
        second.to_str().unwrap()
      ),
    )
    .unwrap();
    let chapters = dir.path("chapters.txt");
    std::fs::write(&chapters, ";FFMETADATA1\n").unwrap();
    let output = dir.path("merged.mp4");

    let result = merge(
      list.to_str().unwrap().to_string(),
      output.to_str().unwrap(),
      chapters.to_str().unwrap(),
      2.0,
    );
    assert!(
      result.status.success(),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );

    let seconds = get_media_seconds(output.to_str().unwrap()).unwrap();
    assert!(
      (seconds - 2.0).abs() < 0.2,
      "expected about 2s, got {seconds}"
    );
  }

  #[test]
  fn adjust_fps_by_ffmpeg_changes_the_frame_rate() {
    let dir = TempDir::new("fps");
    let input = dir.path("30fps.mp4");
    let output = dir.path("25fps.mp4");
    make_clip_with_fps(&input, 30);

    let written = adjust_fps_by_ffmpeg(input.clone(), &25.0, output.clone());
    assert_eq!(written, output);

    let info = run_ffmpeg_info_command(&output).unwrap();
    let text = String::from_utf8_lossy(&info.stderr);
    assert!(text.contains("25 fps"), "expected 25 fps in: {text}");
  }

  #[test]
  fn convert_for_merge_adds_silence_when_input_has_no_audio() {
    let dir = TempDir::new("silent");
    let input = dir.path("video-only.mp4");
    let output = dir.path("with-silence.mp4");
    make_clip(&input, false);

    let profile = AudioProfile {
      sample_rate: 44_100,
      channels: 1,
    };
    let result = convert_for_merge(
      input.to_str().unwrap(),
      output.to_str().unwrap(),
      Some(profile),
      true,
      false,
    );
    assert!(
      result.status.success(),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );

    let streams = probe_streams(output.to_str().unwrap()).unwrap();
    assert!(
      streams.iter().any(|s| s.kind == "audio"),
      "no silent audio added"
    );
  }
}
