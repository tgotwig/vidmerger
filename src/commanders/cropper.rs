use crate::commanders::_cmd::{get_media_seconds, h264_encoder, probe_streams};
use crate::helpers::str_helper::gen_input_file_content_for_ffmpeg;
use std::{fs, path::Path, process::Command};
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
  let crop = uniform_crop(&parse_crop_values(&String::from_utf8_lossy(&output.stderr)))?;

  if is_full_frame(&crop, width, height) {
    None
  } else {
    Some(crop)
  }
}

/// True when `crop` (`w:h:x:y`) is the whole frame, allowing one pixel of rounding per side.
/// `cropdetect` rounds to even sizes, so a bar-free 321x181 frame can report 320:180:0:0.
fn is_full_frame(crop: &str, width: u32, height: u32) -> bool {
  let parts: Vec<i64> = crop.split(':').filter_map(|p| p.parse().ok()).collect();
  if parts.len() != 4 {
    return false;
  }
  let (w, h, x, y) = (parts[0], parts[1], parts[2], parts[3]);
  let (width, height) = (i64::from(width), i64::from(height));
  (width - w) <= 1 && (height - h) <= 1 && x <= 1 && y <= 1
}

/// Returns the width and height of the first video stream, as FFmpeg sees the frames.
/// A stream rotated by a quarter turn (90 or 270 degrees) is swapped, because `cropdetect`
/// works on the rotated frames.
fn frame_size(file: &str) -> Option<(u32, u32)> {
  let output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=width,height:stream_side_data=rotation",
      "-of",
      "csv=p=0",
      file,
    ])
    .output()
    .ok()?;
  parse_frame_size(&String::from_utf8_lossy(&output.stdout))
}

/// Reads ffprobe's csv output: `width,height[,rotation]`. Rotation is only printed for rotated
/// streams, which leaves a trailing comma (e.g. "320,180,-90"). Empty fields are skipped.
fn parse_frame_size(text: &str) -> Option<(u32, u32)> {
  let mut fields = text.trim().split(',').filter(|field| !field.is_empty());
  let width: u32 = fields.next()?.parse().ok()?;
  let height: u32 = fields.next()?.parse().ok()?;
  let quarter_turn = fields
    .next()
    .and_then(|angle| angle.parse::<f64>().ok())
    .is_some_and(|angle| (angle.abs() % 180.0 - 90.0).abs() < 1.0);
  Some(if quarter_turn {
    (height, width)
  } else {
    (width, height)
  })
}

/// Number of frames to sample across the whole clip. Sampling the full length means a black
/// opening (or any black section) cannot hide the bars.
const SAMPLES_PER_CLIP: f64 = 66.0;

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

