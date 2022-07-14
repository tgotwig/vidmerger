use std::{
    io::Error,
    process::{Child, Command, Stdio},
};

static FFMPEG_BINARY_NAME: &str = if cfg!(target_os = "windows") {
    "ffmpeg.exe"
} else {
    "ffmpeg"
};

pub fn merge(args: [String; 10]) -> Result<Child, Error> {
    let cmd = format!("{} {}", FFMPEG_BINARY_NAME, args.join(" "));

    println!("🚀 Calling: '{}'\n", cmd);
    Command::new(FFMPEG_BINARY_NAME)
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
}

pub fn scale(args: [String; 5]) {
    let cmd = format!("{} {}", FFMPEG_BINARY_NAME, args.join(" "));
    println!("🚀 Calling: '{}'", cmd);

    Command::new(FFMPEG_BINARY_NAME)
        .args(&args)
        .output()
        .unwrap();
}
