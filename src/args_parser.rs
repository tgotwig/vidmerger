use clap::{load_yaml, App, AppSettings, ArgMatches};

pub fn fetch() -> (String, String, bool) {
    let matches: ArgMatches = App::from(load_yaml!("cli.yaml"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();
    return (
        matches.value_of("DIR").unwrap().to_string(),
        matches
            .value_of("format")
            .unwrap_or("avchd,avi,flv,mkv,mov,mp4,webm,wmv")
            .to_string(),
        matches.is_present("preview"),
    );
}
