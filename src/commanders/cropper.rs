use crate::commanders::_cmd::{
  get_media_seconds, h264_encoder, is_hevc, keep_source_bit_rate, probe_streams,
};
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
  // Each file's duration is read once here, and reused for the sampling and for the encode.
  let detected: Vec<Option<(String, f64)>> =
    files.iter().map(|file| detect_black_bars(file)).collect();
  if detected.iter().all(Option::is_none) {
    return None;
  }
  let crops: Vec<Option<(String, f64)>> = detected;

  let dir = tmp_dir.join("cropper");
  fs::create_dir_all(&dir).expect("Failed to create the cropper folder");

  let inputs: Vec<String> = files
    .iter()
    .zip(crops)
    .map(|(file, detected)| {
      let Some((crop, duration)) = detected else {
        return file.clone();
      };
      let file_name = Path::new(file).file_name().unwrap().to_string_lossy();
      let output = dir.join(file_name.as_ref()).to_string_lossy().to_string();
      println!("✂️  Removing black bars from {file}, calling: crop={crop}");

      let res = crop_with_cpu_fallback(file, &crop, &output, duration);
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

/// Returns the crop value (`w:h:x:y`) and the file's duration for a file that has black bars,
/// or `None` when it has no video, has no bars, or the detection fails. The duration is returned
/// so the encode step can reuse it instead of probing the file again.
fn detect_black_bars(file: &str) -> Option<(String, f64)> {
  let streams = probe_streams(file).ok()?;
  if !streams.iter().any(|stream| stream.kind == "video") {
    return None;
  }
  let (width, height) = frame_size(file)?;

  let duration = get_media_seconds(file).ok()?;
  // A few spread-out samples first. If every one is the full frame, the file almost certainly
  // has no bars, so the full check (about 66 seeks, several seconds per file) is skipped.
  let quick = sample_crops(file, &seek_times_for(duration, PRE_CHECK_POINTS))?;
  if all_full_frame(&quick, width, height) {
    return None;
  }
  let crops = sample_crops(file, &seek_times(duration))?;
  let crop = majority_crop(&crops)?;

  if is_full_frame(&crop, width, height) {
    None
  } else {
    Some((crop, duration))
  }
}

/// True only when there is at least one reading and every reading is the full frame. An empty
/// list is not evidence of "no bars", so it returns false and the full check still runs.
fn all_full_frame(crops: &[String], width: u32, height: u32) -> bool {
  !crops.is_empty() && crops.iter().all(|crop| is_full_frame(crop, width, height))
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

/// Number of frames read at each seek point. Two frames avoid relying on a single frame.
const FRAMES_PER_SEEK: u32 = 2;

/// Runs `cropdetect` at each seek time and returns every crop value it reports. Seeking jumps
/// straight to each point, so the whole file is not decoded. `round=2` keeps the exact bar size.
/// `skip=0` checks every sampled frame. Returns `None` if FFmpeg cannot be run.
fn sample_crops(file: &str, times: &[f64]) -> Option<Vec<String>> {
  let mut crops = Vec::new();
  for time in times {
    let output = Command::new("ffmpeg")
      .args(["-v", "info", "-ss", &format!("{time:.3}"), "-i", file])
      .args(["-frames:v", &FRAMES_PER_SEEK.to_string()])
      .args(["-vf", "cropdetect=round=2:skip=0", "-an", "-f", "null", "-"])
      .output()
      .ok()?;
    crops.extend(parse_crop_values(&String::from_utf8_lossy(&output.stderr)));
  }
  Some(crops)
}

/// Number of seek points in the quick check that decides whether a file can be skipped.
const PRE_CHECK_POINTS: u32 = 5;

/// Returns `SAMPLES_PER_CLIP` evenly spaced times (in seconds) across `duration_seconds`.
/// Each time is a seek point, so FFmpeg jumps there instead of decoding the whole file.
fn seek_times(duration_seconds: f64) -> Vec<f64> {
  seek_times_for(duration_seconds, SAMPLES_PER_CLIP as u32)
}

/// Returns `points` evenly spaced times (in seconds) across `duration_seconds`.
fn seek_times_for(duration_seconds: f64, points: u32) -> Vec<f64> {
  (0..points)
    .map(|index| duration_seconds * (f64::from(index) + 0.5) / f64::from(points))
    .collect()
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
/// Share of sampled frames that must agree on the same bars before the video is cropped.
/// Most of a film can be letterboxed while a few scenes are full frame, so requiring every frame
/// to agree would skip the whole film. The few full-frame scenes are the minority and are not
/// the majority value, so they are not what decides the crop.
const MIN_AGREEING_SHARE: f64 = 0.6;

/// Returns the crop value that at least `MIN_AGREEING_SHARE` of the sampled frames agree on, or
/// `None` when no value is that common (e.g. a clip with an even mix of framings).
fn majority_crop(values: &[String]) -> Option<String> {
  if values.is_empty() {
    return None;
  }
  let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
  for value in values {
    *counts.entry(value.as_str()).or_default() += 1;
  }
  let (crop, count) = counts.into_iter().max_by_key(|(_, count)| *count)?;
  let share = count as f64 / values.len() as f64;
  (share >= MIN_AGREEING_SHARE).then(|| crop.to_string())
}

/// Crops `input` with the normal encoder. If that fails (e.g. `hevc_videotoolbox` rejects the
/// input), crops again with the CPU encoder `libx264`, so the crop step does not panic on a
/// hardware encoder error.
fn crop_with_cpu_fallback(
  input: &str,
  crop: &str,
  output: &str,
  duration_seconds: f64,
) -> std::process::Output {
  let run = |encoder_override: Option<&str>| {
    Command::new("ffmpeg")
      .args(encode_args(
        input,
        crop,
        output,
        encoder_override,
        duration_seconds,
      ))
      .output()
      .unwrap()
  };
  let res = run(None);
  if res.status.success() {
    return res;
  }
  println!("⚠️  Cropping with the default encoder failed, retrying with libx264");
  run(Some("libx264"))
}

/// The FFmpeg arguments that crop `input` to `crop` and write `output`.
/// Video, audio and subtitles are mapped. Data streams such as `tmcd` are dropped.
/// Chapters and metadata are copied from the input by FFmpeg's defaults.
fn encode_args(
  input: &str,
  crop: &str,
  output: &str,
  encoder_override: Option<&str>,
  duration_seconds: f64,
) -> Vec<String> {
  let mut args: Vec<String> = [
    "-y", "-v", "error", "-i", input, "-map", "0:v:0", "-map", "0:a?", "-map", "0:s?",
  ]
  .map(String::from)
  .to_vec();
  // Subtitles are copied unchanged, so they are kept as they are.
  args.extend(["-c:s", "copy", "-vf"].map(String::from));
  args.push(format!("crop={crop}"));
  if let Some(encoder) = encoder_override.or_else(|| video_encoder_for(output)) {
    // Keep HEVC input as HEVC (VideoToolbox on macOS), instead of re-encoding it to H.264.
    // A retry passes `encoder_override`, which replaces this choice.
    let encoder = if encoder_override.is_none() && is_hevc(input) && cfg!(target_os = "macos") {
      "hevc_videotoolbox"
    } else {
      encoder
    };
    args.extend(["-c:v".to_string(), encoder.to_string()]);
    if encoder.starts_with("hevc") {
      args.extend(["-tag:v", "hvc1"].map(String::from));
    }
    // Keep the source's video bitrate on long inputs (see `keep_source_bit_rate`).
    if let Some(bit_rate) = keep_source_bit_rate(input, duration_seconds) {
      args.extend(["-b:v".to_string(), bit_rate.to_string()]);
    }
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
mod test_all_full_frame {
  use super::all_full_frame;

  #[test]
  fn empty_readings_do_not_count_as_no_bars() {
    assert!(!all_full_frame(&[], 320, 240));
  }

  #[test]
  fn every_full_frame_reading_skips_the_file() {
    let readings = vec!["320:240:0:0".to_string(); 5];
    assert!(all_full_frame(&readings, 320, 240));
  }

  #[test]
  fn one_bar_reading_keeps_the_file_for_the_full_check() {
    let readings = vec!["320:240:0:0".to_string(), "320:180:0:30".to_string()];
    assert!(!all_full_frame(&readings, 320, 240));
  }
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
  use super::{is_valid_crop, parse_crop_values, seek_times};

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
  fn seek_times_are_spread_across_the_whole_clip() {
    let times = seek_times(100.0);
    assert_eq!(times.len(), 66);
    assert!(
      times.first().unwrap() < &1.0,
      "first sample is near the start"
    );
    assert!(times.last().unwrap() > &99.0, "last sample is near the end");
  }

  #[test]
  fn seek_times_stay_inside_the_clip() {
    assert!(seek_times(8552.32).iter().all(|t| *t > 0.0 && *t < 8552.32));
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
mod test_majority_crop {
  use super::majority_crop;

  #[test]
  fn returns_the_crop_when_every_frame_agrees() {
    let values: Vec<String> = ["320:180:0:30", "320:180:0:30"].map(String::from).to_vec();
    assert_eq!(majority_crop(&values), Some("320:180:0:30".to_string()));
  }

  #[test]
  fn letterboxed_film_with_a_few_full_frame_scenes_is_cropped() {
    // 9 letterboxed samples and 1 full-frame sample: 90% agree, so the bars are cropped.
    let mut values: Vec<String> = vec!["3840:1608:0:276".to_string(); 9];
    values.push("3840:2160:0:0".to_string());
    assert_eq!(majority_crop(&values), Some("3840:1608:0:276".to_string()));
  }

  #[test]
  fn a_letterboxed_film_with_many_offset_frames_is_still_cropped() {
    // Measured on a 2h22m film: 136 of 198 readings were the letterbox (about 69%).
    let mut values: Vec<String> = vec!["3840:1608:0:276".to_string(); 136];
    values.extend(vec!["3828:1608:12:276".to_string(); 62]);
    assert_eq!(majority_crop(&values), Some("3840:1608:0:276".to_string()));
  }

  #[test]
  fn an_even_mix_is_not_cropped() {
    let values: Vec<String> = ["320:180:0:30", "320:240:0:0"].map(String::from).to_vec();
    assert_eq!(majority_crop(&values), None);
  }

  #[test]
  fn returns_none_for_no_values() {
    assert_eq!(majority_crop(&[]), None);
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
    let args = encode_args("in.webm", "1920:688:0:16", "out.webm", None, 1.0);
    assert!(args
      .windows(2)
      .any(|pair| pair == ["-vf", "crop=1920:688:0:16"]));
    assert!(args.windows(2).any(|pair| pair == ["-map", "0:v:0"]));
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
    let (crop, duration) = detect_black_bars(clip.path()).expect("bars should be found");
    assert_eq!(crop, "320:180:0:30");
    // The test clip is 2 seconds long.
    assert!((duration - 2.0).abs() < 0.5, "duration was {duration}");
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
    // The guard removes the folder when the test ends, even if an assert fails.
    let tmp = TempFolder::new("crop-run");

    let files = vec![barred.path().to_string(), full.path().to_string()];
    let (new_files, content) = crop_black_bars(&files, &tmp.0).expect("bars should be found");

    // The barred clip is cropped into tmp/cropper. The full-frame clip is used as it is.
    assert!(
      new_files[0].contains("cropper"),
      "not cropped: {}",
      new_files[0]
    );
    assert_eq!(new_files[1], files[1]);
    assert!(std::path::Path::new(&new_files[0]).exists());
    assert!(content.contains("file '"), "no concat list: {content}");
  }

  /// A temp folder that is removed when dropped.
  struct TempFolder(PathBuf);

  impl TempFolder {
    fn new(name: &str) -> Self {
      let dir = std::env::temp_dir().join(format!("vidmerger-{name}-{}", std::process::id()));
      let _ = std::fs::remove_dir_all(&dir);
      std::fs::create_dir_all(&dir).unwrap();
      TempFolder(dir)
    }
  }

  impl Drop for TempFolder {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.0);
    }
  }
}
