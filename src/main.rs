#![deny(warnings)]
mod cli;
mod commanders;
mod helpers;
use crate::commanders::fps_changer::change_fps;
use crate::commanders::selector::select;
use crate::helpers::str_helper::create_order_of_merging;
use clap::ArgMatches;
use cli::Cli;
use helpers::io_helper::create;
use helpers::io_helper::create_tmp_dir;
use helpers::io_helper::exit_when_ffmpeg_not_available;
use helpers::io_helper::remove_file;
use helpers::io_helper::wait_for_enter_or_esc_key;
use helpers::str_helper::split;
use lazy_static::lazy_static;
use path_slash::PathExt;
use std::io::Error;
use std::path::Path;
use system_shutdown::shutdown;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

lazy_static! {
  static ref MATCHES: ArgMatches = Cli::init().get_matches();
  static ref VERBOSE: bool = MATCHES.get_flag("verbose");
}

fn main() -> Result<(), Error> {
  let matches = Cli::init().get_matches();
  exit_when_ffmpeg_not_available();

  let target_dir = Path::new(
    matches
      .get_one::<String>("TARGET_DIR")
      .expect("TARGET_DIR is required"),
  );
  let formats = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("3g2,3gp,aac,ac3,alac,amr,ape,au,avi,awb,dts,f4a,f4b,f4p,f4v,flac,flv,m4a,m4b,m4p,m4r,m4v,mkv,mov,mp2,mp3,mp4,mpeg,mpg,oga,ogg,ogm,ogv,ogx,opus,pcm,spx,wav,webm,wma,wmv")
        .to_string();
  let should_shutdown = matches.get_flag("shutdown");
  let skip_fps_changer = matches.get_flag("skip-fps-changer");
  let yes = matches.get_flag("yes");
  let fps_from_cli = matches
    .get_one::<String>("fps")
    .map(|s| s.as_str())
    .unwrap_or("0")
    .parse::<f32>()
    .unwrap();

  for file_format in split(formats) {
    let ffmpeg_output_file = target_dir.join(format!("output.{}", file_format));

    remove_file(&ffmpeg_output_file)?;

    let (files_to_merge, mut files_to_merge_as_strings, mut ffmpeg_input_content) =
      select(&file_format);

    if !ffmpeg_input_content.is_empty() {
      print!("\n📜 Order of merging:\n");
      println!("{}", create_order_of_merging(&ffmpeg_input_content));
      if !yes {
        wait_for_enter_or_esc_key();
      }

      let tmp_dir = create_tmp_dir();

      if !skip_fps_changer {
        (_, files_to_merge_as_strings, ffmpeg_input_content) =
          change_fps(files_to_merge, &tmp_dir, fps_from_cli);
      }

      let ffmpeg_input_file = tmp_dir.join("ffmpeg_input_file.txt");
      create(&ffmpeg_input_file, ffmpeg_input_content);
      println!(
        "🐣 Generated: {} (contains merge order)",
        BrightBlue.paint(ffmpeg_input_file.to_slash().unwrap())
      );

      commanders::merger::merge(
        ffmpeg_input_file.to_slash().unwrap().into_owned(),
        ffmpeg_output_file.to_slash().unwrap().to_string(),
        files_to_merge_as_strings,
        &file_format,
        tmp_dir,
      );
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
