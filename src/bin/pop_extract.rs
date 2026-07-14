use std::error::Error;
use std::io;
use std::path::PathBuf;

use clap::{Arg, Command};
use pop3::extract::structures::{export_structure_icons, StructureIconRequest, Tribe};

fn cli() -> Command {
    Command::new("pop_extract")
        .about("Extract named, reusable assets from original Populous: The Beginning data")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .arg(
            Arg::new("base")
                .long("base")
                .value_name("GAME_DIR")
                .value_parser(clap::value_parser!(PathBuf))
                .required(true)
                .help("Original Populous: The Beginning installation directory"),
        )
        .subcommand(
            Command::new("structure-icons")
                .about(
                    "Render named construction/building icons from original OBJS and texture data",
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("DIR")
                        .value_parser(clap::value_parser!(PathBuf))
                        .required(true)
                        .help("Destination directory for icons, contact sheet, and manifest"),
                )
                .arg(
                    Arg::new("landscape")
                        .long("landscape")
                        .value_name("KEY")
                        .default_value("0")
                        .help("One-character landscape/palette bank key"),
                )
                .arg(
                    Arg::new("tribe")
                        .long("tribe")
                        .value_name("TRIBE")
                        .value_parser(["blue", "red", "yellow", "green"])
                        .default_value("blue")
                        .help("Tribal model set to render"),
                )
                .arg(
                    Arg::new("size")
                        .long("size")
                        .value_name("PIXELS")
                        .value_parser(clap::value_parser!(u32).range(64..=1024))
                        .default_value("160")
                        .help("Width and height of each transparent PNG icon"),
                ),
        )
}

fn main() {
    if let Err(error) = run() {
        eprintln!("pop_extract: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let matches = cli().get_matches();
    let base = matches
        .get_one::<PathBuf>("base")
        .expect("required by clap")
        .clone();
    match matches.subcommand() {
        Some(("structure-icons", args)) => {
            let tribe_value = args.get_one::<String>("tribe").expect("defaulted by clap");
            let tribe = Tribe::parse(tribe_value)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unsupported tribe"))?;
            let request = StructureIconRequest {
                base,
                output: args
                    .get_one::<PathBuf>("output")
                    .expect("required by clap")
                    .clone(),
                landscape: args
                    .get_one::<String>("landscape")
                    .expect("defaulted by clap")
                    .to_ascii_lowercase(),
                tribe,
                size: *args.get_one::<u32>("size").expect("defaulted by clap"),
            };
            let result = export_structure_icons(&request)?;
            println!("Extracted {} structure icons", result.icon_count);
            println!("Manifest: {}", result.manifest_path.display());
            println!("Contact sheet: {}", result.contact_sheet_path.display());
        }
        _ => unreachable!("subcommand required by clap"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        cli().debug_assert();
    }
}
