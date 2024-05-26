use super::fps_reader::get_fps;
use crate::{
    cli::Cli,
    commanders,
    helpers::{
        io_helper::path_bufs_to_sorted_strings, str_helper::gen_input_file_content_for_ffmpeg,
    },
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

pub fn change_fps(
    files_to_merge: Vec<PathBuf>,
    tmp_dir: &Path,
    fps_from_cli: f32,
) -> (Vec<PathBuf>, Vec<std::string::String>, std::string::String) {
    let matches = Cli::init().get_matches();
    let verbose: bool = matches.is_present("verbose");

    let mut new_files_to_merge = Vec::new();
    let mut map: HashMap<&PathBuf, f32> = HashMap::new();

    for file_to_merge in &files_to_merge {
        map.insert(file_to_merge, get_fps(file_to_merge));
    }

    let fps_goal = if fps_from_cli != 0. {
        fps_from_cli
    } else {
        *map.values()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap()
    };

    let set: HashSet<String> = map.values().map(|value| value.to_string()).collect();
    let files_to_merge = if set.len() > 1 {
        let mut output_directly = Vec::new();
        for (key, value) in &map {
            if value == &fps_goal {
                output_directly.push(format!(
                    "- {} ({} fps)",
                    key.file_name().unwrap().to_string_lossy(),
                    value
                ));
            }
        }
        output_directly.sort();

        let mut output_indirectly = Vec::new();
        for (key, value) in &map {
            if value != &fps_goal {
                output_indirectly.push(format!(
                    "- {} ({} fps)",
                    key.file_name().unwrap().to_string_lossy(),
                    value
                ));
            }
        }
        output_indirectly.sort();

        println!(
            "🔎 FPS mismatches detected ({:?}), scaling to {} fps: {}/{}",
            set,
            fps_goal,
            output_indirectly.len(),
            set.len()
        );

        if verbose {
            println!();
            println!("Will be merged directly: \n");
            for line in output_directly {
                println!("{}", line);
            }
            println!();
            println!("Will be merged indirectly, generating new files from listed below with {} fps and merges with listed above:", fps_goal);
            println!();
            for line in output_indirectly {
                println!("{}", line);
            }
            println!();
        }

        for file_to_merge in files_to_merge {
            let fps = get_fps(&file_to_merge);

            if fps != fps_goal {
                let new_file_to_merge =
                    commanders::fps_adjuster::adjust_fps(file_to_merge, &fps_goal, tmp_dir);
                new_files_to_merge.push(new_file_to_merge);
            } else {
                new_files_to_merge.push(file_to_merge);
            }
        }

        new_files_to_merge
    } else {
        files_to_merge
    };

    let files_to_merge_as_strings = path_bufs_to_sorted_strings(&files_to_merge);
    let ffmpeg_input_content = gen_input_file_content_for_ffmpeg(&files_to_merge_as_strings);

    (
        files_to_merge,
        files_to_merge_as_strings,
        ffmpeg_input_content,
    )
}
