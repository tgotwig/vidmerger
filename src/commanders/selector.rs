use crate::cli::Cli;
use crate::helpers::io_helper::path_bufs_to_sorted_strings;
use crate::helpers::io_helper::read_dir;
use crate::helpers::str_helper::gen_input_file_content_for_ffmpeg;
use crate::helpers::vec_helper::filter_files;
use clap::ArgMatches;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};

lazy_static! {
  static ref MATCHES: ArgMatches = Cli::init().get_matches();
  static ref TARGET_DIR: &'static Path =
    Path::new(MATCHES.get_one::<String>("TARGET_DIR").unwrap());
}

pub fn select(file_format: &str) -> (Vec<PathBuf>, Vec<String>, String) {
  let all_files_on_target_dir = read_dir(*TARGET_DIR).unwrap();
  let files_to_merge = filter_files(all_files_on_target_dir, file_format);
  let files_to_merge_as_strings = path_bufs_to_sorted_strings(&files_to_merge);
  let ffmpeg_input_content = gen_input_file_content_for_ffmpeg(&files_to_merge_as_strings);

  (
    files_to_merge,
    files_to_merge_as_strings,
    ffmpeg_input_content,
  )
}
