use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

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
