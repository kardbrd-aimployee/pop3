# Pop3 — Populous: The Beginning Remake

## What This Is

Pop3 is an open-source faithful reimplementation of Populous: The Beginning in Rust, using wgpu for cross-platform GPU rendering (Metal/Vulkan/DX12). It reads original game data files and aims to reproduce the original game's behavior as closely as possible, documented through Ghidra reverse engineering of popTB.exe. The project targets anyone who wants to play this classic RTS/god-game on modern hardware.

## Core Value

Faithful reproduction of the original Populous: The Beginning gameplay experience on modern platforms — every system must match the original binary's behavior, verified against reverse-engineered specs and Frida-captured ground truth data.

## Requirements

### Validated

- ✓ Binary format parsing (DAT, HDR, PSFB containers) — existing
- ✓ Level loading (25 levels, heightmap, units, tribes, sunlight) — existing
- ✓ 3D landscape rendering with toroidal wrapping and curvature — existing
- ✓ Water animation and sunlight simulation — existing
- ✓ Sky rendering from original palette data — existing
- ✓ 3D building mesh rendering with terrain-following — existing
- ✓ Sprite-based unit rendering with 8-direction animation — existing
- ✓ Camera orbit, zoom, pan controls — existing
- ✓ Unit selection (click and drag-box) — existing
- ✓ 4-tier pathfinding system (Region, Segment, Failure, Bug2) — existing
- ✓ Formation movement (up to 12 followers) — existing
- ✓ Game tick loop matching original binary's subsystem order — existing
- ✓ Person state machine (idle, move, goto, wander, attack basics) — existing
- ✓ Game speed controls — existing
- ✓ Walkability overlay with building footprints — existing
- ✓ Multiple terrain shader variants — existing
- ✓ Deterministic RNG matching original binary — existing

### Active

- [ ] Core Object System — unified object pool with 11 model types, cell-based spatial grid
- [ ] Building System — state machine, construction, occupants, population growth, damage/destruction
- [ ] Spell System — 21 spells with mana economy, cooldowns, targeting
- [ ] Combat System — melee damage, projectiles, knockback, building combat
- [ ] AI/Scripting System — bytecode interpreter, 200+ opcodes, personality traits, decision making
- [ ] Terrain System — height modification, cell flags, dynamic water levels
- [ ] Effect System — 93 visual effect types (spell, environmental, particle, building)
- [ ] Audio System — 3D positional audio, SFX, music via SoundFont
- [ ] HUD/UI — minimap, spell bar, info panels, mana/population display
- [ ] Menu System — main menu, campaign, options, save/load
- [ ] Save/Load System — full game state serialization
- [ ] Creature System — 3 creature types with AI
- [ ] Vehicle System — boats and airships
- [ ] Victory/Defeat conditions — per-mode win/loss checks
- [ ] Campaign progression — 25-level campaign with unlocks

### Out of Scope

- Network/Multiplayer — extremely complex, requires full game state sync; defer to post-v1
- Multi-language support (CJK) — focus on English first
- Custom key bindings — defer to post-v1
- Modding support — not in original game scope

## Context

- All game systems are documented from Ghidra RE of popTB.exe in `docs/specs/`
- Implementation status tracked in `things-to-implement.md` (comprehensive inventory)
- Test fixtures captured from original binary via Frida instrumentation in `tests/pathfinding_fixtures/`
- Architecture: 3-layer (data/engine/render) with clean boundaries (GameCommand input, FrameState output)
- `app.rs` is the largest file at 3296 lines — contains main render loop and GameEngine
- The project has 260 tests across 24 files, primarily in engine/movement subsystem
- Reverse engineering uses Ghidra with ghidra-mcp for AI-assisted analysis

## Constraints

- **Tech stack**: Rust + wgpu + winit — established, no changes
- **Game data**: Requires original POP3 game files as proof of ownership (legal constraint)
- **Faithfulness**: All reimplemented systems must match original binary behavior — verified via address annotations and fixture testing
- **No runtime dependencies**: Fully offline, local file I/O only
- **Single crate**: No workspace split — keep monolithic structure

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| wgpu over raw Vulkan/Metal | Cross-platform with single API, WebGPU standard | ✓ Good |
| Sprites for units, 3D for buildings | Matches original game's hybrid renderer | ✓ Good |
| 4-tier pathfinding faithful to binary | Ensures movement matches original exactly | ✓ Good |
| GameCommand enum as input boundary | Clean decoupling of input from engine | ✓ Good |
| FrameState as output boundary | No GPU types leak into engine layer | ✓ Good |
| Monolithic app.rs | Started organic, may need refactoring | ⚠️ Revisit |
| Lua scripting for AI instead of bytecode VM | Original uses proprietary bytecode (200+ opcodes) but community has documented Lua equivalents; Lua is more maintainable and extensible | — Pending |
| Modern object storage instead of faithful pool replica | Original's two-tier free list was a 1998 single-threaded optimization; use Rust-idiomatic arenas/slotmaps for better perf and parallelism potential | — Pending |

---
*Last updated: 2026-03-17 after initialization*
