use crate::commanders::_cmd;
use crate::helpers::str_helper::extract_fps_from_ffmpeg_output;
use std::path::PathBuf;

pub fn get_fps(file_to_merge: &PathBuf) -> i8 {
    let output = _cmd::run_ffmpeg_info_command(file_to_merge).unwrap().stderr;
    let output_str = String::from_utf8_lossy(&output);
    extract_fps_from_ffmpeg_output(output_str)
}
