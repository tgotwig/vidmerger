use crate::commanders::_cmd;
use crate::helpers::str_helper::gen_input_file_content_for_ffmpeg;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

/// Containers that use the MP4 muxer. They reject many audio codecs (e.g. `pcm_alaw`).
const MP4_FAMILY_FORMATS: [&str; 6] = ["mp4", "m4a", "m4v", "m4b", "mov", "3gp"];
/// Audio codecs that MP4 output can store without re-encoding.
const MP4_SAFE_AUDIO_CODECS: [&str; 7] = ["aac", "mp3", "ac3", "eac3", "alac", "opus", "flac"];
/// Audio codecs that MOV output can store without re-encoding (FLAC and Opus fail in MOV).
const MOV_SAFE_AUDIO_CODECS: [&str; 5] = ["aac", "mp3", "ac3", "eac3", "alac"];
/// Audio codecs that 3GP output can store without re-encoding (only AAC works in 3GP).
const THREE_GP_SAFE_AUDIO_CODECS: [&str; 1] = ["aac"];

/// Audio codecs the given MP4-family output format can store, or `None` if the format
/// is not MP4-family.
fn safe_audio_codecs_for(file_format: &str) -> Option<&'static [&'static str]> {
  match file_format.to_lowercase().as_str() {
    "mov" => Some(&MOV_SAFE_AUDIO_CODECS),
    "3gp" => Some(&THREE_GP_SAFE_AUDIO_CODECS),
    format if MP4_FAMILY_FORMATS.contains(&format) => Some(&MP4_SAFE_AUDIO_CODECS),
    _ => None,
  }
}

/// True when the output is an MP4-family file and at least one input has an audio
/// codec that the output container cannot store. Such audio must be re-encoded to AAC.
fn needs_audio_reencode(file_format: &str, files_to_merge_as_strings: &[String]) -> bool {
  let Some(safe_codecs) = safe_audio_codecs_for(file_format) else {
    return false;
  };
  files_to_merge_as_strings.iter().any(|path| {
    // A failed probe (`None`) is treated as unsafe: re-encoding a file with no audio is harmless.
    _cmd::get_audio_codecs(path).is_none_or(|codecs| {
      codecs
        .iter()
        .any(|codec| !safe_codecs.contains(&codec.as_str()))
    })
  })
}

pub fn merge(
  input: String,
  output: String,
  files_to_merge_as_strings: Vec<String>,
  file_format: &str,
  tmp_dir: PathBuf,
) {
  let extra_args: &[&str] = if needs_audio_reencode(file_format, &files_to_merge_as_strings) {
    &["-c:a", "aac"]
  } else {
    &[]
  };

  // Untruncated total length, used as the progress bar denominator.
  let durations: Vec<f64> = files_to_merge_as_strings
    .iter()
    .map(|path| _cmd::get_media_seconds(path).unwrap())
    .collect();
  let total_seconds: f64 = durations.iter().sum();
  let bounds = chapter_bounds_ms(&durations);

  let mut metadata_string = String::from(";FFMETADATA1\n");
  for (path, (start_ms, end_ms)) in files_to_merge_as_strings.iter().zip(bounds) {
    let title = extract_title(path, file_format);
    metadata_string.push_str(&format!(
      "\n[CHAPTER]\nTIMEBASE=1/1000\nSTART={}\nEND={}\ntitle={}\n",
      start_ms, end_ms, title
    ));
  }
  let mut file_path = tmp_dir.clone();
  file_path.push("chapters.txt");
  let mut file = File::create(&file_path).expect("Failed to create file");
  file
    .write_all(metadata_string.as_bytes())
    .expect("Failed to write to file");

  // ========== Merge with chapters ========== //

  let output_str = output.clone();
  let chapters = file_path.to_string_lossy().to_string();
  let mut res = _cmd::merge(input.clone(), &output, &chapters, total_seconds, extra_args);

  // Fallback for issue #53: an input has a stream MP4 cannot hold (e.g. a `tmcd` data
  // stream in a .mov). Convert each input to a clean MP4, point the concat list at
  // the converted files, and merge once more.
  if !res.status.success() && is_unsupported_stream_error(&res.stderr) {
    println!("⚠️  Unsupported stream found, converting inputs and retrying the merge");
    let converted = convert_inputs(&files_to_merge_as_strings, &tmp_dir);
    fs::write(&input, gen_input_file_content_for_ffmpeg(&converted))
      .expect("Failed to rewrite the concat list");
    res = _cmd::merge(input, &output, &chapters, total_seconds, extra_args);
  }

  if res.status.success() {
    println!("🐣 Generated: {}", BrightBlue.paint(&output_str));
  } else {
    panic!(
      "❌ Something went wrong (exit code: {:?}):{}",
      res.status.code(),
      String::from_utf8_lossy(&res.stderr)
    );
  }
}

