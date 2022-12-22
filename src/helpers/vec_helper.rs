use std::path::PathBuf;

use regex::Regex;

pub fn filter_files(all_files: Vec<PathBuf>, file_format: &str) -> Vec<PathBuf> {
    let re = Regex::new(format!(r"[\\/][^.]*\.{}$", regex::escape(file_format)).as_str()).unwrap();
    let mut filtered_files = Vec::new();

    for possible_file_to_merge in all_files {
        if re.is_match(&format!("{}", possible_file_to_merge.display())) {
            filtered_files.push(possible_file_to_merge);
        }
    }
    filtered_files
}
