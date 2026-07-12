// Live unit entity — a person unit with mutable position and movement state.

use super::animation::AnimationState;
use super::person_state::PersonState;
use crate::data::units::ModelType;
use crate::engine::movement::{PersonMovement, WorldCoord};
use crate::engine::objects::ObjectHandle;

pub type UnitId = usize;

pub struct Unit {
    pub id: UnitId,
    pub handle: ObjectHandle,
    pub model_type: ModelType,
    pub subtype: u8,
    pub tribe_index: u8,
    pub movement: PersonMovement,
    // Rendering cache — cell-space position, updated from world coords each tick.
    pub cell_x: f32,
    pub cell_y: f32,

    // Person state machine
    pub state: PersonState,
    pub prev_state: PersonState,
    pub state_timer: u16,  // countdown timer (ticks)
    pub state_counter: u8, // sub-counter / phase within state (offset 0x2D)

    // Combat stats (offsets 0x6C-0x7C in original)
    pub health: u16,                   // current HP (offset 0x6E)
    pub max_health: u16,               // max HP (offset 0x6C)
    pub target_unit: Option<UnitId>,   // combat target (offset 0x8A)
    pub attacker_unit: Option<UnitId>, // who's attacking us (offset 0x88)
    pub alive: bool,                   // false = dead/removed from game

    // Home/spawn position (offset 0x68/0x6A) — used for wander range
    pub home_pos: WorldCoord,
    // Behavior flags (offset 0x76)
    pub behavior_flags: u16,
    // Wander state (offsets 0x7B/0x7C)
    pub wander_duration: u8, // decrements each tick while wandering
    pub wander_range: u8,    // random walk range (subtype-dependent)
    // Linked object (offset 0x72) — vehicle, effect, etc.
    pub linked_obj_id: Option<UnitId>,
    // Combat modifiers
    pub bloodlust: bool, // bloodlust spell active — doubles damage
    pub shielded: bool,  // inside shield — halves incoming damage

    // Animation state (offsets +0x33..+0x3a in original binary)
    pub anim: AnimationState,

    // Building association — which building this person is entering/inside/exiting
    pub building_handle: Option<ObjectHandle>,
    // Wood being carried (for Gathering/CarryingWood states)
    pub wood_carried: u16,
    // Guard position — where this unit should hold (for Guard state)
    pub guard_position: Option<WorldCoord>,
    // Gather target — tree position for Gathering state navigation
    pub gather_target: Option<WorldCoord>,
}
