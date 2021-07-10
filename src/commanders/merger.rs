use crate::cmd;

pub fn merge(ffmpeg_args: [String; 8], file_format: String) {
    let child = cmd::merge(ffmpeg_args);

    let res = child.unwrap().wait_with_output();
    println!("{:?}\n", res);
    if res.unwrap().status.success() {
        println!("✅ Successfully generated 'output.{}'! 😆🎞", file_format)
    } else {
        println!("❌ Something went wrong 😖")
    }
}
