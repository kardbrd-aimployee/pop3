use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command};

use pop3::render::app::{App, AppConfig};

fn parse_light(s: &str) -> Option<(i16, i16)> {
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() != 2 {
        return None;
    }
    Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
}

/// Parse a physical framebuffer size used for deterministic visual captures.
///
/// This deliberately accepts the compact `WIDTHxHEIGHT` form used by the
/// original HUD evidence so a comparison does not depend on monitor scaling.
fn parse_window_size(s: &str) -> Result<(u32, u32), String> {
    let (width, height) = s
        .trim()
        .split_once(['x', 'X'])
        .ok_or_else(|| "expected WIDTHxHEIGHT".to_owned())?;
    let width = width
        .parse::<u32>()
        .map_err(|_| "width must be a positive integer".to_owned())?;
    let height = height
        .parse::<u32>()
        .map_err(|_| "height must be a positive integer".to_owned())?;
    if width == 0 || height == 0 {
        return Err("width and height must be greater than zero".to_owned());
    }
    Ok((width, height))
}

/// Locate the data directory embedded in a conventional macOS app bundle.
///
/// For `Example.app/Contents/MacOS/binary`, the bundled original-game base is
/// `Example.app/Contents/Resources/original_game`.
fn bundled_base_from_executable(executable: &Path) -> Option<PathBuf> {
    let contents = executable.parent()?.parent()?;
    Some(contents.join("Resources").join("original_game"))
}

fn bundled_original_game_base() -> Option<PathBuf> {
    bundled_base_from_executable(&std::env::current_exe().ok()?)
        .filter(|candidate| candidate.join("data").is_dir())
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
        Arg::new("window-size")
            .long("window-size")
            .action(ArgAction::Set)
            .value_name("WIDTHxHEIGHT")
            .value_parser(parse_window_size)
            .help("Use an exact physical framebuffer size for a visual capture"),
    ];
    Command::new("pop3").about("POP3 wgpu renderer").args(&args)
}

fn main() {
    let matches = cli().get_matches();

    let config = AppConfig {
        base: matches
            .get_one("base")
            .cloned()
            .or_else(bundled_original_game_base),
        level: matches.get_one("level").copied(),
        landtype: matches.get_one("landtype").cloned(),
        cpu: matches.get_flag("cpu"),
        cpu_full: matches.get_flag("cpu-full"),
        debug: matches.get_flag("debug"),
        light: matches
            .get_one::<String>("light")
            .and_then(|s| parse_light(s)),
        script: matches.get_one("script").cloned(),
        window_size: matches.get_one("window-size").copied(),
    };

    let log_level: &str = if config.debug { "debug" } else { "info" };
    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", log_level)
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    App::run(config);
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{bundled_base_from_executable, parse_window_size};

    #[test]
    fn bundled_base_resolves_from_macos_executable_path() {
        assert_eq!(
            bundled_base_from_executable(Path::new(
                "/Applications/Populous 3 Rust.app/Contents/MacOS/Populous3Rust"
            )),
            Some(PathBuf::from(
                "/Applications/Populous 3 Rust.app/Contents/Resources/original_game"
            ))
        );
    }

    #[test]
    fn parse_window_size_accepts_physical_dimensions() {
        assert_eq!(parse_window_size("3648x2500"), Ok((3648, 2500)));
        assert_eq!(parse_window_size("1024X720"), Ok((1024, 720)));
    }

    #[test]
    fn parse_window_size_rejects_missing_or_zero_dimensions() {
        assert!(parse_window_size("1024").is_err());
        assert!(parse_window_size("0x720").is_err());
        assert!(parse_window_size("1024x0").is_err());
    }
}
