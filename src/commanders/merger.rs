use crate::commanders::_cmd;

pub fn merge(ffmpeg_args: [String; 10], file_format: String) {
    let child = _cmd::merge(ffmpeg_args);

    let res = child.unwrap().wait_with_output();
    println!("{:?}", res);

    println!("----------------------------------------------------------------");
    if res.is_ok() {
        println!("✅ Successfully generated:");
        println!();
        println!("- output.{}", file_format);
    } else {
        panic!("❌ Something went wrong: \n\n{}", res.unwrap_err());
    }
}
