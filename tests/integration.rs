mod integration {
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

    fs::remove_dir_all("data").unwrap_or_default();
    fs::create_dir_all("data").unwrap();

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
    let test_root = std::path::PathBuf::from(format!("data/{test_name}"));
    let temp_dir = test_root.join("a-temp");
    let input_dir = test_root.join("z-input");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::create_dir_all(&input_dir).unwrap();

    let temp_dir = fs::canonicalize(temp_dir).unwrap();
    let input_dir = fs::canonicalize(input_dir).unwrap();
    let clips = [
      ("red", "25"),
      ("green", "30"),
      ("blue", "30"),
      ("yellow", "30"),
      ("magenta", "30"),
      ("cyan", "30"),
      ("white", "30"),
      ("black", "30"),
    ];

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
      red > green.saturating_add(100) && red > blue.saturating_add(100),
      "expected the 25-fps red clip first, got RGB({red}, {green}, {blue})"
    );
  }

  // ----------------------------------------------------------------

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

    let mut body = ureq::get(url).call().unwrap().into_body();
    let mut reader = body.as_reader();
    let mut file = File::create(out).unwrap();
    io::copy(&mut reader, &mut file).unwrap();
  }

  fn get_video_info(file_path: &str) -> String {
    let output = Command::new("ffprobe").arg("-i").arg(file_path).output();
    String::from_utf8_lossy(&output.unwrap().stderr).to_string()
  }
}
