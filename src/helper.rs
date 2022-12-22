use crate::helpers::io_helper::read_dir;
use crate::helpers::vec_helper::filter_files;
use std::fmt::Write as FmtWrite;
use std::fs::{canonicalize, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

pub fn gen_ffmpeg_input_content(target_dir: &Path, file_format: &str) -> String {
    let all_files_on_target_dir: Vec<PathBuf> = read_dir(target_dir).unwrap();
    let files_to_merge = filter_files(all_files_on_target_dir, file_format);
    let mut ffmpeg_input_content = String::new();

    for file_to_merge in files_to_merge {
        writeln!(
            ffmpeg_input_content,
            "file '{}'",
            canonicalize(file_to_merge).unwrap().display()
        )
        .unwrap();
    }
    ffmpeg_input_content
}

pub fn print_order_of_merging(ffmpeg_input_content: &str) -> String {
    println!("\n👇 Order of merging:\n");
    let file_names_to_be_merged = ffmpeg_input_content
        .lines()
        .map(|line| {
            format!(
                "📄 {}",
                BrightBlue.paint(line.split(['/', '\\']).last().unwrap().replace('\'', ""))
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    println!("{}\n", file_names_to_be_merged); // todo: mock this for unit tests
    file_names_to_be_merged
}

pub fn gen_ffmpeg_input_file(string: String, mut dir: PathBuf) -> PathBuf {
    dir.push("ffmpeg_input_file.txt");
    File::create(&dir)
        .unwrap()
        .write_all(string.as_bytes())
        .unwrap();
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_order_of_merging_with_slashes() {
        assert_eq!(
            print_order_of_merging("/target_dir/1.mp4\n/target_dir/2.mp4"),
            "📄 1.mp4\n📄 2.mp4"
        );
    }

    #[test]
    fn test_print_order_of_merging_with_backslashes() {
        assert_eq!(
            print_order_of_merging("C:\\target_dir\\1.mp4\nC:\\target_dir\\2.mp4"),
            "📄 1.mp4\n📄 2.mp4"
        );
    }
}
