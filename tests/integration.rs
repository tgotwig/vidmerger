mod integration {
    use assert_cmd::Command;

    static BIN: &'static str = "vidmerger";

    #[test]
    fn calling_vidmerger() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        cmd.arg("data").assert().success();
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
}
