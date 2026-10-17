use crate::commanders::_cmd::{get_media_seconds, h264_encoder, probe_streams};
use crate::helpers::str_helper::gen_input_file_content_for_ffmpeg;
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

/// Removes black bars from the video inputs before the merge.
/// Returns `None` and does nothing when no input has black bars.
/// Otherwise a cropped file is written to `tmp_dir/cropper` with the same file name, so the
/// chapter titles (taken from the file names) stay the same. Inputs without bars are used as
/// they are. Returns the new input paths and the concat list for them.
pub fn crop_black_bars(files: &[String], tmp_dir: &Path) -> Option<(Vec<String>, String)> {
  let crops: Vec<Option<String>> = files.iter().map(|file| detect_black_bars(file)).collect();
  if crops.iter().all(Option::is_none) {
    return None;
  }

  let dir = tmp_dir.join("cropper");
  fs::create_dir_all(&dir).expect("Failed to create the cropper folder");

  let inputs: Vec<String> = files
    .iter()
    .zip(crops)
    .map(|(file, crop)| {
      let Some(crop) = crop else {
        return file.clone();
      };
      let file_name = Path::new(file).file_name().unwrap().to_string_lossy();
      let output = dir.join(file_name.as_ref()).to_string_lossy().to_string();
      println!("✂️  Removing black bars from {file}, calling: crop={crop}");

      let res = Command::new("ffmpeg")
        .args(encode_args(file, &crop, &output))
        .output()
        .unwrap();
      if !res.status.success() {
        panic!(
          "❌ Cropping {} failed (exit code: {:?}):{}",
          file,
          res.status.code(),
          String::from_utf8_lossy(&res.stderr)
        );
      }
      println!("🔪 Cropped: {}", BrightBlue.paint(&output));
      output
    })
    .collect();

  let content = gen_input_file_content_for_ffmpeg(&inputs);
  Some((inputs, content))
}

/// Returns the crop value (`w:h:x:y`) for a file that has black bars, or `None`
/// when it has no video, has no bars, or the detection fails.
fn detect_black_bars(file: &str) -> Option<String> {
  let streams = probe_streams(file).ok()?;
  if !streams.iter().any(|stream| stream.kind == "video") {
    return None;
  }
  let (width, height) = frame_size(file)?;

  let filter = sampling_filter(get_media_seconds(file).ok()?);
  // `round=2` keeps the exact bar size. The default (16) can cut into the picture.
  // `skip=0` checks every sampled frame. The default (2) skips the first ones, so short clips
  // could give no crop value at all.
  let output = Command::new("ffmpeg")
    .args([
      "-v", "info", "-i", file, "-vf", &filter, "-an", "-f", "null", "-",
    ])
    .output()
    .ok()?;
  let crop = most_common(&parse_crop_values(&String::from_utf8_lossy(&output.stderr)))?;

  if crop == format!("{width}:{height}:0:0") {
    None
  } else {
    Some(crop)
  }
}

/// Returns the width and height of the first video stream.
fn frame_size(file: &str) -> Option<(u32, u32)> {
  let output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=width,height",
      "-of",
      "csv=p=0",
      file,
    ])
    .output()
    .ok()?;
  let text = String::from_utf8_lossy(&output.stdout);
  let (width, height) = text.trim().split_once(',')?;
  Some((width.parse().ok()?, height.parse().ok()?))
}

/// Number of frames to sample across the whole clip. Sampling the full length means a black
/// opening (or any black section) cannot hide the bars.
const SAMPLES_PER_CLIP: f64 = 60.0;

/// Returns the `fps` plus `cropdetect` filter that samples about `SAMPLES_PER_CLIP` frames
/// spread evenly over `duration_seconds`. Clips shorter than that get one frame per second.
fn sampling_filter(duration_seconds: f64) -> String {
  let fps = (SAMPLES_PER_CLIP / duration_seconds.max(1.0)).min(1.0);
  format!("fps={fps},cropdetect=round=2:skip=0")
}

/// True for a `w:h:x:y` value with a positive width and height. Black frames can make
/// `cropdetect` report negative sizes (e.g. `-318:-238:320:240`), which FFmpeg cannot crop with.
fn is_valid_crop(value: &str) -> bool {
  let parts: Vec<i64> = value.split(':').filter_map(|p| p.parse().ok()).collect();
  parts.len() == 4 && parts[0] > 0 && parts[1] > 0 && parts[2] >= 0 && parts[3] >= 0
}

/// Returns every `w:h:x:y` value from FFmpeg's `cropdetect` log lines, in order.
fn parse_crop_values(log: &str) -> Vec<String> {
  log
    .split("crop=")
    .skip(1)
    .filter_map(|rest| rest.split_whitespace().next())
    .filter(|value| is_valid_crop(value))
    .map(String::from)
    .collect()
}

