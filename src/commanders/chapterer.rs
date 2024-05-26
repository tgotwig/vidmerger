use crate::commanders::_cmd;
use path_slash::PathExt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

pub fn execute(
    files_to_merge_as_strings: Vec<String>,
    tmp_dir: PathBuf,
    ffmpeg_output_file: PathBuf,
    file_format: &String,
) {
    let mut start_time = 0;
    let mut metadata_string = String::from(";FFMETADATA1\n");

    for path in files_to_merge_as_strings {
        let title = extract_title(&path, file_format);
        let duration = _cmd::get_media_seconds(&path).unwrap() as i64;

        metadata_string.push_str(&format!(
            "\n[CHAPTER]\nTIMEBASE=1/1\nSTART={}\nEND={}\ntitle={}\n",
            start_time,
            start_time + duration,
            title
        ));

        start_time += duration;
    }

    let mut file_path = tmp_dir;
    file_path.push("chapters.txt");
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(metadata_string.as_bytes())
        .expect("Failed to write to file");

    let input_file_for_chapterer: String = ffmpeg_output_file.to_slash().unwrap().to_string();
    let mut output_with_chapters = ffmpeg_output_file.clone();
    let output_file_name = format!("output_with_chapters.{}", file_format);
    output_with_chapters.set_file_name(&output_file_name);
    let output_file_for_chapterer: String = output_with_chapters.to_slash().unwrap().to_string();

    _cmd::merge_with_chapters(
        &input_file_for_chapterer,
        file_path,
        &output_file_for_chapterer,
    )
    .unwrap()
    .wait_with_output()
    .unwrap();
    println!(
        "✅ Successfully generated: {}",
        BrightBlue.paint(&output_file_for_chapterer)
    );

    fs::remove_file(Path::new(&input_file_for_chapterer)).unwrap();
    println!(
        "✅ Successfully deleted: {}",
        BrightBlue.paint(&input_file_for_chapterer)
    );

    fs::rename(&output_file_for_chapterer, ffmpeg_output_file).unwrap();
    println!(
        "✅ Successfully renamed: {} to {}",
        BrightBlue.paint(output_file_for_chapterer),
        BrightBlue.paint(input_file_for_chapterer)
    );
}

fn extract_title(path: &str, file_format: &str) -> String {
    let file_name = path.split('/').last().unwrap_or("");
    let mut parts = file_name.splitn(2, '-');
    parts.next(); // Skip the part before the first '-'
    let content_with_extension = parts.next().unwrap_or("").trim();

    let format_str = format!(".{}", file_format);
    content_with_extension
        .split(&format_str)
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(test)]
mod test_extract_title {
    use super::extract_title;

    #[test]
    fn test_extract_title() {
        let path = "path/to/video-Title of Video.mp4";
        assert_eq!(extract_title(path, "mp4"), "Title of Video");
    }

    #[test]
    fn test_extract_title_with_dot() {
        let path = "path/to/video-[1.0] Title of Video.mp4";
        assert_eq!(extract_title(path, "mp4"), "[1.0] Title of Video");
    }

    #[test]
    fn test_extract_title_with_no_dash() {
        let path = "path/to/videoTitle of Video.mp4";
        assert_eq!(extract_title(path, "mp4"), "");
    }

    #[test]
    fn test_extract_title_with_no_extension() {
        let path = "path/to/video-Title of Video";
        assert_eq!(extract_title(path, "mp4"), "Title of Video");
    }

    #[test]
    fn test_extract_title_with_multiple_dashes() {
        let path = "path/to/video-Title-of-Video.mp4";
        assert_eq!(extract_title(path, "mp4"), "Title-of-Video");
    }

    #[test]
    fn test_extract_title_with_empty_path() {
        let path = "";
        assert_eq!(extract_title(path, "mp4"), "");
    }

    // FAIL
    #[test]
    fn test_extract_title_with_only_dashes() {
        let path = "---";
        assert_eq!(extract_title(path, "mp4"), "--");
    }

    #[test]
    fn test_extract_title_with_special_characters() {
        let path = "path/to/video-Title_@_of_Video.mp4";
        assert_eq!(extract_title(path, "mp4"), "Title_@_of_Video");
    }

    #[test]
    fn test_extract_title_with_different_format() {
        let path = "path/to/video-Title of Video.avi";
        assert_eq!(extract_title(path, "mp4"), "Title of Video.avi");
    }

    #[test]
    fn test_extract_title_with_nested_path() {
        let path = "path/to/some/other/folder/video-Title.mp4";
        assert_eq!(extract_title(path, "mp4"), "Title");
    }
}
