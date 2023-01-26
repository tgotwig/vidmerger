use std::{borrow::Cow, fmt::Write as FmtWrite};
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

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

pub fn extract_fps_from_ffmpeg_output(str: Cow<str>) -> f32 {
    let split = str.split(" fps,").collect::<Vec<&str>>();
    let split_from_split = split[0].split(' ').collect::<Vec<&str>>();
    let fps = String::from(split_from_split[split_from_split.len() - 1]);
    fps.parse::<f32>().unwrap_or_default()
}

pub fn create_order_of_merging(ffmpeg_input_content: &str) -> String {
    ffmpeg_input_content
        .lines()
        .map(|line| {
            format!(
                "- {}",
                BrightBlue.paint(line.split(['/', '\\']).last().unwrap().replace('\'', ""))
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
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

    #[test]
    fn extract_fps_from_ffmpeg_output_with_30_fps() {
        let fps = extract_fps_from_ffmpeg_output(Cow::from("Stream #0:0(und): Video: h264 (Main) (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 1280x720 [SAR 1:1 DAR 16:9], 201 kb/s, 30 fps, 30 tbr, 90k tbn"));
        assert_eq!(fps, 30.);
    }

    #[test]
    fn extract_fps_from_ffmpeg_output_with_28_fps() {
        let fps = extract_fps_from_ffmpeg_output(Cow::from("Stream #0:0(und): Video: h264 (Main) (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 1280x720 [SAR 1:1 DAR 16:9], 201 kb/s, 28 fps, 30 tbr, 90k tbn"));
        assert_eq!(fps, 28.);
    }

    #[test]
    fn test_create_order_of_merging_with_slashes() {
        assert_eq!(
            create_order_of_merging("/target_dir/1.mp4\n/target_dir/2.mp4"),
            "- 1.mp4\n- 2.mp4"
        );
    }

    #[test]
    fn test_create_order_of_merging_with_backslashes() {
        assert_eq!(
            create_order_of_merging("C:\\target_dir\\1.mp4\nC:\\target_dir\\2.mp4"),
            "- 1.mp4\n- 2.mp4"
        );
    }
}
