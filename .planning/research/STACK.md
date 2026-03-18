# Stack Research

**Domain:** Gameplay systems for Rust+wgpu game engine (audio, AI scripting, effects, serialization)
**Researched:** 2026-03-17
**Confidence:** MEDIUM-HIGH

## Context

Pop3 is an existing Rust+wgpu monolithic game engine reimplementing Populous: The Beginning. The rendering pipeline, pathfinding, unit movement, and data parsing are already built. This research covers the stack needed for the next layer: audio, AI scripting, visual effects, building state machines, and save/load.

**Key constraint:** The original game uses proprietary formats (SDT sound banks, SF2 SoundFont, custom bytecode scripts, fixed-layout binary saves). The stack must support low-level binary parsing and custom format handling, not just standard file formats.

## Recommended Stack

### Audio System

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| cpal | 0.17.3 | Low-level cross-platform audio output | Direct PCM submission needed for custom SDT format playback. Both Kira and rodio build on cpal. Using cpal directly gives full control over the mixing pipeline, which is essential for matching the original QSWaveMix behavior (linked-list channel management, manual distance attenuation, pan calculation). The original game does its own 3D audio math -- we must replicate that, not fight a higher-level library's spatial model. |
| rustysynth | 1.3.6 | SoundFont SF2 synthesizer | Pure Rust, zero dependencies, loads popfight.sf2 directly. Provides real-time synthesis from SF2 files which is exactly what the original game does for music via QSWaveMix MIDI sessions. |
| hound | 3.5.1 | WAV encoding/decoding | For any WAV-format samples within SDT banks. Lightweight, well-maintained, single-purpose. |

**Confidence: HIGH** -- cpal is the de facto standard for Rust audio I/O. rustysynth is the only pure-Rust SF2 synth with active maintenance. Versions verified via `cargo search`.

### Serialization / Save-Load

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| serde | 1.0.228 | Serialization framework | Industry standard. Derive macros for game state structs. Even though the original save format is fixed-layout binary (not self-describing), serde's trait system is useful for internal state snapshots and debug serialization. |
| bincode | 3.0.0 | Binary serialization | Fast, compact binary format for internal save states. Bincode 3 makes serde optional -- use the native bincode API for performance-critical paths, serde integration for convenience. |

**However:** The original SAVGAM format is a fixed 5016-byte header + level data at known offsets. This is NOT a serde use case -- it requires manual `std::io::Read`/`Write` with `bytemuck` (already in the project) for byte-level struct casting. Serde+bincode are for the _new_ internal save format, not for original format compatibility.

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| bytemuck | 1.x (existing) | Zero-copy byte casting | Already in project. Use for reading/writing fixed-layout original save format structs (SaveBuffer, tribe data, terrain). Perfect for the original's C-struct-to-file approach. |

**Confidence: HIGH** -- serde and bincode are universally recommended. The manual parsing approach for original formats aligns with how the project already handles DAT/HDR files.

### AI Scripting (Bytecode Interpreter)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| (none -- custom) | -- | Bytecode VM | The AI system is a custom bytecode interpreter with 200+ opcodes, not a standard scripting language. No external library is appropriate. Build a simple stack-based VM matching the original's `AI_ExecuteScriptCommand` dispatch table. This is a match-statement-on-opcode interpreter, not a general scripting engine. |

**Why NOT use external scripting:**
- **rhai/mlua/rlua** -- These are general-purpose scripting runtimes. The original game's AI uses a fixed bytecode format loaded from CPSCR files. We need to interpret those exact bytes, not design a new scripting language. Adding an external scripting runtime would be a large dependency for zero benefit.
- The bytecode format is documented in `docs/specs/ai_scripting.md` with full opcode tables.

**Confidence: HIGH** -- the bytecode format is fully reverse-engineered. A custom interpreter is the only correct approach.

