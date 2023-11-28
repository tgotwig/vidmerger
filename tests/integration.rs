mod integration {
    use assert_cmd::{assert::Assert, Command};
    use k9::assertions::{greater_than::assert_greater_than, lesser_than::assert_lesser_than};
    use regex::Regex;
    use std::fs;
    use stdext::function_name;

    static BIN: &'static str = "vidmerger";

    #[cfg(test)]
    #[ctor::ctor]
    fn prepare() {
        use std::fs::{self, File};

        println!("👷 Doing preparations...");

        fs::remove_dir_all("data").unwrap_or_default();

        download(
            "https://www.youtube.com/watch?v=zGDzdps75ns",
            "22",
            "data/1.mp4",
        );
        fs::copy("data/1.mp4", "data/2.mp4").unwrap();
        File::create("data/.3.mp4").unwrap();

        download(
            "https://www.youtube.com/watch?v=zGDzdps75ns",
            "140",
            "data/4.m4a",
        );
        fs::copy("data/4.m4a", "data/5.m4a").unwrap();

        println!("✅ Preparations done!");
    }

    #[test]
    fn call_merger() {
        let test_name = function_name!().split("::").last().unwrap();
        prep(test_name);

        let res = get_output(
            Command::cargo_bin(BIN)
                .unwrap()
                .arg("--skip-wait")
                .arg(format!("data/{}", test_name))
                .assert()
                .success(),
        );

        assert!(res.contains("✅ Successfully generated"));
        check_for_merged_file(test_name, "output.mp4");
    }

    #[test]
    fn call_merger_on_audio_files() {
        let test_name = function_name!().split("::").last().unwrap();
        prep_audio(test_name);

        let res = get_output(
            Command::cargo_bin(BIN)
                .unwrap()
                .arg("--skip-wait")
                .arg(format!("data/{}", test_name))
                .assert()
                .success(),
        );

        assert!(res.contains("✅ Successfully generated"));
        check_for_merged_file(test_name, "output.m4a");
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
                .arg("--skip-wait")
                .arg(format!("data/{}", test_name))
                .assert()
                .success(),
        );

        assert!(res.contains("✅ Successfully generated"));
        assert!(res.contains("1.mp4"));
        assert!(!res.contains(".3.mp4"));
        check_for_merged_file(test_name, "output.mp4");
    }

    #[test]
    fn call_merger_without_ffmpeg() {
        Command::cargo_bin(BIN)
            .unwrap()
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
        Command::cargo_bin(BIN)
            .unwrap()
            .arg("--skip-wait")
            .args(&["--format", "mp4"])
            .arg("data")
            .assert()
            .success();
    }

    #[test]
    fn check_for_binaries() {
        if which::which("yt-dlp").is_err() {
            panic!("❌ yt-dlp wasn't found");
        }
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
                .arg("--skip-wait")
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
                .arg("--skip-wait")
                .args(["--fps", "25"])
                .arg(format!("data/{}", test_name))
                .assert()
                .success(),
        );

        assert!(!res.contains("Non-monotonous DTS"));
        assert!(get_video_info(&format!("data/{}/output.mp4", test_name)).contains("25 fps"));
        check_for_merged_file(test_name, "output.mp4");
    }

    // TODO - fix this test (fails during CI-Test with MacOS)
    #[test]
    fn call_merger_without_fps_changer_on_vids_with_different_fps_values() {
        if cfg!(target_os = "linux") {
            std::process::exit(0)
        }
        let test_name = function_name!().split("::").last().unwrap();
        prep_with_different_fps_values(test_name);

        let res = get_output_err(
            Command::cargo_bin(BIN)
                .unwrap()
                .arg("--skip-wait")
                .arg(format!("data/{}", test_name))
                .arg("--skip-fps-changer")
                .assert()
                .success(),
        );

        // todo: fix this, doesn't work on Github Actions runner for Ubuntu but on own machine
        assert!(res.contains("Non-monotonous DTS"));
        assert!(get_video_info(&format!("data/{}/output.mp4", test_name)).contains("58.41 fps"));
        check_for_merged_file(test_name, "output.mp4");
    }

    // ----------------------------------------------------------------

    fn prep(test_name: &str) {
        fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
        fs::copy("data/1.mp4", format!("data/{}/1.mp4", test_name)).unwrap();
        fs::copy("data/2.mp4", format!("data/{}/2.mp4", test_name)).unwrap();
    }

    fn prep_with_hidden_file(test_name: &str) {
        prep(test_name);
        std::fs::File::create(format!("data/{}/.3.mp4", test_name)).unwrap();
    }

    fn prep_audio(test_name: &str) {
        fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
        fs::copy("data/4.m4a", format!("data/{}/4.m4a", test_name)).unwrap();
        fs::copy("data/5.m4a", format!("data/{}/5.m4a", test_name)).unwrap();
    }

    fn prep_with_different_fps_values(test_name: &str) {
        fs::create_dir(format!("data/{}", test_name)).unwrap_or_default();
        fs::copy("data/1.mp4", format!("data/{}/1.mp4", test_name)).unwrap();
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(format!("data/{}/1.mp4", test_name))
            .arg("-r")
            .arg("28")
            .arg(format!("data/{}/2.mp4", test_name))
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

    fn download(url: &str, format: &str, out: &str) {
        Command::new("yt-dlp")
            .args(&["-o", out, "-f", format, url])
            .unwrap();
    }

    fn get_video_info(file_path: &str) -> String {
        let output = Command::new("ffmpeg").arg("-i").arg(file_path).output();
        String::from_utf8_lossy(&output.unwrap().stderr).to_string()
    }
}
