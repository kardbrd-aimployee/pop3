use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

use pop3::render::app::{App, AppConfig};

fn parse_light(s: &str) -> Option<(i16, i16)> {
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() != 2 {
        return None;
    }
    Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
}

fn cli() -> Command {
    let args = [
        Arg::new("base")
            .long("base")
            .action(ArgAction::Set)
            .value_name("BASE_PATH")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to POP3 directory"),
        Arg::new("level")
            .long("level")
            .action(ArgAction::Set)
            .value_name("LEVEL")
            .value_parser(clap::value_parser!(u8).range(1..255))
            .help("Level number"),
        Arg::new("landtype")
            .long("landtype")
            .action(ArgAction::Set)
            .value_name("LAND_TYPE")
            .value_parser(clap::builder::StringValueParser::new())
            .help("Override land type"),
        Arg::new("cpu")
            .long("cpu")
            .action(ArgAction::SetTrue)
            .help("Enable CPU texture rendering"),
        Arg::new("cpu-full")
            .long("cpu-full")
            .action(ArgAction::SetTrue)
            .help("Enable full CPU texture rendering"),
        Arg::new("light")
            .long("light")
            .action(ArgAction::Set)
            .help("Light configuration x;y"),
        Arg::new("debug")
            .long("debug")
            .action(ArgAction::SetTrue)
            .help("Enable debug printing"),
        Arg::new("script")
            .long("script")
            .action(ArgAction::Set)
            .value_name("SCRIPT_PATH")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Replay key events from a script file"),
    ];
    Command::new("pop3").about("POP3 wgpu renderer").args(&args)
}

fn main() {
    let matches = cli().get_matches();

    let config = AppConfig {
        base: matches.get_one("base").cloned(),
        level: matches.get_one("level").copied(),
        landtype: matches.get_one("landtype").cloned(),
        cpu: matches.get_flag("cpu"),
        cpu_full: matches.get_flag("cpu-full"),
        debug: matches.get_flag("debug"),
        light: matches
            .get_one::<String>("light")
            .and_then(|s| parse_light(s)),
        script: matches.get_one("script").cloned(),
    };

    let log_level: &str = if config.debug { "debug" } else { "info" };
    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", log_level)
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    App::run(config);
}
