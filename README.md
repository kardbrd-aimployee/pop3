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
