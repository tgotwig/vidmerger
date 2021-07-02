#![deny(warnings)]

use std::fs::{self, DirEntry, File};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command, Stdio};
use std::vec::Vec;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

mod args_parser;
mod helper;

fn main() -> std::io::Result<()> {
    if !helper::is_ffmpeg_available() {
        exit(1);
    }

    let (dir, format, preview_enabled) = args_parser::fetch();

    // creates a vector with the passed file formats or default ones
    let file_formats: Vec<_> = format
        .lines()
        .map(|s| s.trim().split(',').map(String::from).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let file_formats: Vec<String> = file_formats[0].clone();

    for file_format in file_formats {
        // i/o paths
        let input_vids_path = helper::format_path(dir.clone());
        let input_vids_path = Path::new(&input_vids_path);
        let output_list_path = input_vids_path.join("list.txt");
        let output_vid_path = input_vids_path.join(format!("output.{}", file_format));

        // remove merged video from the last run
        if Path::new(&output_vid_path).exists() {
            fs::remove_file(&output_vid_path)?;
        }

        let paths: Vec<DirEntry> = helper::get_sorted_paths(&input_vids_path);

        let list = helper::generate_list_of_vids(file_format.as_str(), paths);

        if !list.is_empty() {
            // print order in blue
            println!("\nOrder of merging 👇\n");
            println!("{}\n", BrightBlue.paint(&list));

            // only continue if the preview flag isn't set
            if !preview_enabled {
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
