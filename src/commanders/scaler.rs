use std::path::{Path, PathBuf};

use regex::Regex;

use crate::{commanders::_cmd, ffmpeg_args_factory};

pub fn execute(file_format: &str, paths: Vec<PathBuf>, tmp_dir: &Path) {
    println!("👷 Start rescaling videos...\n");
    let regex_str = format!(r"\.{}$", regex::escape(&file_format.to_string()));
    let re = Regex::new(regex_str.as_str()).unwrap();
    for path in paths {
        let display = path.display();
        if !display.to_string().contains("/.") && re.is_match(&format!("{}", display)) {
            let file = path.file_name().unwrap().to_str().unwrap();

            let args = ffmpeg_args_factory::make_scale_args(file, tmp_dir);
            _cmd::scale(args);
        }
    }
    println!("\n🙌 Scaled!");
}
