use std::path::{Path, PathBuf};

use crate::commanders::_cmd::adjust_fps_by_ffmpeg;

pub fn adjust_fps(file_to_merge: PathBuf, fps_goal: &i8, tmp_dir: &Path) -> PathBuf {
    let file_name = file_to_merge.file_name().unwrap().to_string_lossy();
    let new_file_location = tmp_dir.join(file_name.to_string());

    adjust_fps_by_ffmpeg(file_to_merge, fps_goal, new_file_location)
}
