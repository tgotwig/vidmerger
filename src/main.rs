#![deny(warnings)]
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::process::Command;

use clap::{load_yaml, App, AppSettings};
use regex::Regex;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

fn main() -> std::io::Result<()> {
    if !is_ffmpeg_available() {
        exit(1);
    }

    // fetch arguments
    let matches = App::from(load_yaml!("cli.yaml"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();
    let file_format = matches.value_of("format").unwrap();

    // i/o paths
    let input_dir = format_path(matches.value_of("DIR").unwrap());
    let input_dir = Path::new(&input_dir);
    let output_list = input_dir.join("input.txt");
    let output_vid = input_dir.join(format!("output.{}", file_format));

    // remove merged video from the last run
    if Path::new(&output_vid).exists() {
        fs::remove_file(&output_vid)?;
    }

    // get sorted paths
    let mut paths: Vec<_> = fs::read_dir(input_dir)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|input_dir| input_dir.path());

    // Generate content for input.txt
    let mut input_txt = String::new();
    let re = Regex::new(format!(r"\.{}$", regex::escape(file_format)).as_str()).unwrap();
    for path in paths {
        let path = path.path();
        if re.is_match(&format!("{}", path.display())) {
            if input_txt.chars().count() == 0 {
                input_txt = format!("file '{}'", path.file_name().unwrap().to_str().unwrap());
            } else {
                input_txt = format!(
                    "{}\nfile '{}'",
                    input_txt,
                    path.file_name().unwrap().to_str().unwrap()
                );
            }
        }
    }

    // print order in blue
    println!("\nOrder of merging 👇\n");
    println!("{}\n", BrightBlue.paint(&input_txt));

    // write input.txt
    let mut file = File::create(output_list.to_str().unwrap())?;
    file.write_all(input_txt.as_bytes())?;

    // generate and write the merged video by ffmpeg
    let output = if cfg!(target_os = "windows") {
        let cmd = format!(
            "ffmpeg.exe -y -f concat -i {format} -c copy {dir}",
            dir = output_vid.to_str().unwrap(),
            format = output_list.to_str().unwrap()
        );
        println!("Calling: '{}' 🚀\n", cmd);

        Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    } else {
        let cmd = format!(
            "ffmpeg -y -f concat -i {format} -c copy {dir}",
            dir = output_vid.to_str().unwrap(),
            format = output_list.to_str().unwrap()
        );
        println!("Calling: '{}' 🚀\n", cmd);

        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    };

    // remove input.txt
    fs::remove_file(output_list.to_str().unwrap())?;

    if output.status.success() {
        println!("Successfully generated 'output.{}'! 😆🎞", file_format);
    } else {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("Something went wrong 😖");
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ffmpeg_available() {
        assert_eq!(is_ffmpeg_available(), true);
    }

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
}
