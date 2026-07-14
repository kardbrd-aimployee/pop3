# Lua Modding and Campaign Scripting Research

## Decision in brief

Pop3 should make campaign scripting a first-class engine feature. It should
support **two complementary paths**:

1. A native, deterministic interpreter for original `CPSCR` and `CPATR` data,
   which remains the compatibility path for the original campaign.
2. An optional Lua extension layer for new campaigns, community mods, richer
   objectives, and experimental rules.

Lua must not replace the original AI bytecode. It gives new content an
expressive, supported API without creating a second incompatible definition of
the original campaign.

## What the community demonstrates

The active Populous community treats a campaign as more than a map. Multiverse
campaign packs bundle maps, scripts, AI files, and metadata, while its mod
catalogue contains narrated campaigns, limited reincarnation, smarter AI,
Swamp damage-over-time, spy sabotage, extra tribes, and alternative fog rules.
That is strong evidence that campaign behavior, presentation, rules, and
progression need an explicit extension boundary rather than source forks.

Multiverse's Lua model has a persistent `main.lua` parent script and
level-scoped child scripts. It exposes level-load and turn callbacks, plus
object, spell, damage, input, drawing, save/load, audio, and menu hooks. This
is a useful feature model, but its binary-patching API also exposes mutable raw
object pointers and globals. Pop3 should adopt the capability, not those unsafe
memory semantics.

## Pop3 architecture to build

`GameSession` already owns the simulation tick and its FIFO `GameAction` queue.
That is the right host boundary. Add an `engine::scripting` subsystem that:

- receives a read-only world view and queued domain events;
- runs native campaign-script and optional Lua callbacks at a fixed simulation
  phase;
- returns typed `ScriptCommand` values;
- converts accepted commands into the same validated action path used by the
  player and AI;
- stores serializable script state with the authoritative world save.

The fixed order should be explicit and stable:

```text
previous-tick events -> CPSCR/Lua callbacks -> validated actions
-> simulation systems -> new events -> frame snapshot
```

Scripts therefore observe a defined world state and cannot make direct,
mid-system mutations. Player, native-AI, and Lua commands should all carry a
source and deterministic ordering rule.

## Lua API v1

Start with a small, stable API designed around gameplay rather than renderer or
memory layout details.

| Area | Initial capability |
| --- | --- |
| Lifecycle | `on_campaign_load`, `on_level_start`, `on_tick`, `on_save`, `on_load`, `on_level_end` |
| Events | entity created/destroyed, spell cast, damage, objective transition |
| Queries | opaque entity handles, player/tribe state, terrain, markers, nearby entities |
| Commands | issue orders, spawn valid entities, grant unlocks, change alliances, cast spells, modify terrain through spell/rule systems |
| Campaign UX | objectives, messages, scripted camera, fog reveal/cover, narrator and music cues |
| Rules | per-campaign data overrides for values such as reincarnation limits and damage-over-time |

Use generational `ObjectHandle`-style identifiers, never exposed pointers. A
stale script reference must fail cleanly instead of resolving to a recycled
game object.

Keep gameplay and presentation separate: a gameplay command changes canonical
state; a cosmetic command requests an effect, sound, message, or camera cue.
Lua must not write GPU state or use render-frame callbacks for gameplay.

## Determinism and safety requirements

- Run Lua only on simulation ticks, never on wall-clock time or frame rate.
- Provide engine-owned seeded random functions; do not expose `os`, filesystem,
  network, processes, or unrestricted standard-library I/O.
- Enforce instruction/time and allocation budgets per callback; report campaign,
  script, event, and line context on failure.
- Make script state a constrained, versioned, JSON-like value tree rather than
  serializing arbitrary Lua VM internals.
- Include script source/content hashes and API version in saves and replay
  metadata.
- Run headless golden scenarios: load a campaign fixture, tick N times, compare
  action trace and world digest, save/load, then continue identically.

## Campaign packaging

Use an explicit package layout, independent of copyrighted base assets:

```text
campaign.toml
levels/
scripts/main.lua
scripts/level_001.lua
assets/
localization/
```

The manifest should declare API version, title, campaign/level metadata,
optional Lua entry points, required Pop3 version, content hashes, and asset
dependencies. Original-game data remains a user-supplied base; a campaign pack
contains only its own derivative content and metadata.

## Delivery sequence

1. Complete the authoritative world/action/event boundary first.
2. Parse and execute original Level 1 CPSCR/CPATR data natively.
3. Add the restricted Lua runtime with `on_level_start`, `on_tick`, query
   functions, commands, deterministic RNG, and save/load state.
4. Build one Lua-authored Level 1 variation with an objective, event message,
   rule tweak, and scripted AI command.
5. Add package validation, a headless test runner, diagnostics, and a small
   documented example campaign before expanding the API.

This sequencing preserves original-campaign accuracy while making the engine
pleasant to build worlds for as soon as the core simulation is ready.

## Research sources

- [Multiverse Lua quick start](https://toksisitee.github.io/docs/mv-script-intro)
- [Multiverse callback events](https://toksisitee.github.io/docs/mv-api/mv-events)
- [Multiverse scripting API](https://toksisitee.github.io/docs/mv-api/mv-functions)
- [Community campaign pack format](https://thebeginning.uk/campaign-packs/)
- [Community mod catalogue](https://thebeginning.uk/multiverse-mods/)
