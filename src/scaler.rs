use std::fs::DirEntry;

use regex::Regex;

use crate::{cmd, remote_args_factory};

pub fn execute(file_format: &str, paths: Vec<DirEntry>) {
    println!("👷 Start rescaling videos...\n");
    let regex_str = format!(r"\.{}$", regex::escape(&file_format.to_string()));
    let re = Regex::new(regex_str.as_str()).unwrap();
    for path in paths {
        let path = path.path();
        if re.is_match(&format!("{}", path.display())) {
            let file = path.file_name().unwrap().to_str().unwrap();

            let args = remote_args_factory::make_scale_args(file);
            cmd::scale(args);
        }
    }
    println!("\n🙌 Scaled!");
}
