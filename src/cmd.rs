use std::process::{Child, Command, Stdio};

pub fn merge(is_windows: bool, args: [String; 8]) -> Result<Child, std::io::Error> {
    let ffmpeg_binary = if is_windows { "ffmpeg.exe" } else { "ffmpeg" };
    let cmd = format!("ffmpeg.exe {}", args.join(" "));

    println!("Calling: '{}' 🚀\n", cmd);
    Command::new(ffmpeg_binary)
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
}
