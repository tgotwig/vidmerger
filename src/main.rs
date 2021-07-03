#![deny(warnings)]

use std::fs::{self, DirEntry, File};
use std::io::Write;
use std::path::Path;
use std::vec::Vec;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

mod cmd;
mod helper;
mod local_args_parser;
mod remote_args_factory;

fn main() -> std::io::Result<()> {
    helper::exit_when_ffmpg_not_available();

    let (dir, formats, preview_enabled) = local_args_parser::fetch();

    for file_format in helper::string_to_vec(formats) {
        // i/o paths
        let input_vids_path = helper::format_path(dir.clone());
        let input_vids_path = Path::new(&input_vids_path);
        let output_list_path = input_vids_path.join("list.txt");
        let output_vid_path = input_vids_path.join(format!("output.{}", file_format));

        remove_previously_generated_video(&output_vid_path);

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

                let ffmpeg_args = remote_args_factory::make(
                    &output_list_path.to_str().unwrap(),
                    output_vid_path.to_str().unwrap().to_string(),
                );

                let child = if cfg!(target_os = "windows") {
                    cmd::merge(true, ffmpeg_args)
                } else {
                    cmd::merge(false, ffmpeg_args)
                };

                let res = child.unwrap().wait_with_output();

                println!("{:?}\n", res);
                if res.unwrap().status.success() {
                    println!("Successfully generated 'output.{}'! 😆🎞", file_format)
                } else {
                    println!("Something went wrong 😖")
                }
                fs::remove_file(output_list_path.to_str().unwrap())?; // remove list.txt
            }
        }
    }

    Ok(())
}

fn remove_previously_generated_video(output_vid_path: &Path) {
    if Path::new(output_vid_path).exists() {
        fs::remove_file(output_vid_path).unwrap();
    }
}
