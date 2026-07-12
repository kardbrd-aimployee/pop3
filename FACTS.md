# Populous Gameplay Facts

This file records gameplay behavior confirmed by the project owner, an experienced player of the original game. Implementation plans should follow these facts. The open questions mark details that still need an answer.

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
- Physical hut occupancy has the same limit as its housing value: three people at stage one, four at stage two, and five at stage three.
- A hut becomes eligible for an upgrade after a period of time.
- Braves living in the hut collect the three additional wood pieces needed for the upgrade, one piece at a time.
- Once the hut is ready and has all three pieces, every occupant exits before renovation starts.
- The hut accepts no occupants while renovation is in progress.
- Braves may enter the hut again after renovation finishes.
- A stage-two hut houses four people.
- The stage-two hut follows the same readiness, wood collection, evacuation, and construction process for its next upgrade.
- A stage-three hut houses five people.

## Open questions

- What is the shared placement radius, and does the game measure it from the center or footprint edge?
- How many builders can work on each building type?
- What slope range counts as valid uneven terrain, and what slope blocks placement?
- How does the game break ties between plans with the same priority and similar distance?
- How long do partial and fully removed trees take to regrow?
- How long does each hut stage wait before becoming eligible for an upgrade?
- Does dismantling return wood, and can the player cancel a dismantling order?
