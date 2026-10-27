mod e2e_mov {
  use assert_cmd::{assert::Assert, Command};
  use std::fs;
  use stdext::function_name;

  static BIN: &'static str = "vidmerger";
  static DATA_1_MOV: &'static str = "1 - Chapter 1.mov";
  static DATA_2_MOV: &'static str = "2 - Chapter 2.mov";
  static DATA_3_MOV: &'static str = ".3.mov";

  #[test]
  fn call_merger_with_mov_files() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_mov(test_name);

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
        "stream=codec_type",
        "-of",
        "csv=p=0",
      ])
      .arg(format!("data/{}/output.mov", test_name))
      .output()
      .unwrap();
    let streams = String::from_utf8_lossy(&probe.stdout);
    assert!(streams.contains("video") && streams.contains("audio"));
  }

  #[test]
  fn call_merger_mov() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_mov(test_name);

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .args(["--format", "mov"])
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    assert!(fs::metadata(format!("data/{}/output.mov", test_name)).is_ok());
  }

  #[test]
  fn call_merger_and_skip_hidden_vids_mov() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_mov(test_name);
    fs::File::create(format!("data/{}/{}", test_name, DATA_3_MOV)).unwrap();

    let res = get_output(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .arg("--verbose")
        .args(["--format", "mov"])
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    assert!(res.contains("🐣 Generated"));
    assert!(res.contains(DATA_1_MOV));
    assert!(!res.contains(DATA_3_MOV));
    assert!(fs::metadata(format!("data/{}/output.mov", test_name)).is_ok());
  }

  #[test]
  fn call_merger_with_fps_changer_mov() {
    let test_name = function_name!().split("::").last().unwrap();
    prep_with_mov(test_name);

    let res = get_output_err(
      Command::cargo_bin(BIN)
        .unwrap()
        .arg("-y")
        .args(["--format", "mov"])
        .arg(format!("data/{}", test_name))
        .assert()
        .success(),
    );

    // The 30 fps clip is scaled down to 29.99 fps, the lowest rate in the inputs.
    assert!(!res.contains("Non-monotonous DTS"));
    let probe = Command::new("ffprobe")
      .args(["-v", "error", "-select_streams", "v:0"])
      .args(["-show_entries", "stream=r_frame_rate", "-of", "csv=p=0"])
      .arg(format!("data/{}/output.mov", test_name))
      .output()
      .unwrap();
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "2999/100");
  }

  /// Copies the two real `.mov` recordings from `tests/data/mov` (their 30 and 29.99 fps differ).
  fn prep_with_mov(test_name: &str) {
    fs::create_dir_all(format!("data/{}", test_name)).unwrap();
    for name in [DATA_1_MOV, DATA_2_MOV] {
      fs::copy(
        format!("tests/data/mov/{}", name),
        format!("data/{}/{}", test_name, name),
      )
      .unwrap();
    }
  }

  fn get_output(assert: Assert) -> String {
    String::from_utf8(assert.get_output().to_owned().stdout).unwrap()
  }

  fn get_output_err(assert: Assert) -> String {
    String::from_utf8(assert.get_output().to_owned().stderr).unwrap()
  }
}
