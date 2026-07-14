# Original Gameplay Visual References

These crops come from the project owner's recording of a legally owned copy of
*Populous: The Beginning*. They are retained as implementation references, not
runtime assets.

- `pop3-original-native-hud.png`: left-side minimap, tabs, status block,
  population bar, and building grid.
- `pop3-original-shore.png`: stippled land texture overlapping animated water.
- `pop3-original-workers-entrance.png`: workers regrouping outside a completed
  hut entrance.
- `original-hut-construction/`: indexed construction timing and phase frames.

Current implementation targets captured by these references:

- Only the building tab is active now; the spell and follower tab silhouettes
  remain inert until those systems are implemented.
- Zero-height shore cells retain animated water, with adjacent land texture
  stippled over the surface rather than a static raised shore strip.
- Once construction finishes, workers will rendezvous outside the entrance in
  groups capped at six; exact slots and idle behavior follow occupancy work.
