use crate::cli::Cli;
use clap::ArgMatches;
use lazy_static::lazy_static;
use path_slash::PathBufExt;
use std::{
  io::Error,
  path::PathBuf,
  process::{Child, Command, Output, Stdio},
};
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

lazy_static! {
  static ref MATCHES: ArgMatches = Cli::init().get_matches();
  static ref VERBOSE: bool = MATCHES.get_flag("verbose");
}

pub fn merge(input: String, output: &str, chapters: &str) -> Output {
  let args = [
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
    output,
  ];

  println!(
    "🚀 Run Merger, calling: {}",
    BrightBlue.paint(describe("ffmpeg", &args))
  );
  spawn_command("ffmpeg", &args, !*VERBOSE)
    .unwrap()
    .wait_with_output()
    .unwrap()
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
  let args = ["-i", input, "-r", fps.as_str(), output];

  println!(
    "🚀 Start FPS Changer, calling: {}",
    BrightBlue.paint(describe("ffmpeg", &args))
  );

  let res = spawn_command("ffmpeg", &args, !*VERBOSE)
    .unwrap()
    .wait_with_output();
  if *VERBOSE {
    println!("{res:?}");
  }
  new_file_location
}

pub fn get_media_seconds(media_path: &str) -> Result<f64, Box<Error>> {
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
  let res = spawn_command("ffprobe", &args, false);

  let output = res.unwrap().wait_with_output().unwrap();
  let output = String::from_utf8(output.stdout).unwrap();
  let output = output.trim().parse::<f64>().unwrap();
  Ok(output)
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
