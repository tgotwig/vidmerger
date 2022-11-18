pub fn make_ffmpeg_merge_args(output_list_path: &str, output_vid_path: String) -> [String; 10] {
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
