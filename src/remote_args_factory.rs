pub fn make(output_list_path: &str, output_vid_path: String) -> [String; 8] {
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
