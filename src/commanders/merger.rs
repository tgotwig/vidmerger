use crate::commanders::_cmd;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
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
    _cmd::get_audio_codecs(path).map_or(true, |codecs| {
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

  let mut start_time = 0;
  // Untruncated total length, used as the progress bar denominator.
  let mut total_seconds = 0.0;
  let mut metadata_string = String::from(";FFMETADATA1\n");
  for path in files_to_merge_as_strings {
    let title = extract_title(&path, file_format);
    let duration = _cmd::get_media_seconds(&path).unwrap();
    total_seconds += duration;
    let duration_secs = duration as i64;

    metadata_string.push_str(&format!(
      "\n[CHAPTER]\nTIMEBASE=1/1\nSTART={}\nEND={}\ntitle={}\n",
      start_time,
      start_time + duration_secs,
      title
    ));

    start_time += duration_secs;
  }
  let mut file_path = tmp_dir;
  file_path.push("chapters.txt");
  let mut file = File::create(&file_path).expect("Failed to create file");
  file
    .write_all(metadata_string.as_bytes())
    .expect("Failed to write to file");

  // ========== Merge with chapters ========== //

  let output_str = output.clone();
  let res = _cmd::merge(
    input,
    &output,
    &file_path.to_string_lossy(),
    total_seconds,
    extra_args,
  );

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
