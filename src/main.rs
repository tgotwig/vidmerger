#![deny(warnings)]

use core::time;
use std::fs::{self, DirEntry, File};
use std::io::Write;
use std::path::Path;
use std::thread;
use std::vec::Vec;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

mod cmd;
mod helper;
mod local_args;
mod logger;
mod remote_args_factory;

fn main() -> std::io::Result<()> {
    helper::exit_when_ffmpg_not_available();

    let (dir, formats, preview_enabled, scale) = local_args::get();

    for file_format in helper::string_to_vec(formats) {
        let input_vids = Path::new(helper::format_path(&*dir));
        let output_list = input_vids.join("list.txt");
        let output_vid = input_vids.join(format!("output.{}", file_format));

        remove_previously_generated_video(&output_vid);

        let paths: Vec<DirEntry> = helper::get_sorted_paths(&input_vids);
        let list = helper::generate_list_of_vids(file_format.as_str(), &paths);

        if !list.is_empty() {
            if scale.is_some() {
                create_dir("scaled_vids");
            }

            print_preview(&list);

            if !preview_enabled {
                write_list_txt(&output_list, list); // list.txt

                let ffmpeg_args = remote_args_factory::make(
                    &output_list.to_str().unwrap(),
                    output_vid.to_str().unwrap().to_string(),
                );

                let child = cmd::merge(ffmpeg_args);

                logger::print_end_status(child, file_format);
                fs::remove_file(output_list.to_str().unwrap())?; // list.txt
            }
        }
    }
    Ok(())
}

fn remove_previously_generated_video(output_vid: &Path) {
    if Path::new(output_vid).exists() {
        fs::remove_file(output_vid).unwrap();
    }
}

fn write_list_txt(output_list: &Path, list: String) {
    let mut file = File::create(output_list.to_str().unwrap()).unwrap();
    file.write_all(list.as_bytes()).unwrap();
}

fn print_preview(preview: &str) {
    println!("\nOrder of merging 👇\n\n{}\n", BrightBlue.paint(&preview));
    println!("Starts after 3 seconds... ⏳\n");
    thread::sleep(time::Duration::from_secs(3));
}

fn create_dir(name: &'static str) {
    if Path::new(name).exists() {
        fs::remove_dir_all(name).unwrap()
    }
    fs::create_dir(name).unwrap()
}
