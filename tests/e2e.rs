mod e2e {
  use assert_cmd::{assert::Assert, Command};
  use regex::Regex;
  use std::fs;
  use stdext::function_name;

  static BIN: &'static str = "vidmerger";

  #[cfg(test)]
  #[ctor::ctor(unsafe)]
  fn prepare() {
    use std::fs;

    println!("👷 Doing preparations...");

    // Only remove the test folders inside `data`.
    fs::create_dir_all("data").unwrap();
    for entry in fs::read_dir("data").unwrap() {
      let path = entry.unwrap().path();
      if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
      }
    }

    println!("✅ Preparations done!");
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
  fn check_for_binaries() {
    if which::which("ffmpeg").is_err() {
      panic!("❌ ffmpeg wasn't found");
    }
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

  fn get_output(assert: Assert) -> String {
    String::from_utf8(assert.get_output().to_owned().stdout).unwrap()
  }

  fn get_output_err(assert: Assert) -> String {
    String::from_utf8(assert.get_output().to_owned().stderr).unwrap()
  }
}
