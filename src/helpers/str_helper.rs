pub fn split(string: String) -> Vec<String> {
    string.split(',').map(|s| s.to_string()).collect()
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
}
