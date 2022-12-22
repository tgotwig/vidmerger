use std::fs;
use std::io::Result;
use std::path::Path;
use std::process::exit;

pub fn exit_when_ffmpeg_not_available() {
    if which::which("ffmpeg").is_err() {
        eprintln!("❌ ffmpeg is not available. Please install it first.");
        exit(1);
    }
}

pub fn remove_file(path: &Path) -> Result<()> {
    if Path::new(path).exists() {
        println!("🔥 Removing {}", path.display());
        fs::remove_file(path)?;
    }
    Ok(())
}
