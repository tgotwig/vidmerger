use nanoid::nanoid;
use std::env::temp_dir;
use std::fs::{self, canonicalize, File};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
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

pub fn read_dir(input_vids_path: &Path) -> Result<Vec<PathBuf>> {
    fs::read_dir(input_vids_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>>>()
}

pub fn create_tmp_dir() -> PathBuf {
    let dir = temp_dir().join(nanoid!(8));
    fs::create_dir(&dir).unwrap();
    dir
}

pub fn create(path: &PathBuf, buf: String) -> &PathBuf {
    File::create(&path)
        .unwrap()
        .write_all(buf.as_bytes())
        .unwrap();
    path
}

pub fn path_bufs_to_strings(path_bufs: Vec<PathBuf>) -> Vec<String> {
    path_bufs
        .iter()
        .map(|path_buf| {
            canonicalize(path_buf.to_str().unwrap())
                .unwrap()
                .display()
                .to_string()
        })
        .collect()
}
