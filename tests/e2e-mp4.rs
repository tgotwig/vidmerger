mod e2e_mp4 {
  use assert_cmd::{assert::Assert, Command};
  use k9::assertions::{greater_than::assert_greater_than, lesser_than::assert_lesser_than};
  use regex::Regex;
  use std::fs;
  use stdext::function_name;

  static BIN: &'static str = "vidmerger";
  static DATA_1_MP4: &'static str = "1 - Chapter 1.mp4";
  static DATA_2_MP4: &'static str = "2 - Chapter 2.mp4";
  static DATA_3_MP4: &'static str = ".3.mp4";

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

  /// Copies the two real `.mp4` samples from `tests/data/mp4`.
  fn prep(test_name: &str) {
    fs::create_dir_all(format!("data/{}", test_name)).unwrap();
    for name in [DATA_1_MP4, DATA_2_MP4] {
      fs::copy(
        format!("tests/data/mp4/{}", name),
        format!("data/{}/{}", test_name, name),
      )
      .unwrap();
    }
  }

  fn prep_with_hidden_file(test_name: &str) {
    prep(test_name);
    fs::File::create(format!("data/{}/{}", test_name, DATA_3_MP4)).unwrap();
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

  /// Clip 1 is the real 29.97 fps sample. Clip 2 is the same clip re-encoded to 28 fps.
  fn prep_with_different_fps_values(test_name: &str) {
    fs::create_dir_all(format!("data/{}", test_name)).unwrap();
    fs::copy(
      format!("tests/data/mp4/{}", DATA_1_MP4),
      format!("data/{}/{}", test_name, DATA_1_MP4),
    )
    .unwrap();
    Command::new("ffmpeg")
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

  fn get_video_info(file_path: &str) -> String {
    let output = Command::new("ffprobe").arg("-i").arg(file_path).output();
    String::from_utf8_lossy(&output.unwrap().stderr).to_string()
  }
}
