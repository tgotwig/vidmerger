use std::env::temp_dir;
use std::fs::{self, canonicalize, File};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use regex::Regex;

use nanoid::nanoid;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

pub fn exit_when_ffmpeg_not_available() {
    if which::which("ffmpeg").is_err() {
        eprintln!("❌ ffmpeg is not available. Please install it first.");
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

pub fn gen_ffmpeg_input_content(target_dir: &Path, file_format: &str) -> String {
    let possible_files_to_merge: Vec<PathBuf> = get_sorted_paths(target_dir).unwrap();
    let re = Regex::new(format!(r"[\\/][^.]*\.{}$", regex::escape(file_format)).as_str()).unwrap();
    let mut ffmpeg_input_content = String::new();

    for possible_file_to_merge in possible_files_to_merge {
        if re.is_match(&format!("{}", possible_file_to_merge.display())) {
            ffmpeg_input_content = if ffmpeg_input_content.chars().count() == 0 {
                format!(
                    "file '{}'",
                    canonicalize(possible_file_to_merge).unwrap().display()
                )
            } else {
                format!(
                    "{}\nfile '{}'",
                    ffmpeg_input_content,
                    canonicalize(possible_file_to_merge).unwrap().display()
                )
            }
        }
    }
    ffmpeg_input_content
}

fn get_sorted_paths(input_vids_path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(input_vids_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

pub fn print_order_of_merging(ffmpeg_input_content: &str) {
    println!("\n👇 Order of merging:\n\n");
    for line in ffmpeg_input_content.lines() {
        println!(
            "📄 {}",
            BrightBlue.paint(line.split(['/', '\\']).last().unwrap().replace('\'', ""))
        );
    }
    println!();
}

pub fn create_tmp_dir() -> PathBuf {
    let dir = temp_dir().join(nanoid!(8));
    fs::create_dir(&dir).unwrap();
    dir
}

pub fn gen_ffmpeg_input_file(string: String, mut dir: PathBuf) -> PathBuf {
    dir.push("ffmpeg_input_file.txt");
    File::create(&dir)
        .unwrap()
        .write_all(string.as_bytes())
        .unwrap();
    dir
}