/// Returns the value that appears most often. Ties go to the smaller value (BTreeMap order).
fn most_common(values: &[String]) -> Option<String> {
  let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
  for value in values {
    *counts.entry(value.as_str()).or_default() += 1;
  }
  counts
    .into_iter()
    .max_by_key(|(_, count)| *count)
    .map(|(value, _)| value.to_string())
}

/// The FFmpeg arguments that crop `input` to `crop` and write `output`.
/// Video, audio and subtitles are mapped. Data streams such as `tmcd` are dropped.
/// Chapters and metadata are copied from the input by FFmpeg's defaults.
fn encode_args(input: &str, crop: &str, output: &str) -> Vec<String> {
  let mut args: Vec<String> = [
    "-y", "-v", "error", "-i", input, "-map", "0:v", "-map", "0:a?", "-map", "0:s?",
  ]
  .map(String::from)
  .to_vec();
  // Subtitles are copied unchanged, so they are kept as they are.
  args.extend(["-c:s", "copy", "-vf"].map(String::from));
  args.push(format!("crop={crop}"));
  if let Some(encoder) = video_encoder_for(output) {
    args.extend(["-c:v".to_string(), encoder.to_string()]);
  }
  args.push(output.to_string());
  args
}

/// Returns the H.264 encoder for containers that can hold H.264. Other containers (e.g. WebM)
/// keep FFmpeg's default video codec, because H.264 cannot be stored in them.
fn video_encoder_for(output: &str) -> Option<&'static str> {
  let extension = Path::new(output)
    .extension()?
    .to_str()?
    .to_ascii_lowercase();
  matches!(extension.as_str(), "mp4" | "m4v" | "mov" | "mkv").then(h264_encoder)
}

#[cfg(test)]
mod test_parse_crop_values {
  use super::{is_valid_crop, parse_crop_values, sampling_filter};

  #[test]
  fn ignores_negative_sizes_from_black_frames() {
    let log = "crop=-318:-238:320:240 pts:0\ncrop=320:180:0:30 pts:1";
    assert_eq!(parse_crop_values(log), vec!["320:180:0:30"]);
  }

  #[test]
  fn valid_crop_needs_positive_width_and_height() {
    assert!(is_valid_crop("320:180:0:30"));
    assert!(!is_valid_crop("-318:-238:320:240"));
    assert!(!is_valid_crop("0:180:0:0"));
  }

  #[test]
  fn samples_long_clips_about_sixty_times() {
    assert_eq!(sampling_filter(600.0), "fps=0.1,cropdetect=round=2:skip=0");
  }

  #[test]
  fn samples_short_clips_once_per_second() {
    assert_eq!(sampling_filter(10.0), "fps=1,cropdetect=round=2:skip=0");
  }

  #[test]
  fn reads_every_crop_line() {
    let log = "[Parsed_cropdetect_1 @ 0x1] x1:0 x2:1919 y1:0 y2:695 w:1920 h:696 x:0 y:0 pts:0 t:0.0 crop=1920:696:0:0\n\
               [Parsed_cropdetect_1 @ 0x1] crop=1920:688:0:16 pts:1 t:1.0";
    assert_eq!(
      parse_crop_values(log),
      vec!["1920:696:0:0", "1920:688:0:16"]
    );
  }

  #[test]
  fn ignores_log_without_crop() {
    assert!(parse_crop_values("frame=  10 fps=0.0 size=N/A").is_empty());
  }
}

#[cfg(test)]
mod test_most_common {
  use super::most_common;

  #[test]
  fn picks_the_most_frequent_value() {
    let values: Vec<String> = ["1920:688:0:16", "1920:696:0:0", "1920:688:0:16"]
      .map(String::from)
      .to_vec();
    assert_eq!(most_common(&values), Some("1920:688:0:16".to_string()));
  }

  #[test]
  fn returns_none_for_no_values() {
    assert_eq!(most_common(&[]), None);
  }
}

#[cfg(test)]
mod test_encode_args {
  use super::{encode_args, video_encoder_for};

  #[test]
  fn keeps_extension_without_h264_for_webm() {
    assert_eq!(video_encoder_for("clip.webm"), None);
  }

  #[test]
  fn crops_and_keeps_video_audio_and_subtitles() {
    let args = encode_args("in.webm", "1920:688:0:16", "out.webm");
    assert!(args
      .windows(2)
      .any(|pair| pair == ["-vf", "crop=1920:688:0:16"]));
    assert!(args.windows(2).any(|pair| pair == ["-map", "0:v"]));
    assert!(args.windows(2).any(|pair| pair == ["-map", "0:s?"]));
    assert!(args.windows(2).any(|pair| pair == ["-c:s", "copy"]));
    assert!(!args.contains(&"-c:v".to_string()));
  }
}
