# Populous Gameplay Facts

This file records gameplay behavior confirmed by the project owner, an experienced player of the original game, plus facts verified against the legally owned original data and executable. Implementation plans should follow these facts. The open questions mark details that still need an answer.

## Input and selection

- Left-clicking a person selects that person.
- Holding the left mouse button and dragging across the ground selects people inside the drag area.
- The cursor shows the number of selected people.
- Left-clicking a destination sends the selected people there. Left-clicking a building plan assigns them to its construction.
- A direct player order interrupts the selected braves' current work and replaces it with the new order.
- Right-clicking clears the current person selection.

## Building placement controls

- Selecting a building attaches its placement footprint to the cursor.
- The footprint uses a white ground overlay and an arrow that shows the entrance direction.
- Pressing Space rotates the building clockwise so the player can choose the entrance direction.
- An ordinary left click places one building plan and exits placement mode.
- Shift + Left Click places a building plan and keeps the same building attached to the cursor, allowing the player to place more plans.
- A placed plan reserves its footprint before construction starts.
- Shift + Right Click on an unbuilt plan removes the plan and releases its footprint.
- Shift + Right Click on a completed building orders workers to dismantle it.

## Placement validity

- Another building or reserved plan can block part of the footprint.
- Terrain can block part of the footprint when its slope exceeds the allowed construction slope.
- The placement overlay marks blocked parts red while leaving valid parts white.
- When no completed friendly building lies within placement range, the entire footprint turns red.
- Builders can construct on slightly uneven terrain. They flatten it before or during the first construction work.
- Builders cannot construct on steep terrain.
- Buildings require another friendly building within a placement radius.
- Only completed friendly buildings extend the placement radius.
- Unbuilt plans do not extend the placement radius, so the player cannot chain plans outward from the settlement.
- The watchtower ignores the building-proximity requirement.
- The player can place a watchtower anywhere on the map without a nearby friendly building, provided the ground is dry and flat enough.
- A completed watchtower extends the normal placement radius for other buildings.
- Players use remote watchtowers to establish new settlements when an existing settlement lacks space or nearby resources.
- A watchtower occupies one ground square and shows its entrance on that square.
- Water blocks watchtower placement. The placement overlay shows the invalid terrain.

## Assigning builders

- The player can select braves and left-click the plan to assign them.
- One brave can complete a building alone.
- Each building type limits the number of braves who can work on it at once.
- Idle braves claim unstaffed building plans without a direct player order.
- Direct player assignments override automatic work priorities.
- Idle braves choose work in this priority order:
  1. Watchtowers
  2. Other buildings
  3. Huts

## Provisional remake decisions

- An idle brave chooses the closest available building plan first.
- When candidate plans have comparable travel distance, the brave uses the construction priority: watchtower, other building, then hut.
- Gameplay verification may change the distance and priority weighting.
- Use one placement radius for every completed friendly building and every normal building type. Gameplay verification should confirm the remembered original-game behavior and determine the exact distance.

## Construction and terrain work

- Braves flatten valid uneven ground by jumping on the building footprint.
- Braves fetch construction wood from trees.
- A brave walks to a tree, chops one piece of wood, carries it to the plan, and contributes it to construction.
- Several assigned braves can split the work. Some can flatten the footprint while others collect wood.

## Trees and wood

- Each full tree contains four pieces of wood.
- One brave removes and carries one piece at a time.
- The tree becomes smaller each time a brave removes a piece.
- A partially chopped tree grows its missing wood back over time.
- Removing all four pieces makes the tree disappear.
- A fully removed tree returns after a longer regrowth period.

## Huts and housing

- Each tribe has three hut stages.
- The player places a generic hut plan. The construction scaffold reveals the initial hut form during construction.
- A stage-one hut costs three pieces of wood and houses three people.
- Idle braves do not enter huts during normal idling.
- A brave enters a hut when that brave built it, spawned there, or worked on its renovation.
- The tribe's follower capacity comes from its huts and their current stages.
- Huts spawn followers while the tribe's follower count remains below its total capacity.
- A hut's follower-spawn progress runs faster when more people occupy that hut.
- A newly spawned brave exits through the hut entrance and idles near it.
- After some time, that brave may enter the hut if it has a free occupant slot.
- When the hut is full, the brave remains outside and idles near the hut.
- A brave counts as an occupant only while physically inside the hut. Braves standing outside do not consume occupant slots.
- Physical hut occupancy has the same limit as its housing value: three people at stage one, five at stage two, and seven at stage three.
- A hut becomes eligible for an upgrade after a period of time.
- Braves living in the hut collect the three additional wood pieces needed for the upgrade, one piece at a time.
- Once the hut is ready and has all three pieces, every occupant exits before renovation starts.
- The hut accepts no occupants while renovation is in progress.
- Braves may enter the hut again after renovation finishes.
- A stage-two hut houses five people.
- The stage-two hut follows the same readiness, wood collection, evacuation, and construction process for its next upgrade.
- A stage-three hut houses seven people.

## Original-game verification

- `LEVELS/constant.dat` is XOR-obfuscated text. The original loader decrypts it at `0x0041EB50` and converts percent values to 8.8 fixed point with integer truncation.
- `constant.dat` sets hut wood costs to `300/300/300`. A carried piece contributes `100`, so initial construction and each renovation require three pieces.
- `constant.dat` sets hut housing limits to `3/5/7`.
- `constant.dat` sets base follower-growth thresholds to `4000/3000/2000` for hut stages one through three.
- `Building_UpdatePopGrowth` at `0x00430020` adds `2 * (occupants + 1)` to a hut's follower-growth progress on each update.
- `Building_CalcPopGrowthRate` at `0x00426220` selects one of 20 population bands. Braves contribute weight 15; warriors, preachers, spies, and firewarriors contribute weight 4; shamans and wild people do not contribute. The band percentages run from 30 through 200 percent and are applied with the loader's 8.8 fixed-point conversion.
- Hut stages one and two both use a renovation-readiness threshold of `2400`. Their readiness counter advances by `8 * occupant_count` per update, so an empty hut does not advance toward renovation.
- The stage-one type record upgrades to stage two, and the stage-two record upgrades to stage three. Stage three has no next subtype.
- `Building_UpdateWoodConsumption` at `0x00430430` creates the next hut subtype and destroys the old hut when renovation begins. The final completed shape is applied by `Building_OnConstructionComplete` at `0x0042FD70`.
- Completed hut assets are separate models: OBJS indices 145, 146, and 147 for the blue tribe, with three consecutive indices per tribe. Construction and renovation visuals must therefore remain distinct from the final hut models.
- Computer-player construction runs through `AI_ExecuteBuildingPriorities` at `0x0041B8D0`. It has ten command slots, evaluates twelve building-priority handlers, orders their candidate records by urgency, and dispatches work through an available command slot.
- `AI_CommandBuildHut` at `0x00448360` is a multi-state command. It retains the selected plan/object, revalidates that object and its position, and returns to plan selection when the target disappears or becomes invalid.

## Open questions

- What is the shared placement radius, and does the game measure it from the center or footprint edge?
- How many builders can work on each building type?
- What slope range counts as valid uneven terrain, and what slope blocks placement?
- How does the game break ties between plans with the same priority and similar distance?
- How long do partial and fully removed trees take to regrow?
- Does dismantling return wood, and can the player cancel a dismantling order?
