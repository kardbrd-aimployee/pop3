# Unit animation checklists

The six checklists map shared person states to subtype-specific logical animation IDs and original-game frames.

| Unit | Native subtype | Extracted sequences | Checklist |
|---|---:|---:|---|
| Brave | 2 | 24 | [Brave](brave.md) |
| Warrior | 3 | 23 | [Warrior](warrior.md) |
| Preacher | 4 | 23 | [Preacher](preacher.md) |
| Spy | 5 | 23 | [Spy](spy.md) |
| Firewarrior | 6 | 24 | [Firewarrior](firewarrior.md) |
| Shaman | 7 | 12 | [Shaman](shaman.md) |

Read the [shared person-state contract](../person-state-animation-checklist.md) for state values, construction subphases, and acceptance rules.

The extracted strips prove sprite identity and frame order. Each checklist keeps gameplay boxes open until an original-game capture and a Rust capture agree on timing, render type, motion overlay, and state transition.
