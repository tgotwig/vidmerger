use crate::helpers::io_helper::{path_bufs_to_strings, read_dir};
use crate::helpers::str_helper::gen_input_file_content_for_ffmpeg;
use crate::helpers::vec_helper::filter_files;
use std::path::{Path, PathBuf};

pub fn gen_ffmpeg_input_content(target_dir: &Path, file_format: &str) -> String {
    let all_files_on_target_dir: Vec<PathBuf> = read_dir(target_dir).unwrap();
    let files_to_merge = filter_files(all_files_on_target_dir, file_format);
    let files_to_merge = path_bufs_to_strings(files_to_merge);
    gen_input_file_content_for_ffmpeg(files_to_merge)
}
