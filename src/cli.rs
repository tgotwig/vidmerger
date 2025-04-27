use clap::{Arg, ArgAction, ArgMatches, Command};

pub struct Cli {
  matches: ArgMatches,
}

impl Cli {
  pub fn init() -> Self {
    let matches = Command::new("vidmerger")
        .version("0.4.0")
        .author("Thomas Gotwig")
        .about("A wrapper around ffmpeg which simplifies merging multiple videos 🎞  Everything in between the first `-` till the fill extension of the input files will be used as chapter titles 📖.")
        .arg(Arg::new("TARGET_DIR")
            .help("Sets the input file to use")
            .required(true)
            .index(1)
        )
        .arg(Arg::new("format")
            .short('f')
            .long("format")
            .help("Specifies which formats should be merged individually, the default is 👉 3g2,3gp,aac,ac3,alac,amr,ape,au,avi,awb,dts,f4a,f4b,f4p,f4v,flac,flv,m4a,m4b,m4p,m4r,m4v,mkv,mov,mp2,mp3,mp4,mpeg,mpg,oga,ogg,ogm,ogv,ogx,opus,pcm,spx,wav,webm,wma,wmv")
            .num_args(1)
        )
        .arg(Arg::new("fps")
            .long("fps")
            .help("Generates videos inside a temporary folder with this fps value and merges them")
            .num_args(1)
        )
        .arg(Arg::new("shutdown")
            .long("shutdown")
            .help("For doing a shutdown at the end (needs sudo)")
            .action(ArgAction::SetTrue)
        )
        .arg(Arg::new("skip-fps-changer")
            .long("skip-fps-changer")
            .help("Skips the fps changer")
            .action(ArgAction::SetTrue)
        )
        .arg(Arg::new("yes")
            .long("yes")
            .short('y')
            .help("Skips confirmation of merge order")
            .action(ArgAction::SetTrue)
        )
        .arg(Arg::new("verbose")
            .long("verbose")
            .help("Prints detailed logs")
            .action(ArgAction::SetTrue)
        )
        .arg_required_else_help(true)
        .get_matches();

    Cli { matches }
  }

  pub fn get_matches(self) -> ArgMatches {
    self.matches
  }
}
