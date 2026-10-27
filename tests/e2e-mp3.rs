mod e2e_mp3 {
  use assert_cmd::{assert::Assert, Command};
  use k9::assertions::{greater_than::assert_greater_than, lesser_than::assert_lesser_than};
  use std::fs;
  use stdext::function_name;

  static BIN: &'static str = "vidmerger";
  static DATA_1_MP3: &'static str = "1 - Chapter 1.mp3";
  static DATA_2_MP3: &'static str = "2 - Chapter 2.mp3";

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

  /// Copies the two real `.mp3` samples from `tests/data`.
  fn prep_audio(test_name: &str) {
    fs::create_dir_all(format!("data/{}", test_name)).unwrap();
    for name in [DATA_1_MP3, DATA_2_MP3] {
      fs::copy(
        format!("tests/data/{}", name),
        format!("data/{}/{}", test_name, name),
      )
      .unwrap();
    }
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
}
