mod e2e {
  use assert_cmd::{assert::Assert, Command};
  use k9::assertions::{greater_than::assert_greater_than, lesser_than::assert_lesser_than};
  use regex::Regex;
  use std::fs;
  use std::fs::File;
  use std::io;
  use stdext::function_name;

  static BIN: &'static str = "vidmerger";
  static DATA_1_MP3: &'static str = "1 - Chapter 1.mp3";
  static DATA_1_MP4: &'static str = "1 - Chapter 1.mp4";
  static DATA_2_MP3: &'static str = "2 - Chapter 2.mp3";
  static DATA_2_MP4: &'static str = "2 - Chapter 2.mp4";
  static DATA_3_MP4: &'static str = ".3.mp4";

  #[cfg(test)]
  #[ctor::ctor(unsafe)]
  fn prepare() {
    use std::fs::{self, File};

    println!("👷 Doing preparations...");

    // Keep the downloaded samples (the cache). Only remove the test folders inside `data`.
    fs::create_dir_all("data").unwrap();
    for entry in fs::read_dir("data").unwrap() {
      let path = entry.unwrap().path();
      if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
      }
    }

    download("https://vidmerger.s3.eu-central-1.amazonaws.com/1+-+Chapter+1.mp4");
    download("https://vidmerger.s3.eu-central-1.amazonaws.com/2+-+Chapter+2.mp4");
    File::create(format!("data/{}", DATA_3_MP4)).unwrap();

    download("https://vidmerger.s3.eu-central-1.amazonaws.com/1+-+Chapter+1.mp3");
    download("https://vidmerger.s3.eu-central-1.amazonaws.com/2+-+Chapter+2.mp3");

