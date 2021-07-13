use std::path::Path;

use crate::config;

pub fn make_merge_args(output_list_path: &str, output_vid_path: String) -> [String; 8] {
    [
        String::from("-y"),
        String::from("-f"),
        String::from("concat"),
        String::from("-i"),
        output_list_path.to_owned(),
        String::from("-c"),
        String::from("copy"),
        output_vid_path,
    ]
}

pub fn make_scale_args(file: &str) -> [String; 5] {
    let (dir, _, _, scale) = config::get();
    let scale = scale.unwrap();

    let src_vid = format!("{}", Path::new(&dir).join(file).display());
    let tar_scale = format!("{}{}", "scale=", scale);
    let tar_vid = format!(
        "{}",
        Path::new(&dir).join("scaled_vids").join(file).display()
    );

    [
        String::from("-i"),
        src_vid,
        String::from("-vf"),
        tar_scale,
        tar_vid,
    ]
}
