use clap::{lazy_static::lazy_static, Arg, ArgMatches, Command};

lazy_static! {
    static ref ARGS: ArgMatches = Command::new("vidmerger")
        .version("0.1.6")
        .author("Thomas Gotwig")
        .about("A wrapper around ffmpeg which simlifies merging multiple videos 🎞")
        .arg(Arg::new("DIR")
            .help("Sets the input file to use")
            .required(true)
            .index(1)
        )
        .arg(Arg::new("format")
            .short('f')
            .long("format")
            .help("Specifies which formats should be merged individually, the default is 👉 avchd,avi,flv,mkv,mov,mp4,webm,wmv")
            .takes_value(true)
        )
        .arg(Arg::new("preview")
            .short('p')
            .long("preview")
            .help("Prints previews of the merge-orders without merging them")
        )
        .arg(Arg::new("scale")
            .short('s')
            .long("scale")
            .help("Scales all videos up before merging, a valid value would be \"320:240\"")
            .takes_value(true)
        )
        .arg(Arg::new("shutdown")
            .long("shutdown")
            .help("For doing a shutdown at the end (needs sudo)")
        )
        .arg_required_else_help(true)
        .get_matches();
}

pub fn get_dir() -> String {
    ARGS.value_of("DIR").unwrap().to_string()
}

pub fn get_format() -> String {
    ARGS.value_of("format")
        .unwrap_or("avchd,avi,flv,mkv,mov,mp4,webm,wmv")
        .to_string()
}

pub fn get_preview() -> bool {
    ARGS.is_present("preview")
}

pub fn get_scale() -> Option<&'static str> {
    ARGS.value_of("scale")
}

pub fn get_shutdown() -> bool {
    ARGS.is_present("shutdown")
}
