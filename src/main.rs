#![deny(warnings)]
mod cli;
mod commanders;
mod helpers;
use crate::commanders::fps_changer::change_fps;
use crate::commanders::selector::select;
use crate::helpers::str_helper::create_order_of_merging;
use cli::Cli;
use core::time;
use helpers::io_helper::create;
use helpers::io_helper::create_tmp_dir;
use helpers::io_helper::exit_when_ffmpeg_not_available;
use helpers::io_helper::remove_file;
use helpers::str_helper::split;
use path_slash::PathExt;
use std::io::Error;
use std::path::Path;
use std::thread;
use system_shutdown::shutdown;

fn main() -> Result<(), Error> {
    let matches = Cli::init().get_matches();
    exit_when_ffmpeg_not_available();

    let target_dir = Path::new(matches.value_of("TARGET_DIR").unwrap());
    let formats = matches
        .value_of("format")
        .unwrap_or("3g2,3gp,aac,ac3,alac,amr,ape,au,avi,awb,dts,f4a,f4b,f4p,f4v,flac,flv,m4a,m4b,m4p,m4r,m4v,mkv,mov,mp2,mp3,mp4,mpeg,mpg,oga,ogg,ogm,ogv,ogx,opus,pcm,spx,wav,webm,wma,wmv")
        .to_string();
    let should_shutdown = matches.is_present("shutdown");
    let skip_fps_changer = matches.is_present("skip-fps-changer");
    let skip_chapterer = matches.is_present("skip-chapterer");
    let skip_wait = matches.is_present("skip-wait");
    let fps_from_cli = matches
        .value_of("fps")
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap();

    for file_format in split(formats) {
        let ffmpeg_output_file = target_dir.join(format!("output.{}", file_format));

        remove_file(&ffmpeg_output_file)?;

        let (files_to_merge, mut files_to_merge_as_strings, mut ffmpeg_input_content) =
            select(target_dir, &file_format);

        if !ffmpeg_input_content.is_empty() {
            println!("\n----------------------------------------------------------------");
            println!("📜 Order of merging:\n");
            println!("{}\n", create_order_of_merging(&ffmpeg_input_content));
            if !skip_wait {
                println!("\n⏳ Waiting 3 seconds to read");
                thread::sleep(time::Duration::from_secs(3));
            }

            let tmp_dir = create_tmp_dir();

            if !skip_fps_changer {
                (_, files_to_merge_as_strings, ffmpeg_input_content) =
                    change_fps(files_to_merge, &tmp_dir, fps_from_cli);
            }

            let ffmpeg_input_file = tmp_dir.join("ffmpeg_input_file.txt");
            create(&ffmpeg_input_file, ffmpeg_input_content);

            commanders::merger::merge(
                ffmpeg_input_file.to_slash().unwrap(),
                ffmpeg_output_file.to_slash().unwrap().to_string(),
                &file_format,
            );

            if !skip_chapterer {
                commanders::chapterer::execute(
                    files_to_merge_as_strings,
                    tmp_dir,
                    ffmpeg_output_file,
                    &file_format,
                );
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
