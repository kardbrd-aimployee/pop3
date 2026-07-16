use std::error::Error;
use std::io;
use std::path::PathBuf;

use clap::{Arg, Command};
use pop3::extract::animations::{export_unit_animations, UnitAnimationRequest};
use pop3::extract::building_panel::{export_building_panel_icons, BuildingPanelIconRequest};
use pop3::extract::hud::{export_hud_sprite_candidates, HudSpriteBank, HudSpriteCandidateRequest};
use pop3::extract::structures::{
    export_structure_icons, StructureIconRequest, Tribe as StructureTribe,
};
use pop3::extract::units::{export_unit_icons, Tribe as UnitTribe, UnitIconRequest};

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
        .subcommand(
            Command::new("unit-animations")
                .about("Export named unit animation atlases from the original sprite data")
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("DIR")
                        .value_parser(clap::value_parser!(PathBuf))
                        .required(true)
                        .help(
                            "Destination directory for animation atlases, previews, and manifest",
                        ),
                )
                .arg(
                    Arg::new("landscape")
                        .long("landscape")
                        .value_name("KEY")
                        .default_value("0")
                        .help("One-character landscape/palette bank key"),
                ),
        )
        .subcommand(
            Command::new("unit-icons")
                .about("Render named idle unit icons from the original sprite and animation data")
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
                        .help("Tribal palette and sprite set to render"),
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
        .subcommand(
            Command::new("building-panel-icons")
                .about("Extract the original construction-panel glyphs from hfx0-0.dat")
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("DIR")
                        .value_parser(clap::value_parser!(PathBuf))
                        .required(true)
                        .help("Destination directory for native-size icons, contact sheet, and manifest"),
                )
                .arg(
                    Arg::new("landscape")
                        .long("landscape")
                        .value_name("KEY")
                        .default_value("0")
                        .help("One-character landscape/palette bank key"),
                )
                .arg(
                    Arg::new("size")
                        .long("size")
                        .value_name("PIXELS")
                        .value_parser(clap::value_parser!(u32).range(48..=512))
                        .default_value("96")
                        .help("Width and height of each contact-sheet cell image"),
                ),
        )
        .subcommand(
            Command::new("hud-sprite-candidates")
                .about("Catalog indexed HSPR sprites while identifying native in-game HUD artwork")
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("DIR")
                        .value_parser(clap::value_parser!(PathBuf))
                        .required(true)
                        .help("Destination directory for indexed sprites, contact sheets, and manifest"),
                )
                .arg(
                    Arg::new("bank")
                        .long("bank")
                        .value_name("BANK")
                        .value_parser([
                            "primary",
                            "extension",
                            "hfx",
                            "hfx1",
                            "hspr1",
                            "hspr2",
                            "mspr",
                            "mspr-extension",
                            "point",
                            "point1",
                            "point2",
                            "panel",
                        ])
                        .default_value("primary")
                        .help("Sprite bank to inspect: HSPR/MSPR variants, POINT, or panel"),
                )
                .arg(
                    Arg::new("landscape")
                        .long("landscape")
                        .value_name("KEY")
                        .default_value("0")
                        .help("One-character palette bank key"),
                )
                .arg(
                    Arg::new("size")
                        .long("size")
                        .value_name("PIXELS")
                        .value_parser(clap::value_parser!(u32).range(48..=1024))
                        .default_value("96")
                        .help("Width and height of each contact-sheet cell image"),
                )
                .arg(
                    Arg::new("min-dimension")
                        .long("min-dimension")
                        .value_name("PIXELS")
                        .value_parser(clap::value_parser!(u16).range(1..=1024))
                        .default_value("12")
                        .help("Minimum source sprite width and height"),
                )
                .arg(
                    Arg::new("max-dimension")
                        .long("max-dimension")
                        .value_name("PIXELS")
                        .value_parser(clap::value_parser!(u16).range(1..=1024))
                        .default_value("64")
                        .help("Maximum source sprite width and height"),
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
            let tribe = StructureTribe::parse(tribe_value)
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
        Some(("unit-icons", args)) => {
            let tribe_value = args.get_one::<String>("tribe").expect("defaulted by clap");
            let tribe = UnitTribe::parse(tribe_value)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unsupported tribe"))?;
            let request = UnitIconRequest {
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
            let result = export_unit_icons(&request)?;
            println!("Extracted {} unit icons", result.icon_count);
            println!("Manifest: {}", result.manifest_path.display());
            println!("Contact sheet: {}", result.contact_sheet_path.display());
        }
        Some(("building-panel-icons", args)) => {
            let request = BuildingPanelIconRequest {
                base,
                output: args
                    .get_one::<PathBuf>("output")
                    .expect("required by clap")
                    .clone(),
                landscape: args
                    .get_one::<String>("landscape")
                    .expect("defaulted by clap")
                    .to_ascii_lowercase(),
                contact_sheet_size: *args.get_one::<u32>("size").expect("defaulted by clap"),
            };
            let result = export_building_panel_icons(&request)?;
            println!("Extracted {} building-panel icons", result.icon_count);
            println!("Manifest: {}", result.manifest_path.display());
            println!("Contact sheet: {}", result.contact_sheet_path.display());
        }
        Some(("unit-animations", args)) => {
            let request = UnitAnimationRequest {
                base,
                output: args
                    .get_one::<PathBuf>("output")
                    .expect("required by clap")
                    .clone(),
                landscape: args
                    .get_one::<String>("landscape")
                    .expect("defaulted by clap")
                    .to_ascii_lowercase(),
            };
            let result = export_unit_animations(&request)?;
            println!(
                "Extracted {} unit animation atlases",
                result.animation_count
            );
            println!("Manifest: {}", result.manifest_path.display());
            println!("Contact sheet: {}", result.contact_sheet_path.display());
        }
        Some(("hud-sprite-candidates", args)) => {
            let bank =
                HudSpriteBank::parse(args.get_one::<String>("bank").expect("defaulted by clap"))
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "unsupported bank")
                    })?;
            let request = HudSpriteCandidateRequest {
                base,
                output: args
                    .get_one::<PathBuf>("output")
                    .expect("required by clap")
                    .clone(),
                bank,
                landscape: args
                    .get_one::<String>("landscape")
                    .expect("defaulted by clap")
                    .to_ascii_lowercase(),
                size: *args.get_one::<u32>("size").expect("defaulted by clap"),
                min_dimension: *args
                    .get_one::<u16>("min-dimension")
                    .expect("defaulted by clap"),
                max_dimension: *args
                    .get_one::<u16>("max-dimension")
                    .expect("defaulted by clap"),
            };
            let result = export_hud_sprite_candidates(&request)?;
            println!("Extracted {} HUD sprite candidates", result.candidate_count);
            println!("Manifest: {}", result.manifest_path.display());
            println!("Contact sheets: {}", result.contact_sheet_paths.len());
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
