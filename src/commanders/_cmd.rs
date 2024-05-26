use std::{
    io::Error,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
};

use path_slash::PathBufExt;

pub fn merge(input: String, output: String) -> Result<Child, std::io::Error> {
    let cmd = format!(
        "ffmpeg -y -f concat -safe 0 -i {} -map 0 -c copy {}",
        input, output
    );

    println!("🚀 Start Merger, calling: `{}`\n", cmd);
    execute_cmd(cmd)
}

pub fn merge_with_chapters(
    input_file_for_chapterer: &str,
    file_path: PathBuf,
    output_file_for_chapterer: &str,
) -> Result<Child, std::io::Error> {
    let cmd = format!(
        "ffmpeg -y -i {} -i {} -map 0 -map_metadata 1 -codec copy {}",
        &input_file_for_chapterer,
        file_path.to_str().unwrap(),
        output_file_for_chapterer
    );

    println!("🚀 Calling:\n");
    println!("- {}\n", cmd);

    execute_cmd(cmd)
}

pub fn run_ffmpeg_info_command(file_to_merge: &PathBuf) -> Result<Output, Error> {
    Command::new("ffmpeg")
        .args(["-i", &file_to_merge.to_slash().unwrap()])
        .output()
}

pub fn adjust_fps_by_ffmpeg(
    file_to_merge: PathBuf,
    fps_goal: &f32,
    new_file_location: PathBuf,
) -> PathBuf {
    let cmd = format!(
        "ffmpeg -i {} -r {} {}",
        file_to_merge.to_str().unwrap(),
        fps_goal,
        new_file_location.to_str().unwrap()
    );
    println!("- {}", cmd);

    let res = execute_cmd(cmd).unwrap().wait_with_output();
    println!("{:?}", res);
    new_file_location
}

pub fn get_media_seconds(media_path: &str) -> Result<f64, Box<Error>> {
    let cmd = format!(
        "ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 '{}'",
        media_path
    );

    println!("🚀 Calling:\n");
    println!("- {}\n", cmd);
    let res = execute_cmd(cmd);

    let output = res.unwrap().wait_with_output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.trim().parse::<f64>().unwrap();
    Ok(output)
}

fn execute_cmd(cmd: String) -> Result<Child, std::io::Error> {
    let (interpreter, arg) = if cfg!(target_os = "windows") {
        ("powershell", "/c")
    } else {
        ("sh", "-c")
    };
    Command::new(interpreter)
        .arg(arg)
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
}
