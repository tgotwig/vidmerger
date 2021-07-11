use std::fs::{self};
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::exit;

use regex::Regex;

use crate::local_args;

pub fn generate_list_of_vids(file_format: &str, paths: &[PathBuf]) -> String {
    let mut list = String::new();
    let re = Regex::new(format!(r"\.{}$", regex::escape(file_format)).as_str()).unwrap();

    let (_, _, _, scale) = local_args::get();

    for path in paths {
        if re.is_match(&format!("{}", path.display())) {
            if scale.is_none() {
                if list.chars().count() == 0 {
                    list = format!("file '{}'", path.file_name().unwrap().to_str().unwrap());
                } else {
                    list = format!(
                        "{}\nfile '{}'",
                        list,
                        path.file_name().unwrap().to_str().unwrap()
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

pub fn get_sorted_paths(input_vids_path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(input_vids_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

pub fn string_to_vec(string: String) -> Vec<String> {
    let file_formats: Vec<_> = string
        .lines()
        .map(|s| s.trim().split(',').map(String::from).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    file_formats[0].clone()
}

pub fn remove_previously_generated_video(output_vid: &Path) -> Result<()> {
    if Path::new(output_vid).exists() {
        println!("🔥 Removing {}", output_vid.display());
        fs::remove_file(output_vid)?;
    }
    Ok(())
}

pub fn exit_when_ffmpg_not_available() {
    if !is_ffmpeg_available() {
        exit(1);
    }
}

fn is_ffmpeg_available() -> bool {
    if cfg!(target_os = "windows") {
        if which::which("ffmpeg.exe").is_err() {
            eprintln!("ffmpeg.exe not found 😬");
            false
        } else {
            true
        }
    } else if which::which("ffmpeg").is_err() {
        eprintln!("ffmpeg not found 😬");
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_ffmpeg_available() {
        assert_eq!(is_ffmpeg_available(), true);
    }
}
