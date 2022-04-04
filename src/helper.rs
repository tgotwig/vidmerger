use core::time;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::thread;

use regex::Regex;

use nanoid::nanoid;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

use crate::config;

pub fn exit_when_ffmpeg_not_available() {
    if cfg!(target_os = "windows") {
        if which::which("ffmpeg.exe").is_err() {
            eprintln!("ffmpeg.exe not found 😬");
            exit(1);
        }
    } else if which::which("ffmpeg").is_err() {
        eprintln!("ffmpeg not found 😬");
        exit(1);
    }
}

pub fn split(string: String) -> Vec<String> {
    let file_formats: Vec<_> = string
        .lines()
        .map(|s| s.trim().split(',').map(String::from).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    file_formats[0].clone()
}

pub fn remove_file(path: &Path) -> Result<()> {
    if Path::new(path).exists() {
        println!("🔥 Removing {}", path.display());
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn get_sorted_paths(input_vids_path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(input_vids_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

pub fn generate_list_of_vids(file_format: &str, paths: &[PathBuf]) -> String {
    let mut list = String::new();
    let re = Regex::new(format!(r"\.{}$", regex::escape(file_format)).as_str()).unwrap();

    let scale = config::get_scale();

    for path in paths {
        if re.is_match(&format!("{}", path.display())) {
            if scale.is_none() {
                if list.chars().count() == 0 {
                    list = format!(
                        "file '{}'",
                        fs::canonicalize(path).unwrap().to_str().unwrap()
                    );
                } else {
                    list = format!(
                        "{}\nfile '{}'",
                        list,
                        fs::canonicalize(path).unwrap().to_str().unwrap()
                    );
                }
            } else if scale.is_some() {
                if list.chars().count() == 0 {
                    list = format!(
                        "file 'scaled_vids/{}'",
                        path.file_name().unwrap().to_str().unwrap()
                    );
                } else {
                    list = format!(
                        "{}\nfile 'scaled_vids/{}'",
                        list,
                        path.file_name().unwrap().to_str().unwrap()
                    );
                }
            }
        }
    }
    list
}

pub fn create_dir(name: &str) {
    if Path::new(name).exists() {
        fs::remove_dir_all(name).unwrap()
    }
    fs::create_dir(name).unwrap()
}

pub fn print_preview(preview: &str) {
    let preview_enabled = config::get_preview();
    println!("\n👇 Order of merging:\n\n{}\n", BrightBlue.paint(&preview));
    if !preview_enabled {
        println!("⏳ Starts after 3 seconds...\n");
        thread::sleep(time::Duration::from_secs(3));
    }
}

pub fn create_tmp_dir() -> PathBuf {
    let dir = temp_dir().join(nanoid!(8));
    fs::create_dir(&dir).unwrap();
    dir
}

pub fn create_list_txt(string: String, mut dir: PathBuf) -> PathBuf {
    dir.push("list.txt");
    File::create(&dir)
        .unwrap()
        .write_all(string.as_bytes())
        .unwrap();
    dir
}
