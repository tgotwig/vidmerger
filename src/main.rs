#![deny(warnings)]
use std::fs::{self, DirEntry, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::{exit, Command, Stdio};
use std::vec::Vec;

use clap::{load_yaml, App, AppSettings, ArgMatches};
use regex::Regex;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

fn main() -> std::io::Result<()> {
    if !is_ffmpeg_available() {
        exit(1);
    }

    // fetch arguments
    let matches: ArgMatches = App::from(load_yaml!("cli.yaml"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    // creates a vector with the passed file formats or default ones
    let format_args: &str = match matches.value_of("format") {
        Some(x) => x,
        None => "avchd,avi,flv,mkv,mov,mp4,webm,wmv",
    };
    let file_formats: Vec<_> = String::from(format_args)
        .lines()
        .map(|s| s.trim().split(',').map(String::from).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let file_formats: Vec<String> = file_formats[0].clone();

    for file_format in file_formats {
        // i/o paths
        let input_vids_path = format_path(matches.value_of("DIR").unwrap());
        let input_vids_path = Path::new(&input_vids_path);
        let output_list_path = input_vids_path.join("list.txt");
        let output_vid_path = input_vids_path.join(format!("output.{}", file_format));

        // remove merged video from the last run
        if Path::new(&output_vid_path).exists() {
            fs::remove_file(&output_vid_path)?;
        }

        let paths: Vec<DirEntry> = get_sorted_paths(&input_vids_path);

        let list = generate_list_of_vids(file_format.as_str(), paths);

        if !list.is_empty() {
            // print order in blue
            println!("\nOrder of merging 👇\n");
            println!("{}\n", BrightBlue.paint(&list));

            // only continue if the preview flag isn't set
            if !matches.is_present("preview") {
                // write list.txt
                let mut file = File::create(output_list_path.to_str().unwrap())?;
                file.write_all(list.as_bytes())?;

                let ffmpeg_args = [
                    "-y",
                    "-f",
                    "concat",
                    "-i",
                    output_list_path.to_str().unwrap(),
                    "-c",
                    "copy",
                    output_vid_path.to_str().unwrap(),
                ];

                // generate and write the merged video by ffmpeg
                let mut child = if cfg!(target_os = "windows") {
                    let cmd = format!("ffmpeg.exe {}", ffmpeg_args.join(" "));
                    println!("Calling: '{}' 🚀\n", cmd);

                    Command::new("ffmpeg.exe")
                        .args(&ffmpeg_args)
                        .stdout(Stdio::piped())
                        .spawn()?
                } else {
                    let cmd = format!("ffmpeg {}", ffmpeg_args.join(" "));
                    println!("Calling: '{}' 🚀\n", cmd);

                    // todo: make it work like the code-block below
                    Command::new("ffmpeg")
                        .args(&ffmpeg_args)
                        .stdout(Stdio::piped())
                        .spawn()?

                    // Command::new("ping")
                    //     .args(&["-c", "3", "google.com"])
                    //     .stdout(Stdio::piped())
                    //     .spawn()?
                };

                match child.try_wait() {
                    Ok(Some(status)) => println!("{}", status),
                    Ok(None) => {
                        let res = child.wait_with_output();
                        println!("{:?}\n", res);
                        if res.unwrap().status.success() {
                            println!("Successfully generated 'output.{}'! 😆🎞", file_format)
                        } else {
                            println!("Something went wrong 😖")
                        }
                        // remove list.txt
                        fs::remove_file(output_list_path.to_str().unwrap())?;
                    }
                    Err(e) => println!("{}", e),
                }
            }
        }
    }

    Ok(())
}

fn format_path(path_to_vids: &str) -> String {
    let path_to_vids: String = if path_to_vids.starts_with('\\') {
        path_to_vids.replacen("\\", "", 1)
    } else {
        path_to_vids.into()
    };

    let path_to_vids: String = if !path_to_vids.ends_with('/') && !path_to_vids.ends_with('\\') {
        format!("{}/", path_to_vids)
    } else {
        path_to_vids
    };

    path_to_vids.replace("\\", "/")
}

fn generate_list_of_vids(file_format: &str, paths: Vec<std::fs::DirEntry>) -> String {
    let mut list = String::new();
    let re = Regex::new(format!(r"\.{}$", regex::escape(file_format)).as_str()).unwrap();
    for path in paths {
        let path = path.path();
        if re.is_match(&format!("{}", path.display())) {
            if list.chars().count() == 0 {
                list = format!("file '{}'", path.file_name().unwrap().to_str().unwrap());
            } else {
                list = format!(
                    "{}\nfile '{}'",
                    list,
                    path.file_name().unwrap().to_str().unwrap()
                );
            }
        }
    }
    list
}

fn get_sorted_paths(input_vids_path: &Path) -> Vec<DirEntry> {
    let mut paths: Vec<_> = fs::read_dir(input_vids_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|input_vids_path| input_vids_path.path());
    paths
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

// --------------------
// tests
// --------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_path() {
        assert_eq!(
            format_path(&String::from("c:\\path\\to\\vids")),
            "c:/path/to/vids/"
        );
        assert_eq!(
            format_path(&String::from("\\path\\to\\vids")),
            "path/to/vids/"
        );
        assert_eq!(
            format_path(&String::from("\\path\\to\\vids\\")),
            "path/to/vids/"
        );
        assert_eq!(format_path(&String::from("path/to/vids")), "path/to/vids/");
        assert_eq!(format_path(&String::from("path/to/vids/")), "path/to/vids/");
    }

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
