use std::fs::{self, DirEntry};
use std::path::Path;
use std::process::exit;

use regex::Regex;

use crate::local_args;

pub fn generate_list_of_vids(file_format: &str, paths: &[std::fs::DirEntry]) -> String {
    let mut list = String::new();
    let re = Regex::new(format!(r"\.{}$", regex::escape(file_format)).as_str()).unwrap();

    let (_, _, _, scale) = local_args::get();

    for path in paths {
        let path = path.path();
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

pub fn get_sorted_paths(input_vids_path: &Path) -> Vec<DirEntry> {
    let mut paths: Vec<_> = fs::read_dir(input_vids_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|input_vids_path| input_vids_path.path());
    paths
}

pub fn string_to_vec(string: String) -> Vec<String> {
    let file_formats: Vec<_> = string
        .lines()
        .map(|s| s.trim().split(',').map(String::from).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    file_formats[0].clone()
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
    use std::fs::File;

    use super::*;

    #[test]
    fn test_get_sorted_paths() {
        if cfg!(target_os = "macos") {
            fs::create_dir("test").unwrap();
            File::create("test/4").unwrap();
            File::create("test/3").unwrap();

            let paths: Vec<_> = fs::read_dir("test").unwrap().map(|r| r.unwrap()).collect();
            assert_eq!(
                format!("{:?}", paths),
                "[DirEntry(\"test/4\"), DirEntry(\"test/3\")]"
            );
            assert_eq!(
                format!("{:?}", get_sorted_paths(Path::new("test"))),
                "[DirEntry(\"test/3\"), DirEntry(\"test/4\")]"
            );
            fs::remove_dir_all("test").unwrap();
        }
    }

    #[test]
    fn test_is_ffmpeg_available() {
        assert_eq!(is_ffmpeg_available(), true);
    }
}