    println!("✅ Preparations done!");
  }

  #[test]
  fn call_merger() {
    let test_name = function_name!().split("::").last().unwrap();
    prep(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    check_for_merged_file(test_name, "output.mp4");
  }

  #[test]
  fn call_merger_on_audio_files() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_audio(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    check_for_merged_file(test_name, "output.mp3");
  }

  #[test]
  fn call_merger_with_alaw_audio_reencodes_to_aac() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_alaw_audio(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    let probe = Command::new("ffprobe")
      .args(["-v", "error", "-select_streams", "a:0"])
      .args(["-show_entries", "stream=codec_name", "-of", "csv=p=0"])
      .arg(format!("data/{}/output.mp4", test_name))
      .output()
      .unwrap();
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "aac");
  }

  #[test]
  fn call_merger_with_timecode_stream_falls_back_and_merges() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_timecode_stream(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    let probe = Command::new("ffprobe")
      .args([
        "-v",
        "error",
        "-show_entries",
        "stream=codec_type,codec_tag_string",
        "-of",
        "csv=p=0",
      ])
      .arg(format!("data/{}/output.mp4", test_name))
      .output()
      .unwrap();
    let streams = String::from_utf8_lossy(&probe.stdout);
    // The chapter track is also reported as a `data` stream, so check the codec tag instead.
    assert!(
      !streams.contains("tmcd"),
      "timecode stream was kept: {streams}"
    );
    assert!(streams.contains("video") && streams.contains("audio"));
  }

  #[test]
  fn call_merger_without_args() {
    let res = get_output_err(Command::cargo_bin(BIN).unwrap().assert().failure());

    assert!(Regex::new(r"vidmerger(\.exe)? \[OPTIONS] <TARGET_DIR>")
      .unwrap()
      .is_match(&res));
  }

  #[test]
  fn call_merger_and_fail_due_to_not_existing_directory() {
    let res = get_output_err(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/nothing"))
        .assert()
        .failure(),
    );

    assert!(
      res.contains("No such file or directory")
        || res.contains("The system cannot find the path specified")
    );
  }

  #[test]
  fn call_merger_and_skip_hidden_vids() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_hidden_file(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg("--verbose")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    assert!(res.contains(DATA_1_MP4));
    assert!(!res.contains(DATA_3_MP4));
    check_for_merged_file(test_name, "output.mp4");
  }

  #[test]
  fn call_merger_without_ffmpeg() {
    Command::cargo_bin(BIN)
      .unwrap()
      .arg("-y")
      .arg("data")
      .env_clear()
      .assert()
      .failure()
      .stderr(format!(
        "❌ ffmpeg is not available. Please install it first.\n"
      ));
  }

  #[test]
  fn call_merger_against_mp4() {
    let test_name = function_name!().split("::").last().unwrap();
    prep(test_name);

    Command::cargo_bin(BIN)
      .unwrap()
      .arg("-y")
      .args(&["--format", "mp4"])
      .arg(format!("data/{}", test_name))
      .assert()
      .success();
  }

  #[test]
  fn check_for_binaries() {
    if which::which("ffmpeg").is_err() {
      panic!("❌ ffmpeg wasn't found");
    }
  }

  #[test]
  fn call_merger_with_fps_changer() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_different_fps_values(test_name);

    let res = get_output_err(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(!res.contains("Non-monotonous DTS"));
    assert!(get_video_info(&format!("data/{}/output.mp4", test_name)).contains("28 fps"));
    check_for_merged_file(test_name, "output.mp4");
  }

  #[test]
  fn call_merger_with_fps_changer_with_fps_cli_arg() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_different_fps_values(test_name);

    let res = get_output_err(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .args(["--fps", "25"])
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(!res.contains("Non-monotonous DTS"));
    assert!(get_video_info(&format!("data/{}/output.mp4", test_name)).contains("25 fps"));
    check_for_merged_file(test_name, "output.mp4");
  }

  #[test]
  fn call_merger_without_fps_changer_on_vids_with_different_fps_values() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_different_fps_values(test_name);

    get_output_err(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .arg("--skip-fps-changer")
        .assert()
        .success(),
    );

    assert!(Regex::new(r"30\.\d+ fps")
      .unwrap()
      .is_match(&get_video_info(&format!("data/{}/output.mp4", test_name))));
    check_for_merged_file(test_name, "output.mp4");
  }

  #[cfg(unix)]
  #[test]
  fn call_merger_keeps_first_video_first_when_fps_changes() {
    let test_name = function_name!().split("::").last().unwrap();
    let input_dir = std::path::PathBuf::from(format!("data/{test_name}"));
    fs::create_dir_all(&input_dir).unwrap();
    let input_dir = fs::canonicalize(input_dir).unwrap();

    // The bug only appears when the converted clip's temp path sorts after the
    // input path. "data/<name>_tmp" sorts after "data/<name>/" because '_' > '/'.
    // The temp folder is removed when the test ends, even if an assert fails.
    struct RemoveDirOnDrop(std::path::PathBuf);
    impl Drop for RemoveDirOnDrop {
      fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
      }
    }
    let temp_dir = std::path::PathBuf::from(format!("data/{test_name}_tmp"));
    fs::create_dir_all(&temp_dir).unwrap();
    let temp_dir = fs::canonicalize(temp_dir).unwrap();
    let _cleanup = RemoveDirOnDrop(temp_dir.clone());

    let clips = [("green", "30"), ("red", "25")];

    for (index, (color, fps)) in clips.iter().enumerate() {
      let input = input_dir.join(format!("{} - Chapter {}.mp4", index + 1, index + 1));
      let source = format!("color=c={color}:s=160x90:r={fps}:d=1");
      let result = Command::new("ffmpeg")
        .args(["-v", "error", "-f", "lavfi", "-i"])
        .arg(source)
        .args([
          "-c:v",
          "libx264",
          "-preset",
          "ultrafast",
          "-pix_fmt",
          "yuv420p",
          "-an",
          "-y",
        ])
        .arg(input)
        .output()
        .unwrap();
      assert!(result.status.success(), "failed to generate test video");
    }

    Command::cargo_bin(BIN)
      .unwrap()
      .arg("-y")
      .arg(&input_dir)
      .env("TMPDIR", &temp_dir)
      .assert()
      .success();

    let first_frame = Command::new("ffmpeg")
      .args(["-v", "error", "-ss", "0.5", "-i"])
      .arg(input_dir.join("output.mp4"))
      .args([
        "-frames:v",
        "1",
        "-vf",
        "scale=2:2",
        "-pix_fmt",
        "rgb24",
        "-f",
        "rawvideo",
        "pipe:1",
      ])
      .output()
      .unwrap();
    assert!(
      first_frame.status.success(),
      "failed to decode merged video"
    );
    assert!(first_frame.stdout.len() >= 3);
    let (red, green, blue) = (
      first_frame.stdout[0],
      first_frame.stdout[1],
      first_frame.stdout[2],
    );
    assert!(
      green > red.saturating_add(100) && green > blue.saturating_add(100),
      "expected the 30-fps green clip first, got RGB({red}, {green}, {blue})"
    );
  }

  // ----------------------------------------------------------------

  #[test]
  fn call_merger_with_skip_cropper_keeps_black_bars() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_black_bars(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg("--skip-cropper")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    // The cropper is skipped, so it never runs and no crop is printed.
    assert!(!res.contains("✂️"));
    let probe = Command::new("ffprobe")
      .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
      .args(["stream=width,height", "-of", "csv=p=0"])
      .arg(format!("data/{}/output.mp4", test_name))
      .output()
      .unwrap();
    // The bars are kept, so the output is the full 320x240 stored size.
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "320,240");
  }

  #[test]
  fn call_merger_with_skip_cropper_and_mixed_bars_keeps_everything() {
    let test_name = function_name!().split("::").last().unwrap();
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    // Clip 1 has bars. Clip 2 has no bars.
    Command::new("ffmpeg")
      .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
      .arg("color=c=white:s=320x180:d=10:r=25")
      .args([
        "-vf",
        "pad=320:240:0:30:black",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-an",
      ])
      .arg(format!("data/{}/1 - Clip A.mp4", test_name))
      .assert()
      .success();
    Command::new("ffmpeg")
      .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
      .arg("color=c=blue:s=320x240:d=10:r=25")
      .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-an"])
      .arg(format!("data/{}/2 - Clip B.mp4", test_name))
      .assert()
      .success();

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg("--skip-cropper")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    // Nothing is cropped, not even the clip that has bars.
    assert!(!res.contains("✂️"));
    let probe = Command::new("ffprobe")
      .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
      .args(["stream=width,height", "-of", "csv=p=0"])
      .arg(format!("data/{}/output.mp4", test_name))
      .output()
      .unwrap();
    // The merge converts the clips to the first clip's size, so the output keeps the stored
    // size of the barred clip (320x240), which is what skipping the cropper must preserve.
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "320,240");
  }

  #[test]
  fn call_merger_crops_black_bars() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_black_bars(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("✂️"));
    let probe = Command::new("ffprobe")
      .args([
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height",
        "-of",
        "csv=p=0",
      ])
      .arg(format!("data/{}/output.mp4", test_name))
      .output()
      .unwrap();
    // Each clip is 320x240 with 30 px black bars at the top and bottom, so the picture is 320x180.
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "320,180");
  }

  #[test]
  fn call_merger_crops_only_some_inputs_and_merges() {
    let test_name = function_name!().split("::").last().unwrap();
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    // Clip 1 has bars and is cropped. Clip 2 has no bars and is not cropped.
    Command::new("ffmpeg")
      .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
      .arg("color=c=white:s=320x180:d=10:r=25")
      .args([
        "-vf",
        "pad=320:240:0:30:black",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-an",
      ])
      .arg(format!("data/{}/1 - Clip A.mp4", test_name))
      .assert()
      .success();
    Command::new("ffmpeg")
      .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
      .arg("color=c=blue:s=320x180:d=10:r=25")
      .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-an"])
      .arg(format!("data/{}/2 - Clip B.mp4", test_name))
      .assert()
      .success();

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("✂️"));
    let probe = Command::new("ffprobe")
      .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
      .args(["stream=width,height", "-of", "csv=p=0"])
      .arg(format!("data/{}/output.mp4", test_name))
      .output()
      .unwrap();
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "320,180");
  }

  /// Creates two 320x240 `.mp4` clips with 30 px black bars at the top and bottom.
  fn prep_with_black_bars(test_name: &str) {
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    for name in ["1 - Clip A.mp4", "2 - Clip B.mp4"] {
      Command::new("ffmpeg")
        .args([
          "-v",
          "error",
          "-y",
          "-f",
          "lavfi",
          "-i",
          "color=c=white:s=320x180:d=10:r=25",
          "-vf",
          "pad=320:240:0:30:black",
          "-c:v",
          "libx264",
          "-pix_fmt",
          "yuv420p",
          "-an",
        ])
        .arg(format!("data/{}/{}", test_name, name))
        .assert()
        .success();
    }
  }

  fn prep(test_name: &str) {
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    fs::copy(
      format!("data/{}", DATA_1_MP4),
      format!("data/{}/{}", test_name, DATA_1_MP4),
    )
    .unwrap();
    fs::copy(
      format!("data/{}", DATA_2_MP4),
      format!("data/{}/{}", test_name, DATA_2_MP4),
    )
    .unwrap();
  }

  fn prep_with_hidden_file(test_name: &str) {
    prep(test_name);
    std::fs::File::create(format!("data/{}/{}", test_name, DATA_3_MP4)).unwrap();
  }

  fn prep_audio(test_name: &str) {
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    fs::copy(
      format!("data/{}", DATA_1_MP3),
      format!("data/{}/{}", test_name, DATA_1_MP3),
    )
    .unwrap();
    fs::copy(
      format!("data/{}", DATA_2_MP3),
      format!("data/{}/{}", test_name, DATA_2_MP3),
    )
    .unwrap();
  }

  /// Creates two `.mp4`-named inputs whose audio is `pcm_alaw` (issue #17).
  /// Matroska content is used because MP4 itself cannot store `pcm_alaw`.
  fn prep_with_alaw_audio(test_name: &str) {
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    for (name, freq) in [("1 - Clip A.mp4", "440"), ("2 - Clip B.mp4", "880")] {
      Command::new("ffmpeg")
        .args([
          "-v",
          "error",
          "-y",
          "-f",
          "lavfi",
          "-i",
          "testsrc=d=1:s=160x120:r=30",
        ])
        .args(["-f", "lavfi", "-i", &format!("sine=f={freq}:d=1")])
        .args([
          "-c:v",
          "mpeg4",
          "-c:a",
          "pcm_alaw",
          "-shortest",
          "-f",
          "matroska",
        ])
        .arg(format!("data/{}/{}", test_name, name))
        .assert()
        .success();
    }
  }

  /// Creates two `.mp4`-named MOV files that carry a `tmcd` timecode data stream
  /// (issue #53). MP4 output cannot map that stream, so merging needs the fallback.
  fn prep_with_timecode_stream(test_name: &str) {
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    for (name, freq) in [("1 - Clip A.mp4", "440"), ("2 - Clip B.mp4", "880")] {
      Command::new("ffmpeg")
        .args([
          "-v",
          "error",
          "-y",
          "-f",
          "lavfi",
          "-i",
          "testsrc=d=1:s=160x120:r=30",
        ])
        .args(["-f", "lavfi", "-i", &format!("sine=f={freq}:d=1")])
        .args([
          "-c:v",
          "mpeg4",
          "-c:a",
          "aac",
          "-shortest",
          "-timecode",
          "00:00:00:00",
        ])
        .args(["-f", "mov"])
        .arg(format!("data/{}/{}", test_name, name))
        .assert()
        .success();
    }
  }

  fn prep_with_different_fps_values(test_name: &str) {
    fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
    fs::copy(
      format!("data/{}", DATA_1_MP4),
      format!("data/{}/{}", test_name, DATA_1_MP4),
    )
    .unwrap();
    let mut cmd = Command::new("ffmpeg");
    cmd
      .arg("-i")
      .arg(format!("data/{}/{}", test_name, DATA_1_MP4))
      .arg("-r")
      .arg("28")
      .arg(format!("data/{}/2 - Chapter 2.mp4", test_name))
      .output()
      .unwrap();
  }

  fn check_for_merged_file(test_name: &str, merged_file_name: &str) {
    let len = fs::metadata(format!("data/{}/{}", test_name, merged_file_name))
      .unwrap()
      .len();
    assert_greater_than(len, 600000);
    assert_lesser_than(len, 700000);
  }

  fn get_output(assert: Assert) -> String {
    String::from_utf8(assert.get_output().to_owned().stdout).unwrap()
  }

  fn get_output_err(assert: Assert) -> String {
    String::from_utf8(assert.get_output().to_owned().stderr).unwrap()
  }

  fn download(url: &str) {
    let filename = url.split('/').last().unwrap().replace("+", " ");
    let out = format!("data/{}", filename);

    // Only the headers are read here. The body is downloaded only if the cache is not valid.
    let response = ureq::get(url).call().unwrap();
    let expected_len = response
      .headers()
      .get("content-length")
      .and_then(|v| v.to_str().ok())
      .and_then(|v| v.parse::<u64>().ok());

    // Cache hit: a local file with the full size is kept. A partial file from an
    // older run has a different size, so it is downloaded again.
    if let Some(expected_len) = expected_len {
      if fs::metadata(&out).is_ok_and(|m| m.len() == expected_len) {
        println!("♻️  Using cached sample: {out}");
        return;
      }
    }

    // Write to a temporary file first, so an interrupted download is never used as a cache hit.
    let tmp = format!("{out}.part");
    let mut body = response.into_body();
    let mut reader = body.as_reader();
    let mut file = File::create(&tmp).unwrap();
    io::copy(&mut reader, &mut file).unwrap();
    fs::rename(&tmp, &out).unwrap();
  }

  fn get_video_info(file_path: &str) -> String {
    let output = Command::new("ffprobe").arg("-i").arg(file_path).output();
    String::from_utf8_lossy(&output.unwrap().stderr).to_string()
  }
}
