use crate::commanders::_cmd;
use term_painter::Color::BrightBlue;
use term_painter::ToStyle;

pub fn merge(input: String, output: String) {
    let child = _cmd::merge(input, &output);

    let res = child.unwrap().wait_with_output();

    if res.is_ok() {
        println!("🐣 Generated: {}", BrightBlue.paint(output));
    } else {
        panic!("❌ Something went wrong: \n\n{}", res.unwrap_err());
    }
}
