use std::path::{Path, PathBuf};

use crate::config;

pub fn make_merge_args(output_list_path: &str, output_vid_path: String) -> [String; 10] {
    [
        String::from("-y"),
        String::from("-f"),
        String::from("concat"),
        String::from("-safe"),
        String::from("0"),
        String::from("-i"),
        output_list_path.to_owned(),
        String::from("-c"),
        String::from("copy"),
        output_vid_path,
    ]
}

pub fn make_scale_args(file: &str, tmp_dir: &PathBuf) -> [String; 5] {
    let (dir, scale) = (config::get_dir(), config::get_scale());
    [
        String::from("-i"),
        format!("{}", Path::new(&dir).join(file).display()),
        String::from("-vf"),
        format!("scale={}", scale.unwrap()),
        format!("{}", tmp_dir.join("scaled_vids").join(file).display()),
    ]
}
