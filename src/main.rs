#![deny(warnings)]
use cli::Cli;
use core::time;
use helpers::io_helper::path_bufs_to_strings;
use helpers::io_helper::read_dir;
use helpers::str_helper::gen_input_file_content_for_ffmpeg;
use helpers::vec_helper::filter_files;
use path_slash::PathExt;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
mod cli;
mod commanders;
mod ffmpeg_args_factory;
mod helpers;
mod logger;
use crate::logger::print_order_of_merging;
use helpers::io_helper::create;
use helpers::io_helper::create_tmp_dir;
use helpers::io_helper::exit_when_ffmpeg_not_available;
use helpers::io_helper::remove_file;
use helpers::str_helper::split;
use system_shutdown::shutdown;

fn main() -> Result<(), Error> {
    exit_when_ffmpeg_not_available();
    let matches = Cli::init().get_matches();
    let target_dir = Path::new(matches.value_of("TARGET_DIR").unwrap());
    let formats = matches
        .value_of("format")
        .unwrap_or("3g2,3gp,aac,ac3,alac,amr,ape,au,avi,awb,dts,f4a,f4b,f4p,f4v,flac,flv,m4a,m4b,m4p,m4r,m4v,mkv,mov,mp2,mp3,mp4,mpeg,mpg,oga,ogg,ogm,ogv,ogx,opus,pcm,spx,wav,webm,wma,wmv")
        .to_string();
    let should_shutdown = matches.is_present("shutdown");

    for file_format in split(formats) {
        let ffmpeg_output_file = target_dir.join(format!("output.{}", file_format));

        remove_file(&ffmpeg_output_file)?;

        let ffmpeg_input_content =
            gen_input_file_content_for_ffmpeg_from_dir(target_dir, file_format.as_str());

        if !ffmpeg_input_content.is_empty() {
            print_order_of_merging(&ffmpeg_input_content);
            println!("⏳ Starts after 3 seconds...\n");
            thread::sleep(time::Duration::from_secs(3));

            let ffmpeg_input_file = create_tmp_dir().join("ffmpeg_input_file.txt");
            create(&ffmpeg_input_file, ffmpeg_input_content);

            let ffmpeg_merge_args = ffmpeg_args_factory::make_ffmpeg_merge_args(
                &ffmpeg_input_file.to_slash().unwrap(),
                ffmpeg_output_file.to_slash().unwrap().to_string(),
            );

            commanders::merger::merge(ffmpeg_merge_args, file_format);
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

pub fn gen_input_file_content_for_ffmpeg_from_dir(target_dir: &Path, file_format: &str) -> String {
    let all_files_on_target_dir: Vec<PathBuf> = read_dir(target_dir).unwrap();
    let files_to_merge = filter_files(all_files_on_target_dir, file_format);
    let files_to_merge = path_bufs_to_strings(files_to_merge);
    gen_input_file_content_for_ffmpeg(files_to_merge)
}