### Visual Effects System

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| (none -- custom) | -- | Effect state machines | 93 effect types are all sprite-based, using the existing sprite rendering pipeline. Effects are game objects with position/velocity/lifetime managed by the engine tick loop. No particle library needed -- the original game doesn't use GPU particles, it uses sprite objects in a linked list with per-type state machines. |
| cgmath | 0.18.0 (existing) | Vector math for effect physics | Already in project. Velocity, gravity, angle calculations for projectiles and particles. |

**Why NOT use a particle library:**
- **bevy_particles/hanabi** -- These are GPU compute-based particle systems. The original game's effects are sprite objects tracked in a fixed-size pool (512 max), updated on the CPU each tick. Using GPU particles would be unfaithful and architecturally wrong.
- All 93 effect types have documented state machines. They are game objects, not GPU-spawned particles.

**Confidence: HIGH** -- the effect system is fully documented in `docs/specs/water_and_effects.md`.

### Building State Machines

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| (none -- custom) | -- | Building FSM | Buildings have a documented state machine (construction, occupied, damaged, destroyed). This is 5-10 states with explicit transitions. An enum + match statement is the correct Rust pattern. No state machine library needed. |

**Why NOT use a state machine library:**
- **statig/sm** -- Overkill for a handful of states with explicit transitions. The original game uses integer state fields with switch statements. A Rust enum with match arms is idiomatic, zero-overhead, and directly maps to the reverse-engineered code.

**Confidence: HIGH** -- standard Rust pattern for game state machines.

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0.228 | Derive macros for serializable game state | When implementing new save format (not original format) |
| serde_json | 1.x | Debug/config serialization | For debug dumps of game state, config files |
| bincode | 3.0.0 | Binary save format | For internal save/load (compact, fast) |
| cpal | 0.17.3 | Audio output | Always -- core audio backend |
| rustysynth | 1.3.6 | SF2 SoundFont synthesis | For music playback via popfight.sf2 |
| hound | 3.5.1 | WAV read/write | If SDT banks contain WAV-format samples |

## Installation

