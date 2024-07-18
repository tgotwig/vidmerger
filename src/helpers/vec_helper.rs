use std::path::PathBuf;

use regex::Regex;

/// Returns a vector of PathBufs that match the given file format.
/// Also filters out files that start with a dot.
pub fn filter_files(all_files: Vec<PathBuf>, file_format: &str) -> Vec<PathBuf> {
    let re: Regex =
        Regex::new(format!(r"(?i)[\\/][^.\\/][^\\/]*\.{}$", regex::escape(file_format)).as_str())
            .unwrap();
    let mut filtered_files = Vec::new();

    for possible_file_to_merge in all_files {
        if re.is_match(&format!("{}", possible_file_to_merge.display())) {
            filtered_files.push(possible_file_to_merge);
        }
    }
    filtered_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_filter_files() {
        let all_files = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("/path/to/file2.txt"),
            PathBuf::from("/path/to/file3.doc"),
        ];
        let file_format = "txt";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 2);
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file1.txt")));
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file2.txt")));
    }

    #[test]
    fn test_filter_files_with_dots_at_the_start() {
        let all_files = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("/path/to/file2.txt"),
            PathBuf::from("/path/to/file3.doc"),
            PathBuf::from("/path/to/file4.txt.txt"),
            PathBuf::from("/path/to/.file5.txt"),
            PathBuf::from("/path/to/.file6.txt.txt"),
        ];
        let file_format = "txt";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 3);
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file1.txt")));
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file2.txt")));
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file4.txt.txt")));
    }

    #[test]
    fn test_filter_files_with_multiple_dots_after_the_start() {
        let all_files = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("/path/to/file2.txt"),
            PathBuf::from("/path/to/file3.doc"),
            PathBuf::from("/path/to/file4.txt.txt"),
        ];
        let file_format = "txt";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 3);
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file1.txt")));
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file2.txt")));
        assert!(filtered_files.contains(&PathBuf::from("/path/to/file4.txt.txt")));
    }

    #[test]
    fn test_filter_files_with_no_matches() {
        let all_files = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("/path/to/file2.txt"),
            PathBuf::from("/path/to/file3.doc"),
        ];
        let file_format = "mp4";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 0);
    }

    #[test]
    fn test_filter_files_with_no_files() {
        let all_files = vec![];
        let file_format = "mp4";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 0);
    }

    #[test]
    fn test_filter_files_with_no_file_format() {
        let all_files = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("/path/to/file2.txt"),
            PathBuf::from("/path/to/file3.doc"),
        ];
        let file_format = "";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 0);
    }

    #[test]
    fn test_filter_files_with_no_file_format_and_no_files() {
        let all_files = vec![];
        let file_format = "";
        let filtered_files = filter_files(all_files, file_format);

        assert_eq!(filtered_files.len(), 0);
    }
}
