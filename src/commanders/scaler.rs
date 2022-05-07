use std::path::{Path, PathBuf};

use regex::Regex;

use crate::{commanders::_cmd, ffmpeg_args_factory};

pub fn execute(file_format: &str, paths: Vec<PathBuf>, tmp_dir: &Path) {
    println!("👷 Start rescaling videos...\n");
    let regex_str = format!(r"\.{}$", regex::escape(file_format));
    let re = Regex::new(regex_str.as_str()).unwrap();
    for path in paths {
        let display = path.display();
        let display_str = display.to_string();
        let file_starts_with_dot = display_str.contains("/.") || display_str.contains("\\.");
        if !file_starts_with_dot && re.is_match(&format!("{}", display)) {
            let file = path.file_name().unwrap().to_str().unwrap();

            let args = ffmpeg_args_factory::make_scale_args(file, tmp_dir);
            _cmd::scale(args);
        }
    }
    println!("\n🙌 Scaled!");
}