```bash
# Audio
cargo add cpal
cargo add rustysynth
cargo add hound

# Serialization
cargo add serde --features derive
cargo add bincode

# Optional: debug serialization
cargo add serde_json
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| cpal (low-level) | kira 0.12.0 | If you want high-level audio mixing with built-in tweening/effects. Kira is excellent for games that don't need to match specific legacy audio behavior. Not suitable here because we must replicate QSWaveMix's exact distance attenuation formula and channel management. |
| cpal (low-level) | rodio 0.22.2 | If you want simple "play this file" audio. Good for games without custom 3D audio. Same problem as kira -- its spatial model won't match the original's attenuation formula. |
| Custom bytecode VM | rhai 1.x | If you wanted to add modding support with a safe scripting language. Not appropriate for faithfully interpreting the original's CPSCR bytecode files. |
| Custom bytecode VM | mlua 0.10.x | If you wanted Lua scripting for modding. Same issue as rhai. |
| bytemuck (manual) | rkyv 0.8.x | If you wanted zero-copy deserialization for a new save format. Rkyv is fast but complex; bytemuck is simpler and already in the project for the fixed-layout original format. |
| bincode 3 | postcard 1.x | If you needed embedded-friendly serialization. Postcard is excellent for no_std, but bincode is more widely used and tested for game saves. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| bevy_audio / bevy_kira_audio | Bevy ECS dependency for a non-Bevy project. Pulls in the entire Bevy audio plugin system. | cpal directly |
| OpenAL (alto/openal-soft) | C library dependency, adds FFI complexity. The original game's audio is simple enough that a pure-Rust solution works. | cpal |
| GPU particle systems | The original game uses CPU-tracked sprite effects. GPU particles would be architecturally wrong and visually unfaithful. | Custom sprite-based effect system using existing renderer |
| General scripting engines (rhai, lua) | The AI bytecode format is fixed and documented. A general scripting engine adds complexity without solving the actual problem. | Custom bytecode interpreter |
| msgpack/protobuf/flatbuffers | Over-engineered for game saves. The original format is raw bytes; the new format just needs fast binary serialization. | bincode for new format, bytemuck for original format |

## Stack Patterns by System

**Audio System:**
- Use cpal for cross-platform audio output stream
- Parse SDT files manually (proprietary format, no crate exists)
- Feed raw PCM samples to cpal's output callback
- Implement distance attenuation and pan calculation matching original formulas
- Use rustysynth for SF2 music: load popfight.sf2, render MIDI to PCM, feed to cpal

**AI Scripting:**
- Load CPSCR binary files with existing file I/O
- Build opcode dispatch table as `match` on u16 opcode values
- Script state per tribe: instruction pointer, variable array, call stack
- Integrate into existing game tick loop at `AI_UpdateAllTribes` position

**Visual Effects:**
- Effects are game objects in the existing object pool
- Each of 93 types gets an init function and an update function
- Sprite rendering uses existing sprite pipeline (already handles 8-direction animation)
- Pool size: 512 max simultaneous effects (matching original)

**Save/Load:**
- Original format: manual byte-level read/write with bytemuck for struct casting
- New format (optional): serde + bincode for full game state snapshot
- Save buffer: 5016 bytes header + variable level data
- Key: serialize entire game tick state deterministically

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| cpal 0.17.3 | rustysynth 1.3.6 | rustysynth outputs raw f32 PCM; feed directly to cpal output stream |
| bincode 3.0.0 | serde 1.0.228 | Enable `serde` feature on bincode 3 to use serde derives |
| bytemuck 1.x | wgpu 28.0 | Already compatible in current project |
| cpal 0.17.3 | macOS / Linux | Uses CoreAudio on macOS, ALSA/PulseAudio on Linux. Nix flake may need `alsa-lib` added. |

## Platform Notes

**macOS (current dev):**
- cpal uses CoreAudio -- no additional dependencies needed
- Audio latency is good out of the box

**Linux (Nix):**
- cpal needs ALSA headers: add `alsa-lib` and `pkg-config` to flake.nix devShell
- Alternative: `cpal` also supports PulseAudio and JACK backends via feature flags

**Cross-platform:**
- All recommended crates are pure Rust or use platform-native APIs through cpal
- No C/C++ build dependencies except ALSA headers on Linux

## Sources

- [cpal on crates.io](https://crates.io/crates/cpal) -- version 0.17.3 verified via cargo search (HIGH confidence)
- [kira on crates.io](https://crates.io/crates/kira) -- version 0.12.0 verified via cargo search (HIGH confidence)
- [rodio on crates.io](https://github.com/RustAudio/rodio) -- version 0.22.2 verified via cargo search (HIGH confidence)
- [rustysynth on crates.io](https://crates.io/crates/rustysynth) -- version 1.3.6 verified via cargo search (HIGH confidence)
- [bincode on crates.io](https://crates.io/crates/bincode) -- version 3.0.0 verified via cargo search (HIGH confidence)
- [serde on crates.io](https://serde.rs/) -- version 1.0.228 verified via cargo search (HIGH confidence)
- [hound on GitHub](https://github.com/ruuda/hound) -- version 3.5.1 verified via cargo search (HIGH confidence)
- [Are We Game Yet - Audio](https://arewegameyet.rs/ecosystem/audio/) -- ecosystem overview (MEDIUM confidence)
- [Rust Audio Programming 2025](https://andrewodendaal.com/rust-audio-programming-ecosystem/) -- ecosystem comparison (MEDIUM confidence)
- `docs/specs/audio.md` -- original game audio system reverse engineering (HIGH confidence)
- `docs/specs/ai_scripting.md` -- original game AI bytecode format (HIGH confidence)
- `docs/specs/water_and_effects.md` -- original game effect system (HIGH confidence)
- `docs/specs/level_save_network.md` -- original game save format (HIGH confidence)

---
*Stack research for: Pop3 gameplay systems (audio, AI, effects, serialization)*
*Researched: 2026-03-17*
