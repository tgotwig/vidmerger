use crate::commanders::_cmd;

pub fn merge(ffmpeg_args: [String; 10], file_format: String) {
    let child = _cmd::merge(ffmpeg_args);

    let res = child.unwrap().wait_with_output();
    println!("{:?}\n", res);

    if res.is_ok() {
        println!(
            "✅ Successfully generated 'output.{}'! (it can still be broken 🙈)",
            file_format
        )
    } else {
        println!("❌ Something went wrong 😖");
        println!("{:?}\n", res.unwrap_err())
    }
}
