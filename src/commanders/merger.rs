use crate::commanders::_cmd::{self, AudioProfile, MediaStream};
use crate::helpers::str_helper::gen_input_file_content_for_ffmpeg;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::str;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

/// What happens to one input before the merge.
#[derive(Debug, PartialEq, Eq)]
enum InputAction {
  /// Use the file as it is (stream copy).
  Copy,
  /// Re-encode the file to the shared profile. `silent_audio` adds silence when the file has no audio.
  Convert { silent_audio: bool },
}

/// The shared audio profile and one action per input (same order as the inputs).
#[derive(Debug, PartialEq, Eq)]
struct MergePlan {
  profile: Option<AudioProfile>,
  actions: Vec<InputAction>,
}

/// The audio settings of the first audio stream found in any input, or `None` if no input has audio.
fn audio_profile(probes: &[Vec<MediaStream>]) -> Option<AudioProfile> {
  let stream = probes
    .iter()
    .flatten()
    .find(|stream| stream.kind == "audio")?;
  Some(AudioProfile {
    sample_rate: stream.sample_rate?,
    channels: stream.channels?,
  })
}

/// The audio codec an input must have to be copied into the output format.
/// Converted inputs are always AAC, so MP3 output needs MP3 audio.
fn copyable_audio_codec(file_format: &str) -> &'static str {
  if file_format == "mp3" {
    "mp3"
  } else {
    "aac"
  }
}

/// Decides how each input is merged. An input is copied only when its audio has the codec the
/// output format needs, the shared profile, and it has no extra streams (e.g. `tmcd`).
/// With `force_convert`, every input is converted.
fn plan_merge(probes: &[Vec<MediaStream>], file_format: &str, force_convert: bool) -> MergePlan {
  let profile = audio_profile(probes);
  let codec = copyable_audio_codec(file_format);
  let matches_profile = |stream: &MediaStream| {
    stream.codec == codec
      && profile.is_some_and(|p| {
        Some(p.sample_rate) == stream.sample_rate && Some(p.channels) == stream.channels
      })
  };
  let actions = probes
    .iter()
    .map(|streams| {
      let has_audio = streams.iter().any(|s| s.kind == "audio");
      let has_video = streams.iter().any(|s| s.kind == "video");
      let has_extra = streams
        .iter()
        .any(|s| s.kind != "audio" && s.kind != "video");
      let audio_fits = streams
        .iter()
        .filter(|s| s.kind == "audio")
        .all(matches_profile);
      // Silence is only added when there is video, because `-shortest` ends the output with it.
      let silent_audio = profile.is_some() && !has_audio && has_video;
      if force_convert || has_extra || !audio_fits || silent_audio {
        InputAction::Convert { silent_audio }
      } else {
        InputAction::Copy
      }
    })
    .collect();
  MergePlan { profile, actions }
}

