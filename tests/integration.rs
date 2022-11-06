mod integration {
    use std::fs;

    use assert_cmd::{assert::Assert, Command};
    use k9::assert_greater_than;
    use stdext::function_name;

    static BIN: &'static str = "vidmerger";

    #[cfg(test)]
    #[ctor::ctor]
    fn prepare() {
        use std::fs::{self, File};

        println!("👷 Doing preparations...");

        fs::remove_dir_all("data").unwrap_or_default();
        let yt = check_for_yt_dlp_or_youtube_dl();

        Command::new(yt)
            .args(&[
                "-o",
                "data/1.mp4",
                "-f",
                "22",
                "https://www.youtube.com/watch?v=zGDzdps75ns",
            ])
            .unwrap();
        fs::copy("data/1.mp4", "data/2.mp4").unwrap();
        File::create("data/.3.mp4").unwrap();

        println!("✅ Preparations done!");
    }

    #[test]
    fn calling_vidmerger() {
        let fun_name = function_name!().split("::").last().unwrap();
        prep(fun_name);

        let res = get_output(
            Command::cargo_bin(BIN)
                .unwrap()
                .arg(format!("data/{}", fun_name))
                .assert()
                .success(),
        );

        assert!(res.contains("✅ Successfully generated"));
        check_for_merged_file(fun_name);
    }

    #[test]
    fn calling_vidmerger_without_ffmpeg() {
        // todo: 🐛 fix for windows
        if cfg!(target_os = "windows") {
        } else {
            let mut cmd = Command::cargo_bin(BIN).unwrap();
            cmd.arg("data")
                .env_clear()
                .assert()
                .failure()
                .stderr("ffmpeg not found 😬\n");
        }
    }

    #[test]
    fn calling_vidmerger_in_preview_mode() {
        // todo: don't remove file in preview mode
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        cmd.arg("data").arg("--preview").assert().success();
    }

    #[test]
    fn calling_vidmerger_against_mp4() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        cmd.arg("data")
            .args(&["--format", "mp4"])
            .assert()
            .success();
    }

    #[test]
    fn calling_vidmerger_against_mp4_with_scaling() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        cmd.arg("data")
            .args(&["--format", "mp4"])
            .args(&["--scale", "320:240"])
            .assert()
            .success();
    }

    // ----------------------------------------------------------------

    fn prep(fun_name: &str) {
        fs::create_dir(format!("data/{}", fun_name)).unwrap_or_default();
        fs::copy("data/1.mp4", format!("data/{}/1.mp4", fun_name)).unwrap();
        fs::copy("data/2.mp4", format!("data/{}/2.mp4", fun_name)).unwrap();
    }

    fn check_for_merged_file(fun_name: &str) {
        let len = fs::metadata(format!("data/{}/output.mp4", fun_name))
            .unwrap()
            .len();
        assert_greater_than!(len, 9000);
    }

    fn check_for_yt_dlp_or_youtube_dl() -> &'static str {
        if which::which("yt-dlp").is_ok() {
            "yt-dlp"
        } else if which::which("youtube-dl").is_ok() {
            "youtube-dl"
        } else {
            panic!("Neither yt-dlp nor youtube-dl was found 😬")
        }
    }

    fn get_output(assert: Assert) -> String {
        String::from_utf8(assert.get_output().to_owned().stdout).unwrap()
    }
}