/// Returns the `(start_ms, end_ms)` of each chapter, given each clip's duration in seconds.
/// The running total is kept unrounded and rounded once per boundary, so the error
/// does not grow with the number of clips.
fn chapter_bounds_ms(durations: &[f64]) -> Vec<(i64, i64)> {
  let mut elapsed_seconds: f64 = 0.0;
  durations
    .iter()
    .map(|duration| {
      let start_ms = (elapsed_seconds * 1000.0).round() as i64;
      elapsed_seconds += duration;
      let end_ms = (elapsed_seconds * 1000.0).round() as i64;
      (start_ms, end_ms)
    })
    .collect()
}

/// True when FFmpeg failed because an input stream has a type the output cannot hold.
fn is_unsupported_stream_error(stderr: &[u8]) -> bool {
  String::from_utf8_lossy(stderr).contains("unsupported type")
}

/// Converts every input to an MP4 in `tmp_dir` (see `_cmd::convert_for_merge`) and
/// returns the new paths in the same order as the inputs.
fn convert_inputs(files: &[String], tmp_dir: &Path) -> Vec<String> {
  files
    .iter()
    .enumerate()
    .map(|(index, input)| {
      let output = tmp_dir.join(format!("converted_{index}.mp4"));
      let output = output.to_string_lossy().to_string();
      let res = _cmd::convert_for_merge(input, &output);
      if !res.status.success() {
        panic!(
          "❌ Converting {} failed (exit code: {:?}):{}",
          input,
          res.status.code(),
          String::from_utf8_lossy(&res.stderr)
        );
      }
      output
    })
    .collect()
}

fn extract_title(path: &str, file_format: &str) -> String {
  let file_name = path.split('/').next_back().unwrap_or("");
  let mut parts = file_name.splitn(2, '-');
  parts.next(); // Skip the part before the first '-'
  let content_with_extension = parts.next().unwrap_or("").trim();

  let format_str = format!(".{file_format}");
  content_with_extension
    .split(&format_str)
    .next()
    .unwrap_or("")
    .trim()
    .to_string()
}

#[cfg(test)]
mod test_chapter_bounds_ms {
  use super::chapter_bounds_ms;

  #[test]
  fn test_chapter_bounds_with_fractional_durations() {
    let durations = [1.40, 2.64, 1.96, 3.32];
    assert_eq!(
      chapter_bounds_ms(&durations),
      vec![(0, 1400), (1400, 4040), (4040, 6000), (6000, 9320)]
    );
  }

  #[test]
  fn test_chapter_bounds_do_not_drift_over_many_clips() {
    let durations = [1.0003; 100];
    let bounds = chapter_bounds_ms(&durations);
    assert_eq!(bounds.last(), Some(&(99_030, 100_030)));
  }

  #[test]
  fn test_chapter_bounds_empty() {
    assert!(chapter_bounds_ms(&[]).is_empty());
  }
}

#[cfg(test)]
mod test_extract_title {
  use super::extract_title;

  #[test]
  fn test_extract_title() {
    let path = "path/to/video-Title of Video.mp4";
    assert_eq!(extract_title(path, "mp4"), "Title of Video");
  }

  #[test]
  fn test_extract_title_with_dot() {
    let path = "path/to/video-[1.0] Title of Video.mp4";
    assert_eq!(extract_title(path, "mp4"), "[1.0] Title of Video");
  }

  #[test]
  fn test_extract_title_with_no_dash() {
    let path = "path/to/videoTitle of Video.mp4";
    assert_eq!(extract_title(path, "mp4"), "");
  }

  #[test]
  fn test_extract_title_with_no_extension() {
    let path = "path/to/video-Title of Video";
    assert_eq!(extract_title(path, "mp4"), "Title of Video");
  }

  #[test]
  fn test_extract_title_with_multiple_dashes() {
    let path = "path/to/video-Title-of-Video.mp4";
    assert_eq!(extract_title(path, "mp4"), "Title-of-Video");
  }

  #[test]
  fn test_extract_title_with_empty_path() {
    let path = "";
    assert_eq!(extract_title(path, "mp4"), "");
  }

  // FAIL
  #[test]
  fn test_extract_title_with_only_dashes() {
    let path = "---";
    assert_eq!(extract_title(path, "mp4"), "--");
  }

  #[test]
  fn test_extract_title_with_special_characters() {
    let path = "path/to/video-Title_@_of_Video.mp4";
    assert_eq!(extract_title(path, "mp4"), "Title_@_of_Video");
  }

  #[test]
  fn test_extract_title_with_different_format() {
    let path = "path/to/video-Title of Video.avi";
    assert_eq!(extract_title(path, "mp4"), "Title of Video.avi");
  }

  #[test]
  fn test_extract_title_with_nested_path() {
    let path = "path/to/some/other/folder/video-Title.mp4";
    assert_eq!(extract_title(path, "mp4"), "Title");
  }
}
