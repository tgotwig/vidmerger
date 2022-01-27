use clap::{lazy_static::lazy_static, load_yaml, App, AppSettings, ArgMatches};

lazy_static! {
    static ref ARGS: ArgMatches = App::from(load_yaml!("cli.yaml"))
        .setting(AppSettings::ArgRequiredElseHelp)
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
