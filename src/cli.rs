use clap::{Arg, ArgMatches, Command};

pub struct Cli {
    matches: ArgMatches,
}

impl Cli {
    pub fn init() -> Self {
        let matches = Command::new("vidmerger")
        .version("0.2.0")
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
        .arg(Arg::new("shutdown")
            .long("shutdown")
            .help("For doing a shutdown at the end (needs sudo)")
        )
        .arg_required_else_help(true)
        .get_matches();

        Cli { matches }
    }

    pub fn get_matches(self) -> ArgMatches {
        self.matches
    }
}
