use std::{
    io::Error,
    process::{Child, Command, Stdio},
};

pub fn merge(args: [String; 10]) -> Result<Child, Error> {
    let cmd = format!("ffmpeg {}", args.join(" "));

    println!("🚀 Calling: '{}'\n", cmd);
    Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
}

pub fn scale(args: [String; 5]) {
    let cmd = format!("ffmpeg {}", args.join(" "));
    println!("🚀 Calling: '{}'", cmd);

    Command::new("ffmpeg").args(&args).output().unwrap();
}
