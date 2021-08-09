mod integration {
    use assert_cmd::Command;

    static BIN: &'static str = "vidmerger";

    #[test]
    fn calling_vidmerger() {
        let mut cmd = Command::cargo_bin(BIN).unwrap();
        cmd.arg("data").assert().success();
    }
}
