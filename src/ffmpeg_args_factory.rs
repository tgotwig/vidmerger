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
    let (dir, _, _, scale, _) = config::get();
    [
        String::from("-i"),
        format!("{}", Path::new(&dir).join(file).display()),
        String::from("-vf"),
        format!("scale={}", scale.unwrap()),
        format!(
            "{}",
            Path::new(&dir).join("scaled_vids").join(file).display()
        ),
    ]
}
