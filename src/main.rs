use std::env;
use std::process::Command;
use std::process::exit;
use std::path::PathBuf;
use std::path::Path;
use std::fs::{self, File};
use std::io::prelude::*;

use regex::Regex;
use clap::{App, load_yaml};

fn main() -> std::io::Result<()> {
    if find_it("ffmpeg").is_some() || find_it("ffmpeg.exe").is_some() {
        ()
    } else {
        eprintln!("ffmpeg not found 😬");
        exit(1);
    }

    let matches = App::from(load_yaml!("cli.yaml")).get_matches();
    let file_format = matches.value_of("format").unwrap();
    
    let dir = if matches.value_of("DIR").unwrap() == "." {
        "./"
    } else {
        matches.value_of("DIR").unwrap()
    };

    if Path::new(&format!("output.{}", file_format)).exists() {
        fs::remove_file(format!("output.{}", file_format))?;
    }

    // get sorted paths
    let mut paths: Vec<_> = fs::read_dir(dir).unwrap()
                                              .map(|r| r.unwrap())
                                              .collect();
    paths.sort_by_key(|dir| dir.path());

    // Generate content for input.txt
    let mut input_txt = String::new();
    let re = Regex::new(
        format!(r"\.{}$", regex::escape(file_format)).as_str()).unwrap();
    for path in paths {
        let my_path = path.path();
        if re.is_match(&format!("{}", my_path.display())) {
            if input_txt.chars().count() == 0 {
                input_txt = format!("file '{}'", my_path.display().to_string().replace(dir, ""));
            } else {
                input_txt = format!("{}\nfile '{}'", input_txt, my_path.display().to_string().replace(dir, ""));
            }
        }
    }

    // write input.txt
    let mut file = File::create(&format!("{}input.txt", dir))?;
    file.write_all(input_txt.as_bytes())?;

    // generate and write the merged video by ffmpeg
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(format!(
                "ffmpeg.exe -y -f concat -i {dir}input.txt -c copy {dir}output.{file_format}", file_format=file_format, dir=dir))
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!(
                "ffmpeg -y -f concat -i {dir}input.txt -c copy {dir}output.{file_format}", file_format=file_format, dir=dir))
            .output()
            .expect("failed to execute process")
    };

    if output.status.success() {
        println!("Successfully generated 'output.{}'! 😆🎞", file_format);
    } else {
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("Something went wrong 😖");
    }

    Ok(())
}

fn find_it<P>(exe_name: P) -> Option<PathBuf>
    where P: AsRef<Path>,
{
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).filter_map(|dir| {
            let full_path = dir.join(&exe_name);
            if full_path.is_file() {
                Some(full_path)
            } else {
                None
            }
        }).next()
    })
}
