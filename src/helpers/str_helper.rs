use std::fmt::Write as FmtWrite;

pub fn split(string: String) -> Vec<String> {
    string.split(',').map(|s| s.to_string()).collect()
}

pub fn gen_input_file_content_for_ffmpeg(files_to_merge: Vec<String>) -> String {
    let mut ffmpeg_input_content = String::new();

    for file_to_merge in files_to_merge {
        if file_to_merge.trim().is_empty() {
            continue;
        }
        writeln!(ffmpeg_input_content, "file '{}'", file_to_merge).unwrap();
    }
    ffmpeg_input_content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let string = String::from("mp4,mkv,avi");
        let file_formats = split(string);
        assert_eq!(file_formats, vec!["mp4", "mkv", "avi"]);
    }

    #[test]
    fn test_split_with_space() {
        let string = String::from("mp4,mkv, avi");
        let file_formats = split(string);
        assert_eq!(file_formats, vec!["mp4", "mkv", " avi"]);
    }

    #[test]
    fn test_split_with_empty_input() {
        let string = String::from("");
        let file_formats = split(string);
        assert_eq!(file_formats, vec![""]);
    }

    #[test]
    fn test_gen_file_to_merge_with_one_input() {
        let files_to_merge = vec![String::from("/1.mp4")];
        let ffmpeg_input_content = gen_input_file_content_for_ffmpeg(files_to_merge);
        assert_eq!(ffmpeg_input_content, "file '/1.mp4'\n")
    }

    #[test]
    fn test_gen_file_to_merge_with_multiple_inputs() {
        let files_to_merge = vec![
            String::from("/1.mp4"),
            String::from("/2.mp4"),
            String::from("/3.mp4"),
        ];
        let ffmpeg_input_content = gen_input_file_content_for_ffmpeg(files_to_merge);
        assert_eq!(
            ffmpeg_input_content,
            "file '/1.mp4'\nfile '/2.mp4'\nfile '/3.mp4'\n"
        )
    }

    #[test]
    fn test_gen_file_to_merge_with_empty_input() {
        let files_to_merge = vec![];
        let ffmpeg_input_content = gen_input_file_content_for_ffmpeg(files_to_merge);
        assert_eq!(ffmpeg_input_content, "")
    }

    #[test]
    fn test_gen_file_to_merge_with_empty_string_input() {
        let files_to_merge = vec![String::from("")];
        let ffmpeg_input_content = gen_input_file_content_for_ffmpeg(files_to_merge);
        assert_eq!(ffmpeg_input_content, "")
    }
}
