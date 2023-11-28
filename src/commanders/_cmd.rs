use std::{
    io::Error,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
};

use path_slash::PathBufExt;

pub fn merge(args: [String; 10]) -> Result<Child, std::io::Error> {
    let cmd = format!("ffmpeg {}", args.join(" "));

    println!("- {}", cmd);
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

fn execute_cmd(cmd: String) -> Result<Child, std::io::Error> {
    let (interpreter, arg) = if cfg!(target_os = "windows") {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };
    Command::new(interpreter)
        .arg(arg)
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
}
