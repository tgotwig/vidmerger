#![deny(warnings)]

use core::time;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::thread;
use std::vec::Vec;

use cli::Cli;
use path_slash::PathExt;

mod commanders;
mod cli;
mod ffmpeg_args_factory;
mod helper;

use system_shutdown::shutdown;

use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

fn main() -> Result<(), Error> {
    helper::exit_when_ffmpeg_not_available();
    let matches = Cli::init().get_matches();
    let (dir, formats, preview_enabled, scale, should_shutdown) = (
        matches.value_of("DIR").unwrap().to_string(),
        matches
            .value_of("format")
            .unwrap_or("avchd,avi,flv,mkv,mov,mp4,webm,wmv")
            .to_string(),
        matches.is_present("preview"),
        matches.value_of("scale"),
        matches.is_present("shutdown"),
    );

    for file_format in helper::split(formats) {
        let input_vids = Path::new(&dir);
        let output_vid = input_vids.join(format!("output.{}", file_format));

        helper::remove_file(&output_vid)?;

        let paths: Vec<PathBuf> = helper::get_sorted_paths(input_vids)?;
        let list = helper::generate_list_of_vids(file_format.as_str(), &paths, scale);

        if !list.is_empty() {
            let tmp_dir = helper::create_tmp_dir();

            if scale.is_some() {
                helper::create_dir(tmp_dir.join("scaled_vids").to_str().unwrap());
                commanders::scaler::execute(&file_format, paths, &tmp_dir, &matches);
            }

            println!("\n👇 Order of merging:\n\n{}\n", BrightBlue.paint(&list));
            if !preview_enabled {
                println!("⏳ Starts after 3 seconds...\n");
                thread::sleep(time::Duration::from_secs(3));
            }

            if !preview_enabled {
                let list_txt = helper::create_list_txt(list, tmp_dir);

                let ffmpeg_args = ffmpeg_args_factory::make_merge_args(
                    &list_txt.to_slash().unwrap(),
                    output_vid.to_slash().unwrap().to_string(),
                );

                commanders::merger::merge(ffmpeg_args, file_format);
            }
        }
    }

    if should_shutdown {
        match shutdown() {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    } else {
        Ok(())
    }
}