pub fn merge(
  input: String,
  output: String,
  files_to_merge_as_strings: Vec<String>,
  file_format: &str,
  tmp_dir: PathBuf,
) {
  // Probe every input once. The plan decides which inputs can be copied as they are.
  let probes: Vec<Vec<MediaStream>> = files_to_merge_as_strings
    .iter()
    .map(|path| {
      _cmd::probe_streams(path).unwrap_or_else(|err| panic!("❌ Probing {path} failed: {err}"))
    })
    .collect();

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
  // First try: copy inputs that already fit and convert the rest.
  let plan = plan_merge(&probes, file_format, false);
  let mut res = run_plan(
    &plan,
    &files_to_merge_as_strings,
    &tmp_dir,
    &input,
    &output,
    &chapters,
    total_seconds,
  );

  // Fallback for issue #53: an input has a stream MP4 cannot hold (e.g. a `tmcd` data
  // stream in a .mov). Convert every input to the shared profile and merge once more.
  if !res.status.success() && is_unsupported_stream_error(&res.stderr) {
    println!("⚠️  Unsupported stream found, converting every input and retrying the merge");
    let plan = plan_merge(&probes, file_format, true);
    res = run_plan(
      &plan,
      &files_to_merge_as_strings,
      &tmp_dir,
      &input,
      &output,
      &chapters,
      total_seconds,
    );
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

/// Converts the inputs as `plan` says, rewrites the concat list to use them, and runs the merge.
fn run_plan(
  plan: &MergePlan,
  files: &[String],
  tmp_dir: &Path,
  concat_list: &str,
  output: &str,
  chapters: &str,
  total_seconds: f64,
) -> Output {
  let inputs = convert_inputs(files, plan, tmp_dir);
  fs::write(concat_list, gen_input_file_content_for_ffmpeg(&inputs))
    .expect("Failed to write the concat list");
  _cmd::merge(concat_list.to_string(), output, chapters, total_seconds)
}

/// Returns the path to merge for each input, in the same order as the inputs. Copied inputs
/// keep their path. Converted inputs are written as MP4 files in `tmp_dir`.
fn convert_inputs(files: &[String], plan: &MergePlan, tmp_dir: &Path) -> Vec<String> {
  files
    .iter()
    .zip(&plan.actions)
    .enumerate()
    .map(|(index, (input, action))| match action {
      InputAction::Copy => input.clone(),
      InputAction::Convert { silent_audio } => {
        let output = tmp_dir.join(format!("converted_{index}.mp4"));
        let output = output.to_string_lossy().to_string();
        let res = _cmd::convert_for_merge(input, &output, plan.profile, *silent_audio);
        if !res.status.success() {
          panic!(
            "❌ Converting {} failed (exit code: {:?}):{}",
            input,
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
          );
        }
        output
      }
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
mod test_plan_merge {
  use super::{InputAction, MergePlan};
  use crate::commanders::_cmd::{AudioProfile, MediaStream};

  /// Plans for an MP4 output, the common case.
  fn plan_merge(probes: &[Vec<MediaStream>], force_convert: bool) -> MergePlan {
    super::plan_merge(probes, "mp4", force_convert)
  }

  fn video() -> MediaStream {
    stream("video", "h264")
  }

  fn stream(kind: &str, codec: &str) -> MediaStream {
    MediaStream {
      kind: kind.into(),
      codec: codec.into(),
      sample_rate: None,
      channels: None,
    }
  }

  fn audio(codec: &str, sample_rate: u32, channels: u32) -> MediaStream {
    MediaStream {
      kind: "audio".into(),
      codec: codec.into(),
      sample_rate: Some(sample_rate),
      channels: Some(channels),
    }
  }

  fn aac_stereo_48k() -> MediaStream {
    audio("aac", 48_000, 2)
  }

  const STEREO_48K: AudioProfile = AudioProfile {
    sample_rate: 48_000,
    channels: 2,
  };

  #[test]
  fn copies_inputs_that_already_match() {
    let probes = vec![
      vec![video(), aac_stereo_48k()],
      vec![video(), aac_stereo_48k()],
    ];
    assert_eq!(
      plan_merge(&probes, false),
      MergePlan {
        profile: Some(STEREO_48K),
        actions: vec![InputAction::Copy, InputAction::Copy],
      }
    );
  }

  #[test]
  fn converts_only_the_input_with_unsupported_audio() {
    let probes = vec![
      vec![video(), aac_stereo_48k()],
      vec![video(), audio("pcm_alaw", 8_000, 1)],
    ];
    assert_eq!(
      plan_merge(&probes, false).actions,
      vec![
        InputAction::Copy,
        InputAction::Convert {
          silent_audio: false
        }
      ]
    );
  }

  #[test]
  fn converts_input_with_different_sample_rate() {
    let probes = vec![
      vec![video(), aac_stereo_48k()],
      vec![video(), audio("aac", 44_100, 2)],
    ];
    assert_eq!(
      plan_merge(&probes, false).actions,
      vec![
        InputAction::Copy,
        InputAction::Convert {
          silent_audio: false
        }
      ]
    );
  }

  #[test]
  fn converts_input_with_data_stream() {
    let probes = vec![vec![video(), aac_stereo_48k(), stream("data", "tmcd")]];
    assert_eq!(
      plan_merge(&probes, false).actions,
      vec![InputAction::Convert {
        silent_audio: false
      }]
    );
  }

  #[test]
  fn adds_silence_to_video_input_without_audio() {
    let probes = vec![vec![video(), aac_stereo_48k()], vec![video()]];
    assert_eq!(
      plan_merge(&probes, false).actions,
      vec![
        InputAction::Copy,
        InputAction::Convert { silent_audio: true }
      ]
    );
  }

  #[test]
  fn does_not_add_silence_without_video() {
    let probes = vec![
      vec![stream("data", "tmcd")],
      vec![video(), aac_stereo_48k()],
    ];
    assert_eq!(
      plan_merge(&probes, false).actions,
      vec![
        InputAction::Convert {
          silent_audio: false
        },
        InputAction::Copy
      ]
    );
  }

  #[test]
  fn copies_everything_when_no_input_has_audio() {
    let probes = vec![vec![video()], vec![video()]];
    assert_eq!(
      plan_merge(&probes, false),
      MergePlan {
        profile: None,
        actions: vec![InputAction::Copy, InputAction::Copy],
      }
    );
  }

  #[test]
  fn force_convert_converts_every_input() {
    let probes = vec![
      vec![video(), aac_stereo_48k()],
      vec![video(), aac_stereo_48k()],
    ];
    assert_eq!(
      plan_merge(&probes, true).actions,
      vec![
        InputAction::Convert {
          silent_audio: false
        },
        InputAction::Convert {
          silent_audio: false
        },
      ]
    );
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
