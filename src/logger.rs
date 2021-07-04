use std::{io::Error, process::Child};

pub fn print_end_status(child: Result<Child, Error>, file_format: String) {
    let res = child.unwrap().wait_with_output();
    println!("{:?}\n", res);
    if res.unwrap().status.success() {
        println!("✅ Successfully generated 'output.{}'! 😆🎞", file_format)
    } else {
        println!("❌ Something went wrong 😖")
    }
}
