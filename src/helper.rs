use std::env::temp_dir;

use std::fmt::Write as FmtWrite;
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

pub fn remove_file(path: &Path) -> Result<()> {
    if Path::new(path).exists() {
        println!("🔥 Removing {}", path.display());
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn gen_ffmpeg_input_content(target_dir: &Path, file_format: &str) -> String {
    let all_files_on_target_dir: Vec<PathBuf> = read_dir(target_dir).unwrap();
    let files_to_merge = filter_files(all_files_on_target_dir, file_format);
    let mut ffmpeg_input_content = String::new();

    for file_to_merge in files_to_merge {
        writeln!(
            ffmpeg_input_content,
            "file '{}'",
            canonicalize(file_to_merge).unwrap().display()
        )
        .unwrap();
    }
    ffmpeg_input_content
}

fn read_dir(input_vids_path: &Path) -> Result<Vec<PathBuf>> {
    fs::read_dir(input_vids_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>>>()
}

fn filter_files(all_files: Vec<PathBuf>, file_format: &str) -> Vec<PathBuf> {
    let re = Regex::new(format!(r"[\\/][^.]*\.{}$", regex::escape(file_format)).as_str()).unwrap();
    let mut filtered_files = Vec::new();

    for possible_file_to_merge in all_files {
        if re.is_match(&format!("{}", possible_file_to_merge.display())) {
            filtered_files.push(possible_file_to_merge);
        }
    }
    filtered_files
}

pub fn print_order_of_merging(ffmpeg_input_content: &str) -> String {
    println!("\n👇 Order of merging:\n");
    let file_names_to_be_merged = ffmpeg_input_content
        .lines()
        .map(|line| {
            format!(
                "📄 {}",
                BrightBlue.paint(line.split(['/', '\\']).last().unwrap().replace('\'', ""))
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    println!("{}\n", file_names_to_be_merged); // todo: mock this for unit tests
    file_names_to_be_merged
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_order_of_merging_with_slashes() {
        assert_eq!(
            print_order_of_merging("/target_dir/1.mp4\n/target_dir/2.mp4"),
            "📄 1.mp4\n📄 2.mp4"
        );
    }

    #[test]
    fn test_print_order_of_merging_with_backslashes() {
        assert_eq!(
            print_order_of_merging("C:\\target_dir\\1.mp4\nC:\\target_dir\\2.mp4"),
            "📄 1.mp4\n📄 2.mp4"
        );
    }
}
