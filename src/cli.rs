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
        .arg(Arg::new("TARGET_DIR")
            .help("Sets the input file to use")
            .required(true)
            .index(1)
        )
        .arg(Arg::new("format")
            .short('f')
            .long("format")
            .help("Specifies which formats should be merged individually, the default is 👉 3g2,3gp,aac,ac3,alac,amr,ape,au,avi,awb,dts,f4a,f4b,f4p,f4v,flac,flv,m4a,m4b,m4p,m4r,m4v,mkv,mov,mp2,mp3,mp4,mpeg,mpg,oga,ogg,ogm,ogv,ogx,opus,pcm,spx,wav,webm,wma,wmv")
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
