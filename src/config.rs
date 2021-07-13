use clap::{lazy_static::lazy_static, load_yaml, App, AppSettings, ArgMatches};

lazy_static! {
    static ref ARGS: ArgMatches = App::from(load_yaml!("cli.yaml"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();
}

pub fn get() -> (String, String, bool, Option<&'static str>) {
    return (
        ARGS.value_of("DIR").unwrap().to_string(),
        ARGS.value_of("format")
            .unwrap_or("avchd,avi,flv,mkv,mov,mp4,webm,wmv")
            .to_string(),
        ARGS.is_present("preview"),
        ARGS.value_of("scale"),
    );
}
