#![deny(warnings)]

use core::time;
use std::fs::{self, File};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::vec::Vec;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

use path_slash::PathExt;

mod commanders;
mod helper;
mod local_args;
mod remote_args_factory;

fn main() -> Result<()> {
    helper::exit_when_ffmpg_not_available();

    let (dir, formats, preview_enabled, scale) = local_args::get();

    for file_format in helper::string_to_vec(formats) {
        let input_vids = Path::new(dir.as_str());
        let output_list = input_vids.join("list.txt");
        let output_vid = input_vids.join(format!("output.{}", file_format));

        remove_previously_generated_video(&output_vid)?;

        let paths: Vec<PathBuf> = helper::get_sorted_paths(&input_vids)?;
        let list = helper::generate_list_of_vids(file_format.as_str(), &paths);

        if !list.is_empty() {
            if scale.is_some() {
                create_dir(&Path::new(&dir).join("scaled_vids").to_str().unwrap());
                commanders::scaler::execute(&file_format, paths);
            }

            print_preview(&list);

            if !preview_enabled {
                write_list_txt(&output_list, list); // list.txt

                let ffmpeg_args = remote_args_factory::make_merge_args(
                    &output_list.to_slash().unwrap(),
                    output_vid.to_slash().unwrap().to_string(),
                );

                commanders::merger::merge(ffmpeg_args, file_format);
                fs::remove_file(output_list.to_str().unwrap())?; // list.txt
            }
        }
    }
    Ok(())
}

fn remove_previously_generated_video(output_vid: &Path) -> Result<()> {
    if Path::new(output_vid).exists() {
        println!("🔥 Removing {}", output_vid.display());
        fs::remove_file(output_vid)?;
    }
    Ok(())
}

fn write_list_txt(output_list: &Path, list: String) {
    let mut file = File::create(output_list.to_slash().unwrap()).unwrap();
    file.write_all(list.as_bytes()).unwrap();
}

fn print_preview(preview: &str) {
    println!("\n👇 Order of merging:\n\n{}\n", BrightBlue.paint(&preview));
    println!("⏳ Starts after 3 seconds...\n");
    thread::sleep(time::Duration::from_secs(3));
}

fn create_dir(name: &str) {
    if Path::new(name).exists() {
        fs::remove_dir_all(name).unwrap()
    }
    fs::create_dir(name).unwrap()
}
