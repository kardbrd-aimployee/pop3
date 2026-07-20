# Pop3

Pop3 is an open-source project that aims to remake [Populous: The Beginning](https://en.wikipedia.org/wiki/Populous:_The_Beginning) using modern technologies, staying as close to the original as practically possible.

Video demo [here](https://www.youtube.com/watch?v=cE2oCt4YUz0)

<img width="1920" height="1080" alt="Screenshot 2026-03-09 at 11 32 22 PM (2)" src="https://github.com/user-attachments/assets/97226697-69b2-4492-8e6a-7813456d6e63" />

Built with [Rust](https://www.rust-lang.org/), using [wgpu](https://wgpu.rs/) for cross-platform GPU rendering (Metal/Vulkan/DX12), Pop3 is a standalone application but requires original game files as proof of ownership.

We welcome contributions from anyone interested in preserving and improving this classic game.

## Disclaimer

Pop3 is a fan-made, open-source project. It is not affiliated with or endorsed by Bullfrog Productions or Electronic Arts. You will need a copy of the original Populous: The Beginning game files as proof of ownership. These can be obtained from [GOG.com](https://www.gog.com/) or original CDs. All trademarks and copyrights are the property of their respective owners.

## Features

- 3D landscape rendering with multiple texture modes (full GPU, CPU/GPU hybrid, full CPU)
- Toroidal world wrapping with optional curvature distortion
- Water animation and sunlight simulation
- Panoramic sky rendering from original palette data
- 3D building and tree rendering from original game meshes
- Sprite-based unit rendering with 8-direction animation system
- 25 levels supported

## Project direction

- [Design constraints and hard blockers](DESIGN.md)
- [Completion roadmap and acceptance criteria](docs/plans/game-completion-roadmap.md)
- [Reverse-engineering specifications](docs/specs/index.md)

## Building

Requires the [Rust toolchain](https://rustup.rs/).

```bash
cargo build --release
```

Key dependencies: [wgpu](https://wgpu.rs/) (GPU rendering), [winit](https://github.com/rust-windowing/winit) (windowing), [cgmath](https://github.com/rustgd/cgmath) (math), [clap](https://github.com/clap-rs/clap) (CLI).

## Usage

All executables require `--base` pointing to your Populous: The Beginning game data directory.

### pop3 — Main renderer

A wgpu-based 3D renderer for viewing Populous levels. Reads original game files and renders them using modern graphics APIs.

```bash
cargo run --release -- --base /path/to/pop3 --level 1
```

| Option | Description |
|--------|-------------|
| `--level N` | Start at level N (1-255) |
| `--cpu` | CPU/GPU hybrid texture rendering |
| `--cpu-full` | Full CPU texture rendering |
| `--light X;Y` | Sunlight parameters |
| `--debug` | Enable debug logging |
| `--script PATH` | Replay key events from script file |

Scripts contain one command per line; blank lines and lines beginning with `#`
are ignored. Alongside `wait`, `click`, `rightclick`, `zoom`, and `screenshot`,
the renderer provides deterministic capture commands:

- `construct_hut` places the nearest valid blue small-hut plan, assigns one
  brave, and pauses the simulation.
- `advance_hut_phase N` advances that hut deterministically to phase `0..4`
  and pauses exactly at the requested threshold.
- `center_cell X Y` centers the camera on a landscape cell without key-repeat
  drift.
- `unit_gallery ACTION` arranges Brave, Warrior, Preacher, Spy, Firewarrior,
  and Shaman fixtures using the exact native animation row for `idle`, `walk`,
  `action`, `die`, `celebrate`, `chop`, `swim`, `carry`, `dig`, `build`, or
  `run`.

For a repeatable Level 1 construction sequence, run:

```bash
cargo run --release -- \
  --base /path/to/pop3 \
  --level 1 \
  --script scripts/capture_hut_construction.txt
```

To capture the implemented unit animation rows in the live renderer:

```bash
cargo run --release -- \
  --base /path/to/pop3 \
  --level 1 \
  --script scripts/capture_unit_animations.txt
```

| Key | Action |
|-----|--------|
| WASD | Pan terrain |
| Q / E | Rotate camera |
| Up / Down | Tilt camera |
| Mouse wheel | Zoom |
| B / V | Next / Previous level |
| N / M | Next / Previous shader |
| Space | Center on shaman spawn |
| C | Toggle curvature |
| O | Toggle object markers |
| Escape | Quit |

### unit_viewer — Unit animation viewer

Browse unit animations from the VELE/VFRA/VSTART animation chain.

```bash
cargo run --release --bin unit_viewer -- --base /path/to/pop3 --anim 15
```

| Option | Description |
|--------|-------------|
| `--anim N` | Start at animation index N (default: 15 = Brave Idle) |
| `--tribe N` | Start with tribe N (0-3) |

| Key | Action |
|-----|--------|
| N / P | Next / Previous animation |
| + / - | Jump 10 animations |
| T | Cycle tribe (Blue/Red/Yellow/Green) |
| U | Cycle unit features |
| Space | Pause / Resume |
| Up / Down | Animation speed |
| Left / Right | Frame step (when paused) |
| Q / E | Rotate |
| Escape | Quit |

### sprite_viewer — Sprite animation viewer

Browse sprite atlases and directional animations.

```bash
cargo run --release --bin sprite_viewer -- --base /path/to/pop3
```

| Key | Action |
|-----|--------|
| Tab / N / P | Switch character |
| Space | Pause / Resume |
| Up / Down | Animation speed |
| Left / Right | Frame step (when paused) |
| Q / E | Rotate |
| Escape | Quit |

### sky_viewer — Sky texture viewer

Browse sky variants across landscape types.

```bash
cargo run --release --bin sky_viewer -- --base /path/to/pop3
```

| Key | Action |
|-----|--------|
| N / P or Left / Right | Next / Previous sky |
| Q / E | Scroll horizontally |
| Escape | Quit |

### pop_obj_view — 3D object viewer

Inspect individual game 3D models (buildings, trees, etc.).

```bash
cargo run --release --bin pop_obj_view -- --base /path/to/pop3 --obj_num 0
```

| Option | Description |
|--------|-------------|
| `--obj_num N` | Object index (0-based) |
| `--landtype TYPE` | Override landscape type |

| Key | Action |
|-----|--------|
| Arrow keys | Rotate object |
| V / B | Previous / Next object |
| N / M | Scale down / up |
| R | Reset scale |
| Escape | Quit |

### pop_res — Resource extraction tool

CLI tool for extracting and converting game resources to images.

```bash
cargo run --release --bin pop_res -- globe 1 --base /path/to/pop3
```

Available subcommands: `globe`, `land`, `minimap`, `water`, `bl320`, `bl160`, `bigf0`, `disp`, `palette`, `objects`, `units`, `anims`, `anims_draw`, `pls`, `psfb`.

See `scripts/` for usage examples.

### pop_extract — Named original-data catalog

`pop_extract` turns original game resources into stable, named assets with a
machine-readable manifest. Unlike the low-level `pop_res` decoder, each command
combines the source files needed for a gameplay concept.

```bash
cargo run --release --bin pop_extract -- \
  --base /path/to/pop3 \
  structure-icons \
  --output data/extracted/structure-icons \
  --landscape 0 \
  --tribe blue
```

The structure-icon catalog renders the three visual families of every hut stage
and the seven other player construction structures from the original OBJS mesh,
palette, and BL320 texture data. It writes transparent PNG files under `icons/`,
a labeled `contact-sheet.png`, and `manifest.json` containing subtype, tribe,
visual variant, source object index, and mesh counts. Planned catalog families
include raw/construction-phase meshes, landscape textures, and person animations.

To extract the idle unit catalog for one tribe:

```bash
cargo run --release --bin pop_extract -- \
  --base /path/to/pop3 \
  unit-icons \
  --output data/extracted/unit-icons \
  --landscape 0 \
  --tribe blue
```

To extract the original construction-panel glyphs, use the HFX element-parameter
catalog. Unlike `structure-icons`, these are the exact images passed by the
original build-menu code rather than newly rendered 3D building previews:

```bash
cargo run --release --bin pop_extract -- \
  --base /path/to/pop3 \
  building-panel-icons \
  --output data/extracted/building-panel-icons \
  --landscape 0
```

The output contains native-size transparent PNG pairs for the nine active
construction cells: normal and highlighted. The manifest records their
canonical building subtypes and original `hfx0-0.dat` image numbers
`1028, 1029, 1030, 1032, 1033, 1031, 1034, 1035, 1036` in native menu order;
the highlighted companion is always the source image number plus 18. It also
exports the original state-4 `?` overlay (`1055`) as
`icons/blocked-overlay.png`; this is a state layer, not a replacement icon.

To inspect other native in-game HUD artwork, generate an indexed catalog from
the complete primary HSPR bank:

```bash
cargo run --release --bin pop_extract -- \
  --base /path/to/pop3 \
  hud-sprite-candidates \
  --output data/extracted/hud-sprite-candidates \
  --bank primary \
  --landscape 0
```

Use `--bank extension` to inspect `HSPR0-1.DAT` / `HSPR0-1.TAB`. Every PNG
filename and manifest item retains the original HSPR sprite index.

The unit-icon catalog composes the original sprite bank and animation tables for
Braves, Warriors, Preachers, Spies, and Firewarriors, and uses the direct tribal
Shaman sprites. It writes transparent PNG files under `icons/`, a labeled contact
sheet, and a manifest recording animation IDs, sprite-layer combinations, and
source files.

To extract the full named unit animation catalog for the main rewrite:

```bash
cargo run --release --bin pop_extract -- \
  --base /path/to/pop3 \
  unit-animations \
  --output data/extracted/unit-animations \
  --landscape 0
```

This writes one atlas per unit animation under `animations/<unit>/`. Each atlas
uses the renderer-compatible layout of four tribes by five stored directions by
animation frame; the manifest records the original animation ID, VSTART base,
frame size, frame count, compositing layer, and atlas path. The renderer can
mirror the two omitted display directions using the layout metadata. The catalog
includes idle, walk, work, carry, dig, build, combat, transport, sitting, and
special sequences, plus direct Shaman idle and walk sprites.

## Project structure

```
src/
  main.rs           CLI entry point
  data/             Binary format parsers (levels, units, objects, sprites, animations)
  engine/           Game logic — simulation, movement, units (no GPU dependency)
    command.rs      Input → GameCommand translation
    frame.rs        Per-frame output boundary for rendering
    state/          Game simulation (tick loop, flags, RNG, tribes, victory)
    movement/       Pathfinding and unit movement
    units/          Unit state machines and selection
  render/           Everything visual — meshes, GPU, HUD, camera, geometry
    app.rs          Application struct, render loop, winit integration
    terrain.rs      Landscape mesh generation
    camera.rs       Camera / MVP matrices
    buildings.rs    3D building mesh construction
    sprites/        Sprite-based unit rendering
    hud/            UI overlay rendering
    gpu/            wgpu abstraction (context, pipeline, buffer, texture)
    geometry/       Procedural mesh generation (cube, sphere, circle)
  bin/              Standalone viewer executables
shaders/            WGSL shaders
scripts/            Testing and resource extraction tools
docs/               Documentation and reverse engineering notes
```

## Reverse engineering

Reverse engineering notes for the original game binary are available in [docs/specs/](docs/specs/index.md). RE work is done using Ghidra with [ghidra-mcp](https://github.com/bethington/ghidra-mcp) for AI-assisted analysis.

## Contributing

We welcome contributions from the community to improve and expand Pop3.

- Report bugs by opening [issues](https://github.com/SkinyMonkey/pop3/issues)
- Submit feature requests and discuss potential improvements
- Contribute code by creating pull requests
- Help with reverse engineering the original game binary

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