/// Returns the crop value only when every sampled frame agrees on it. A clip that mixes
/// letterboxed and full-frame scenes (e.g. a cutscene) gives different values, so it returns
/// `None` and is left uncropped. This keeps the picture from being cut.
fn uniform_crop(values: &[String]) -> Option<String> {
  let first = values.first()?;
  values
    .iter()
    .all(|value| value == first)
    .then(|| first.clone())
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
mod test_is_full_frame {
  use super::is_full_frame;

  #[test]
  fn rounding_to_even_size_is_full_frame() {
    // A bar-free 321x181 frame can be reported as 320:180:0:0.
    assert!(is_full_frame("320:180:0:0", 321, 181));
  }

  #[test]
  fn exact_full_frame_is_full_frame() {
    assert!(is_full_frame("320:240:0:0", 320, 240));
  }

  #[test]
  fn real_bars_are_not_full_frame() {
    assert!(!is_full_frame("320:180:0:30", 320, 240));
  }

  #[test]
  fn garbage_is_not_full_frame() {
    assert!(!is_full_frame("not-a-crop", 320, 240));
  }
}

#[cfg(test)]
mod test_frame_size_fields {
  use super::parse_frame_size;

  #[test]
  fn reads_plain_size() {
    assert_eq!(parse_frame_size("320,240\n"), Some((320, 240)));
  }

  #[test]
  fn reads_size_with_trailing_comma_from_rotation() {
    assert_eq!(parse_frame_size("320,180,\n"), Some((320, 180)));
  }

  #[test]
  fn swaps_size_for_a_quarter_turn() {
    assert_eq!(parse_frame_size("320,180,-90\n"), Some((180, 320)));
    assert_eq!(parse_frame_size("320,180,90\n"), Some((180, 320)));
  }

  #[test]
  fn keeps_size_for_a_half_turn() {
    assert_eq!(parse_frame_size("320,180,180\n"), Some((320, 180)));
  }
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
  fn samples_long_clips_about_sixty_six_times() {
    assert_eq!(sampling_filter(600.0), "fps=0.11,cropdetect=round=2:skip=0");
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
mod test_uniform_crop {
  use super::uniform_crop;

  #[test]
  fn returns_the_crop_when_every_frame_agrees() {
    let values: Vec<String> = ["320:180:0:30", "320:180:0:30"].map(String::from).to_vec();
    assert_eq!(uniform_crop(&values), Some("320:180:0:30".to_string()));
  }

  #[test]
  fn returns_none_when_frames_disagree() {
    // A cutscene: some frames letterboxed, some full frame.
    let values: Vec<String> = ["320:180:0:30", "320:240:0:0"].map(String::from).to_vec();
    assert_eq!(uniform_crop(&values), None);
  }

  #[test]
  fn returns_none_for_no_values() {
    assert_eq!(uniform_crop(&[]), None);
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

/// Integration tests: these call real ffmpeg and ffprobe on generated clips.
#[cfg(test)]
mod test_cropper_integration {
  use super::{crop_black_bars, detect_black_bars, frame_size};
  use std::path::PathBuf;
  use std::process::Command;

  /// A generated clip that is removed when dropped.
  struct Clip(PathBuf);

  impl Clip {
    /// Creates a 2-second 320x240 clip. With `bars`, a white 320x180 picture sits between
    /// 30 px black bars at the top and bottom. Without it, the picture fills the frame.
    fn new(name: &str, bars: bool) -> Self {
      let path =
        std::env::temp_dir().join(format!("vidmerger-crop-{name}-{}.mp4", std::process::id()));
      let mut cmd = Command::new("ffmpeg");
      cmd.args(["-v", "error", "-y", "-f", "lavfi", "-i"]);
      if bars {
        cmd.arg("color=c=white:s=320x180:d=2:r=25");
        cmd.args(["-vf", "pad=320:240:0:30:black"]);
      } else {
        cmd.arg("color=c=white:s=320x240:d=2:r=25");
      }
      let status = cmd
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-an"])
        .arg(&path)
        .status()
        .expect("ffmpeg must be installed");
      assert!(status.success(), "failed to create test clip");
      Clip(path)
    }

    fn path(&self) -> &str {
      self.0.to_str().unwrap()
    }
  }

  impl Drop for Clip {
    fn drop(&mut self) {
      let _ = std::fs::remove_file(&self.0);
    }
  }

  #[test]
  fn frame_size_reads_the_real_clip_size() {
    let clip = Clip::new("size", false);
    assert_eq!(frame_size(clip.path()), Some((320, 240)));
  }

  #[test]
  fn detect_black_bars_finds_the_bars_on_a_real_clip() {
    let clip = Clip::new("bars", true);
    // The 320x180 picture between the bars is cropped out of 320x240.
    assert_eq!(
      detect_black_bars(clip.path()),
      Some("320:180:0:30".to_string())
    );
  }

  #[test]
  fn detect_black_bars_returns_none_for_a_full_frame_clip() {
    let clip = Clip::new("full", false);
    assert_eq!(detect_black_bars(clip.path()), None);
  }

  #[test]
  fn crop_black_bars_writes_cropped_file_and_returns_new_paths() {
    let barred = Clip::new("crop-barred", true);
    let full = Clip::new("crop-full", false);
    let tmp = std::env::temp_dir().join(format!("vidmerger-crop-run-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let files = vec![barred.path().to_string(), full.path().to_string()];
    let (new_files, content) = crop_black_bars(&files, &tmp).expect("bars should be found");

    // The barred clip is cropped into tmp/cropper. The full-frame clip is used as it is.
    assert!(
      new_files[0].contains("cropper"),
      "not cropped: {}",
      new_files[0]
    );
    assert_eq!(new_files[1], files[1]);
    assert!(std::path::Path::new(&new_files[0]).exists());
    assert!(content.contains("file '"), "no concat list: {content}");

    let _ = std::fs::remove_dir_all(&tmp);
  }
}
