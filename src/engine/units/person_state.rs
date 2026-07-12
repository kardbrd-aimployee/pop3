// Person state machine — faithful to Person_SetState (0x004fd5d0).
//
// State values are stored at object offset 0x2C in the original binary.
// All 44 values are defined for binary compatibility, but only core states
// (Idle, Moving, Wander, GoToPoint, Fighting, Fleeing, Drowning, Dead)
// have real implementations in this phase.

use super::unit::Unit;
use crate::engine::movement::WorldCoord;
use crate::engine::objects::ObjectHandle;
use crate::engine::state::rng::GameRng;

/// All person states from the original binary's Person_SetState switch.
/// Values match offset 0x2C exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PersonState {
    Idle = 0x01,
    Dying = 0x02,
    Moving = 0x03,
    Wander = 0x04,
    GoToPoint = 0x05,
    FollowPath = 0x06,
    GoToMarker = 0x07,
    WaitForPath = 0x08,
    WaitAtMarker = 0x09,
    EnterBuilding = 0x0A,
    InsideBuilding = 0x0B,
    InsideTraining = 0x0C,
    Building = 0x0D,
    InTraining = 0x0E,
    WaitOutside = 0x0F,
    Training = 0x10,
    Housing = 0x11,
    // 0x12 unused
    Gathering = 0x13,
    // 0x14 unused
    GatheringWood = 0x15,
    CarryingWood = 0x16,
    Drowning = 0x17,
    Dead = 0x18,
    Fighting = 0x19,
    Fleeing = 0x1A,
    Spawning = 0x1B,
    BeingSacrificed = 0x1C,
    InShield = 0x1D,
    InShieldIdle = 0x1E,
    Preaching = 0x1F,
    SitDown = 0x20,
    BeingConverted = 0x21,
    WaitingAfterConvert = 0x22,
    WaitingForBoat = 0x23,
    Placeholder = 0x24,
    GetOffBoat = 0x25,
    WaitingInWater = 0x26,
    EnteringVehicle = 0x27,
    ExitingVehicle = 0x28,
    Celebrating = 0x29,
    Teleporting = 0x2A,
    InternalState = 0x2B,
    WaitingAtReincPillar = 0x2C,
}

impl Default for PersonState {
    fn default() -> Self {
        PersonState::Idle
    }
}

/// Default stats per person subtype.
/// From the Unit Type Data Table at 0x0059FE44 (stride 0x32).
pub struct PersonTypeDefaults {
    pub max_health: u16,
    pub speed: u16,
    pub fight_damage: u16,
}

/// Returns default stats for a given person subtype.
/// Max health values extracted from binary at 0x0059FE50 + subtype * 0x32.
/// Speed values from 0x5A0974 (stride 26).
pub fn person_type_defaults(subtype: u8) -> PersonTypeDefaults {
    match subtype {
        1 => PersonTypeDefaults {
            max_health: 32,
            speed: 0x30,
            fight_damage: 64,
        }, // Wild
        2 => PersonTypeDefaults {
            max_health: 1400,
            speed: 0x30,
            fight_damage: 200,
        }, // Brave
        3 => PersonTypeDefaults {
            max_health: 1800,
            speed: 0x28,
            fight_damage: 400,
        }, // Warrior
        4 => PersonTypeDefaults {
            max_health: 1400,
            speed: 0x28,
            fight_damage: 150,
        }, // Religious
        5 => PersonTypeDefaults {
            max_health: 1400,
            speed: 0x30,
            fight_damage: 200,
        }, // Spy
        6 => PersonTypeDefaults {
            max_health: 1200,
            speed: 0x28,
            fight_damage: 500,
        }, // SuperWarrior
        7 => PersonTypeDefaults {
            max_health: 900,
            speed: 0x28,
            fight_damage: 300,
        }, // Shaman
        8 => PersonTypeDefaults {
            max_health: 2000,
            speed: 0x30,
            fight_damage: 600,
        }, // Angel of Death
        _ => PersonTypeDefaults {
            max_health: 200,
            speed: 0x30,
            fight_damage: 100,
        }, // Fallback
    }
}

// --- State entry ---

/// Enter a new state, saving the previous state and running entry logic.
/// Mirrors the preamble + switch of Person_SetState (0x004fd5d0).
pub fn enter_state(unit: &mut Unit, new_state: PersonState, rng: &mut GameRng) {
    log::debug!(
        "[state] unit {} {:?} → {:?}",
        unit.id,
        unit.state,
        new_state
    );
    unit.prev_state = unit.state;
    unit.state = new_state;
    unit.state_counter = 0;

    // Common flag clearing (matches original's preamble):
    // flags1 &= 0xFCDEFDDD — clears MOVING, BLOCKED, and various control bits
    unit.movement.flags1 &= 0xFCDE_FDDD;

    match new_state {
        PersonState::Idle => enter_idle(unit, rng),
        PersonState::Wander => enter_wander(unit, rng),
        PersonState::Moving => { /* movement system handles entry */ }
        PersonState::GoToPoint | PersonState::GoToMarker => { /* state_goto called separately */ }
        PersonState::Fighting => enter_fighting(unit),
        PersonState::Fleeing => enter_fleeing(unit, rng),
        PersonState::Drowning => enter_drowning(unit),
        PersonState::Dead => enter_dead(unit, rng),
        PersonState::EnterBuilding => enter_enter_building(unit),
        PersonState::Housing => enter_housing(unit),
        PersonState::Training | PersonState::InTraining | PersonState::InsideTraining => {
            enter_training(unit)
        }
        PersonState::WaitOutside => enter_wait_outside(unit),
        PersonState::Gathering => enter_gathering(unit),
        PersonState::GatheringWood => enter_gathering_wood(unit),
        PersonState::CarryingWood => enter_carrying_wood(unit),
        PersonState::InsideBuilding => { /* no special entry logic */ }
        // Guard behavior uses guard_position field, managed by coordinator when unit is at a GuardPost
        _ => { /* Unimplemented states — no-op */ }
    }
}

/// Idle: speed=0, random timer 50-100 ticks.
/// Original: case '\x01' in Person_SetState.
fn enter_idle(unit: &mut Unit, rng: &mut GameRng) {
    unit.movement.speed = 0;
    unit.state_timer = (rng.next() % 50 + 50) as u16;
}

/// Wander sub-phases stored in `state_counter`.
/// Original: Person_ProcessIdleWanderState uses phase byte at +0x2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WanderPhase {
    /// Walking in random direction (32-63 ticks).
    Walking = 0,
    /// Paused/idle, looking around (64-127 ticks).
    Pausing = 1,
    /// Second walking phase (32-63 ticks).
    Walking2 = 2,
    /// Water escape — pathfind away from water, or idle (32-63 ticks).
    WaterEscape = 3,
}

/// Wander: start in Walking phase with random direction.
/// Original: case '\x04' in Person_SetState.
fn enter_wander(unit: &mut Unit, rng: &mut GameRng) {
    unit.state_counter = WanderPhase::Walking as u8;
    enter_wander_walking(unit, rng);
}

/// Set up a walking sub-phase: random direction, timer 32-63 ticks, MOVING flag.
fn enter_wander_walking(unit: &mut Unit, rng: &mut GameRng) {
    unit.state_timer = ((rng.next() & 0x1F) + 0x20) as u16; // 32-63
    let angle = (rng.next() & 0x7FF) as u16;
    unit.movement.facing_angle = angle;
    unit.movement.flags1 |= 0x1080;
    let defaults = person_type_defaults(unit.subtype);
    unit.movement.speed = defaults.speed;
}

/// Set up a pausing sub-phase: stop moving, timer 64-127 ticks.
fn enter_wander_pausing(unit: &mut Unit, rng: &mut GameRng) {
    unit.state_timer = ((rng.next() & 0x3F) + 0x40) as u16; // 64-127
    unit.movement.flags1 &= !0x1000; // Stop moving
    unit.movement.speed = 0;
}

/// Fighting: enter Seek phase, stop moving.
/// Original: case '\x19' → Person_EnterFightingState (0x00437b40).
fn enter_fighting(unit: &mut Unit) {
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // Stop moving
    unit.state_counter = CombatPhase::Seek as u8;
    unit.state_timer = 0;
}

/// Fleeing: random direction, speed=0x6E, timer=0x40.
/// Original: case '\x1a' in Person_SetState.
fn enter_fleeing(unit: &mut Unit, rng: &mut GameRng) {
    unit.movement.speed = 0x6E; // Flee speed (faster than normal)
    unit.state_timer = 0x40; // 64 ticks
    let angle = (rng.next() & 0x7FF) as u16;
    unit.movement.facing_angle = angle;
    // Set MOVING and BLOCKED flags for flee movement
    unit.movement.flags1 |= 0x1080;
}

/// Drowning: set drowning flags.
/// Original: case '\x17' → Person_EnterDrowningState (0x00503190).
fn enter_drowning(unit: &mut Unit) {
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // Stop moving
}

/// Dead: speed=0, set dead flags, random counter 0-7.
/// Original: case '\x18' in Person_SetState.
fn enter_dead(unit: &mut Unit, rng: &mut GameRng) {
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // Stop moving
                                     // Original: flags1 |= 0x480, flags2 |= 0x4000
    unit.movement.flags1 |= 0x480;
    unit.state_counter = (rng.next() & 7) as u8;
}

// --- Per-tick state update ---

/// Result of a single tick_state call.
pub enum TickResult {
    /// Stay in current state.
    Continue,
    /// Transition to a new state.
    Transition(PersonState),
}

/// Actions that the caller (coordinator) must execute after tick_state returns.
/// Enables building/resource interaction without borrow checker conflicts:
/// tick_state only has &mut Unit, but building data lives in the pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeferredAction {
    /// No deferred action needed.
    None,
    /// Person arrived at building — coordinator should call add_occupant.
    AddToBuilding {
        person: ObjectHandle,
        building: ObjectHandle,
    },
    /// Person exiting building — coordinator should call remove_occupant.
    RemoveFromBuilding {
        person: ObjectHandle,
        building: ObjectHandle,
    },
    /// Person depositing wood at building.
    DepositWood { building: ObjectHandle, amount: u16 },
    /// Person spawning at building exit (after training).
    SpawnAtBuilding { building: ObjectHandle },
    /// Person needs nearest tree position for wood gathering navigation.
    FindNearestTree { unit_index: usize },
}

/// Per-tick state update for a single unit.
/// Called each game tick from the coordinator.
/// Returns (TickResult, DeferredAction) — the coordinator must execute any
/// deferred action after processing the tick result.
pub fn tick_state(unit: &mut Unit, rng: &mut GameRng) -> (TickResult, DeferredAction) {
    match unit.state {
        PersonState::Idle => (tick_idle(unit), DeferredAction::None),
        PersonState::Moving | PersonState::GoToPoint | PersonState::GoToMarker => {
            (tick_moving(unit), DeferredAction::None)
        }
        PersonState::Wander => (tick_wander(unit, rng), DeferredAction::None),
        PersonState::Fighting => (tick_fighting(unit), DeferredAction::None),
        PersonState::Fleeing => (tick_fleeing(unit), DeferredAction::None),
        PersonState::Drowning => (tick_drowning(unit), DeferredAction::None),
        PersonState::Dead => (tick_dead(unit), DeferredAction::None),
        PersonState::EnterBuilding => tick_enter_building(unit),
        PersonState::Housing => tick_housing(unit),
        PersonState::Training | PersonState::InTraining | PersonState::InsideTraining => {
            tick_training(unit)
        }
        PersonState::WaitOutside => tick_wait_outside(unit),
        PersonState::Gathering => tick_gathering(unit),
        PersonState::GatheringWood => tick_gathering_wood(unit),
        PersonState::CarryingWood => tick_carrying_wood(unit),
        PersonState::InsideBuilding => (tick_inside_building(unit), DeferredAction::None),
        _ => (TickResult::Continue, DeferredAction::None), // Unimplemented states hold
    }
}

/// Idle: countdown state_timer; on expiry, look up default state from unit type table.
/// Original: Object_ProcessPersonState case 0x01 — decrements timer,
/// on expiry looks up default state from DAT_0059fe44[subtype*0x32].
/// For most subtypes the default state is 0 (= no transition, stay idle).
/// Wander is NOT triggered by idle timer — it's an AI/command-driven state.
fn tick_idle(unit: &mut Unit) -> TickResult {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
    }
    TickResult::Continue
}

/// Moving/GoToPoint/GoToMarker: check if movement completed.
/// Movement itself is handled by the existing movement system in coordinator.tick().
fn tick_moving(unit: &mut Unit) -> TickResult {
    if !unit.movement.is_moving() {
        TickResult::Transition(PersonState::Idle)
    } else {
        TickResult::Continue
    }
}

/// Wander: cycle through walking/pausing sub-phases.
/// Original: Person_ProcessIdleWanderState — phase 0=walk, 1=pause, 2=walk, then → Idle.
/// Movement (walking in random direction) is handled by the coordinator.
fn tick_wander(unit: &mut Unit, rng: &mut GameRng) -> TickResult {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
        return TickResult::Continue;
    }

    match unit.state_counter {
        0 => {
            // Walking → Pausing
            unit.state_counter = WanderPhase::Pausing as u8;
            enter_wander_pausing(unit, rng);
            TickResult::Continue
        }
        1 => {
            // Pausing → Walking2
            unit.state_counter = WanderPhase::Walking2 as u8;
            enter_wander_walking(unit, rng);
            TickResult::Continue
        }
        2 => {
            // Walking2 → back to Idle
            unit.movement.flags1 &= !0x1000;
            unit.movement.speed = 0;
            TickResult::Transition(PersonState::Idle)
        }
        _ => {
            // WaterEscape or unknown → Idle
            unit.movement.flags1 &= !0x1000;
            unit.movement.speed = 0;
            TickResult::Transition(PersonState::Idle)
        }
    }
}

/// Fighting: advance through combat sub-phases.
/// Actual damage application and chase movement happen in the coordinator
/// (needs access to both units). This manages the phase state machine.
///
/// Phase flow: Seek → Approach → SwingReady → Strike → LungeBack → LungeFwd → Recovering → Seek
pub fn tick_fighting(unit: &mut Unit) -> TickResult {
    if unit.target_unit.is_none() {
        return TickResult::Transition(PersonState::Idle);
    }

    let phase = CombatPhase::from_counter(unit.state_counter);
    match phase {
        CombatPhase::Seek => {
            // Coordinator will set Approach phase when target is detected
            // (via process_combat chase logic)
            TickResult::Continue
        }
        CombatPhase::Approach => {
            // Coordinator handles movement toward target.
            // When in melee range, coordinator sets SwingReady phase.
            TickResult::Continue
        }
        CombatPhase::SwingReady => {
            // Pre-strike pause
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                unit.state_counter = CombatPhase::Strike as u8;
                TickResult::Continue
            }
        }
        CombatPhase::Strike => {
            // Damage is applied by coordinator when it sees Strike phase.
            // Immediately transition to LungeBack.
            unit.state_counter = CombatPhase::LungeBack as u8;
            unit.state_timer = LUNGE_TICKS;
            TickResult::Continue
        }
        CombatPhase::LungeBack => {
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                unit.state_counter = CombatPhase::LungeFwd as u8;
                unit.state_timer = LUNGE_TICKS;
                TickResult::Continue
            }
        }
        CombatPhase::LungeFwd => {
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                unit.state_counter = CombatPhase::Recovering as u8;
                unit.state_timer = RECOVERING_TICKS;
                TickResult::Continue
            }
        }
        CombatPhase::Recovering => {
            if unit.state_timer > 0 {
                unit.state_timer -= 1;
                TickResult::Continue
            } else {
                // Back to Seek for next attack cycle
                unit.state_counter = CombatPhase::Seek as u8;
                TickResult::Continue
            }
        }
    }
}

/// Fleeing: countdown timer, transition to Idle when expired.
fn tick_fleeing(unit: &mut Unit) -> TickResult {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
        TickResult::Continue
    } else {
        unit.movement.flags1 &= !0x1000;
        TickResult::Transition(PersonState::Idle)
    }
}

/// Drowning: lose health each tick, die when health reaches 0.
fn tick_drowning(unit: &mut Unit) -> TickResult {
    // Lose ~2% of max_health per tick (matches original's gradual drowning)
    let damage = (unit.max_health / 50).max(1);
    if unit.health <= damage {
        unit.health = 0;
        TickResult::Transition(PersonState::Dead)
    } else {
        unit.health -= damage;
        TickResult::Continue
    }
}

/// Dead: countdown state_counter, mark not alive when done.
fn tick_dead(unit: &mut Unit) -> TickResult {
    if unit.state_counter > 0 {
        unit.state_counter -= 1;
    } else {
        unit.alive = false;
    }
    TickResult::Continue
}

// --- Building/economy state handlers ---

/// EnterBuilding enter: building_handle should already be set by the command that triggered this state.
fn enter_enter_building(unit: &mut Unit) {
    unit.state_timer = 0;
}

/// EnterBuilding tick: walk toward building position; when arrived, transition to InsideBuilding.
fn tick_enter_building(unit: &mut Unit) -> (TickResult, DeferredAction) {
    unit.state_timer += 1;
    if unit.state_timer >= 30 {
        let action = if let Some(bh) = unit.building_handle {
            DeferredAction::AddToBuilding {
                person: unit.handle,
                building: bh,
            }
        } else {
            DeferredAction::None
        };
        (TickResult::Transition(PersonState::InsideBuilding), action)
    } else {
        (TickResult::Continue, DeferredAction::None)
    }
}

/// InsideBuilding tick: coordinator decides whether to transition to Housing or Training
/// based on building type. Stays here until coordinator transitions.
fn tick_inside_building(_unit: &mut Unit) -> TickResult {
    TickResult::Continue
}

/// Housing enter: person is now housed, contributes to population count.
fn enter_housing(unit: &mut Unit) {
    unit.state_timer = 0;
}

/// Housing tick: stay indefinitely until ejected by building destruction or player command.
fn tick_housing(_unit: &mut Unit) -> (TickResult, DeferredAction) {
    (TickResult::Continue, DeferredAction::None)
}

/// Training enter: start conversion countdown timer.
/// Default: 256 ticks for warrior conversion; coordinator may set a different value before entry.
fn enter_training(unit: &mut Unit) {
    if unit.state_timer == 0 {
        unit.state_timer = 256;
    }
}

/// Training tick: decrement timer; when done, signal state complete for coordinator
/// to change person subtype and transition to WaitOutside.
fn tick_training(unit: &mut Unit) -> (TickResult, DeferredAction) {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
    }
    if unit.state_timer == 0 {
        let action = if let Some(bh) = unit.building_handle {
            DeferredAction::SpawnAtBuilding { building: bh }
        } else {
            DeferredAction::None
        };
        (TickResult::Transition(PersonState::WaitOutside), action)
    } else {
        (TickResult::Continue, DeferredAction::None)
    }
}

/// WaitOutside enter: clear building_handle, person walks away from building.
fn enter_wait_outside(unit: &mut Unit) {
    let _prev_building = unit.building_handle;
    unit.building_handle = None;
    unit.state_timer = 0;
}

/// WaitOutside tick: walk away from building; when clear, transition to Idle.
fn tick_wait_outside(unit: &mut Unit) -> (TickResult, DeferredAction) {
    unit.state_timer += 1;
    if unit.state_timer >= 20 {
        (
            TickResult::Transition(PersonState::Idle),
            DeferredAction::None,
        )
    } else {
        (TickResult::Continue, DeferredAction::None)
    }
}

/// Gathering enter: clear wood carried, request tree target via deferred action.
/// state_timer: 0 = waiting for tree target, 1 = navigating to tree.
fn enter_gathering(unit: &mut Unit) {
    unit.state_timer = 0; // need tree target
    unit.wood_carried = 0;
    unit.gather_target = None;
    unit.movement.speed = 0;
    unit.movement.flags1 &= !0x1000; // stop moving until we have a target
}

/// Gathering tick: navigate toward tree target; transition to GatheringWood on arrival.
/// state_timer=0: emit FindNearestTree deferred action (coordinator sets gather_target + state_timer=1).
/// state_timer=1: walk toward gather_target; when within 128 world units, transition.
fn tick_gathering(unit: &mut Unit) -> (TickResult, DeferredAction) {
    if unit.state_timer == 0 {
        // Need a tree target — ask coordinator
        return (
            TickResult::Continue,
            DeferredAction::FindNearestTree {
                unit_index: unit.id,
            },
        );
    }

    // state_timer == 1: navigating toward tree
    if let Some(target) = unit.gather_target {
        let dx = (target.x as i32 - unit.movement.position.x as i32);
        let dz = (target.z as i32 - unit.movement.position.z as i32);
        let dist = dx.abs() + dz.abs();

        if dist < 128 {
            // Arrived at tree — transition to GatheringWood
            unit.movement.speed = 0;
            unit.movement.flags1 &= !0x1000;
            return (
                TickResult::Transition(PersonState::GatheringWood),
                DeferredAction::None,
            );
        }

        // Move toward tree: step of ~4 world units per axis per tick (matching brave walk speed)
        let step = 4i32;
        if dx.abs() > 0 {
            unit.movement.position.x += (dx.signum() * step.min(dx.abs())) as i16;
        }
        if dz.abs() > 0 {
            unit.movement.position.z += (dz.signum() * step.min(dz.abs())) as i16;
        }
    } else {
        // Had state_timer=1 but no target (shouldn't happen) — re-request
        unit.state_timer = 0;
    }

    (TickResult::Continue, DeferredAction::None)
}

/// GatheringWood enter: start chop timer (60 ticks).
fn enter_gathering_wood(unit: &mut Unit) {
    unit.state_timer = 60;
}

/// GatheringWood tick: decrement chop timer; when done, pick up wood and transition to CarryingWood.
fn tick_gathering_wood(unit: &mut Unit) -> (TickResult, DeferredAction) {
    if unit.state_timer > 0 {
        unit.state_timer -= 1;
    }
    if unit.state_timer == 0 {
        unit.wood_carried = 1;
        (
            TickResult::Transition(PersonState::CarryingWood),
            DeferredAction::None,
        )
    } else {
        (TickResult::Continue, DeferredAction::None)
    }
}

/// CarryingWood enter: set timer for walk to building.
fn enter_carrying_wood(unit: &mut Unit) {
    unit.state_timer = 0;
}

/// CarryingWood tick: walk toward building; when arrived, deposit wood and loop back to Gathering.
fn tick_carrying_wood(unit: &mut Unit) -> (TickResult, DeferredAction) {
    unit.state_timer += 1;
    if unit.state_timer >= 40 {
        let amount = unit.wood_carried;
        unit.wood_carried = 0;
        let action = if let Some(bh) = unit.building_handle {
            DeferredAction::DepositWood {
                building: bh,
                amount,
            }
        } else {
            DeferredAction::None
        };
        (TickResult::Transition(PersonState::Gathering), action)
    } else {
        (TickResult::Continue, DeferredAction::None)
    }
}

// --- Combat helpers ---

/// Calculate melee damage from attacker to defender.
/// Original: Combat_ProcessMeleeDamage (0x004c5d20).
/// damage = (fight_damage * health) / max_health, minimum 32.
/// Bloodlust doubles the damage output.
pub fn calculate_melee_damage(attacker: &Unit) -> u16 {
    let defaults = person_type_defaults(attacker.subtype);
    let base = defaults.fight_damage as u32;
    let mut damage = (base * attacker.health as u32) / attacker.max_health.max(1) as u32;
    if attacker.bloodlust {
        damage *= 2;
    }
    damage.max(0x20) as u16 // Min damage = 32
}

/// Apply damage to a unit, accounting for shield protection.
/// Original: Object_ApplyDamage (0x00504f20).
/// Shield halves incoming damage (right-shift by 1).
pub fn apply_damage(unit: &mut Unit, damage: u16) {
    let effective = if unit.shielded { damage >> 1 } else { damage };
    if unit.health <= effective {
        unit.health = 0;
    } else {
        unit.health -= effective;
    }
}

/// Detection range for combat (world coordinate units).
/// Units within this distance will engage each other.
pub const COMBAT_DETECT_RANGE: i32 = 512;

/// Melee attack range (world coordinate units).
/// Units must be this close to deal damage.
pub const COMBAT_MELEE_RANGE: i32 = 72;

/// Ticks between melee attacks (used as fallback; sub-phases have own timers).
pub const COMBAT_ATTACK_INTERVAL: u16 = 8;

/// Combat sub-phases stored in `state_counter` (offset 0x2D).
/// Original: Person_ProcessCombatState uses phase byte to drive micro-states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CombatPhase {
    /// Find/validate target, set initial approach direction.
    Seek = 0x00,
    /// Walk toward enemy (facing updated each tick by coordinator).
    Approach = 0x22,
    /// Pre-strike pause (0x10 = 16 ticks).
    SwingReady = 0x07,
    /// Deliver damage this tick, then transition to lunge/recovery.
    Strike = 0x26,
    /// Post-strike lunge backward (cosmetic, random angle offset).
    LungeBack = 0x27,
    /// Post-strike lunge forward.
    LungeFwd = 0x28,
    /// Post-attack cooldown (8 ticks), then back to Seek.
    Recovering = 0x0C,
}

/// Timer for SwingReady phase (original: 0x10 ticks).
pub const SWING_READY_TICKS: u16 = 0x10;
/// Timer for Recovering phase (original: 8 ticks).
pub const RECOVERING_TICKS: u16 = 8;
/// Timer for lunge phases (original: ~4 ticks each).
pub const LUNGE_TICKS: u16 = 4;

impl CombatPhase {
    pub fn from_counter(val: u8) -> Self {
        match val {
            0x22 => CombatPhase::Approach,
            0x07 => CombatPhase::SwingReady,
            0x26 => CombatPhase::Strike,
            0x27 => CombatPhase::LungeBack,
            0x28 => CombatPhase::LungeFwd,
            0x0C => CombatPhase::Recovering,
            _ => CombatPhase::Seek,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn h(slot: u16) -> ObjectHandle {
        ObjectHandle::new(slot, 1)
    }
    use crate::data::units::ModelType;
    use crate::engine::movement::PersonMovement;

    fn make_unit(subtype: u8, tribe: u8) -> Unit {
        use crate::engine::movement::WorldCoord;
        use crate::engine::units::animation::AnimationState;
        let defaults = person_type_defaults(subtype);
        Unit {
            id: 0,
            handle: ObjectHandle::new(0, 1),
            model_type: ModelType::Person,
            subtype,
            tribe_index: tribe,
            movement: PersonMovement::default(),
            cell_x: 0.0,
            cell_y: 0.0,
            state: PersonState::Idle,
            prev_state: PersonState::Idle,
            state_timer: 0,
            state_counter: 0,
            health: defaults.max_health,
            max_health: defaults.max_health,
            target_unit: None,
            attacker_unit: None,
            alive: true,
            home_pos: WorldCoord::new(0, 0),
            behavior_flags: 0,
            wander_duration: 0,
            wander_range: 0,
            linked_obj_id: None,
            bloodlust: false,
            shielded: false,
            anim: AnimationState::default(),
            building_handle: None,
            wood_carried: 0,
            guard_position: None,
            gather_target: None,
        }
    }

    #[test]
    fn person_state_enum_values() {
        assert_eq!(PersonState::Idle as u8, 0x01);
        assert_eq!(PersonState::Moving as u8, 0x03);
        assert_eq!(PersonState::Wander as u8, 0x04);
        assert_eq!(PersonState::GoToPoint as u8, 0x05);
        assert_eq!(PersonState::GoToMarker as u8, 0x07);
        assert_eq!(PersonState::Drowning as u8, 0x17);
        assert_eq!(PersonState::Dead as u8, 0x18);
        assert_eq!(PersonState::Fighting as u8, 0x19);
        assert_eq!(PersonState::Fleeing as u8, 0x1A);
        assert_eq!(PersonState::Celebrating as u8, 0x29);
        assert_eq!(PersonState::WaitingAtReincPillar as u8, 0x2C);
    }

    #[test]
    fn person_type_defaults_health() {
        assert_eq!(person_type_defaults(1).max_health, 32); // Wild
        assert_eq!(person_type_defaults(2).max_health, 1400); // Brave
        assert_eq!(person_type_defaults(3).max_health, 1800); // Warrior
        assert_eq!(person_type_defaults(7).max_health, 900); // Shaman
    }

    #[test]
    fn enter_idle_sets_timer_and_zero_speed() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Idle, &mut rng);
        assert_eq!(unit.state, PersonState::Idle);
        assert_eq!(unit.movement.speed, 0);
        assert!(unit.state_timer >= 50 && unit.state_timer <= 99);
    }

    #[test]
    fn enter_wander_sets_flags_and_direction() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Wander, &mut rng);
        assert_eq!(unit.state, PersonState::Wander);
        assert!(unit.state_timer >= 32 && unit.state_timer <= 95);
        assert!(unit.movement.facing_angle <= 2047);
        assert!(unit.movement.flags1 & 0x1000 != 0); // MOVING set
        assert_eq!(unit.movement.speed, 0x30); // Brave speed
    }

    #[test]
    fn enter_dead_sets_flags() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Dead, &mut rng);
        assert_eq!(unit.state, PersonState::Dead);
        assert_eq!(unit.movement.speed, 0);
        assert!(unit.movement.flags1 & 0x480 != 0);
        assert!(unit.state_counter <= 7);
    }

    #[test]
    fn enter_fleeing_sets_speed_and_timer() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Fleeing, &mut rng);
        assert_eq!(unit.state, PersonState::Fleeing);
        assert_eq!(unit.movement.speed, 0x6E);
        assert_eq!(unit.state_timer, 0x40);
        assert!(unit.movement.flags1 & 0x1000 != 0); // MOVING set
    }

    #[test]
    fn idle_counts_down_and_stays_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Idle;
        unit.state_timer = 2;
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_timer, 1);
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_timer, 0);
        // Timer expired — unit stays idle (default state for brave is 0 = no transition)
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state, PersonState::Idle);
    }

    #[test]
    fn wander_sub_phases_walk_pause_walk_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Wander, &mut rng);
        assert_eq!(unit.state_counter, WanderPhase::Walking as u8);
        assert!(unit.state_timer >= 32 && unit.state_timer <= 63);
        assert!(unit.movement.flags1 & 0x1000 != 0); // MOVING

        // Drain walking timer
        while unit.state_timer > 0 {
            assert!(matches!(
                tick_state(&mut unit, &mut rng),
                (TickResult::Continue, _)
            ));
        }
        // Timer=0, should transition to Pausing
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_counter, WanderPhase::Pausing as u8);
        assert!(unit.state_timer >= 64 && unit.state_timer <= 127);
        assert_eq!(unit.movement.flags1 & 0x1000, 0); // NOT moving

        // Drain pausing timer
        while unit.state_timer > 0 {
            assert!(matches!(
                tick_state(&mut unit, &mut rng),
                (TickResult::Continue, _)
            ));
        }
        // Timer=0, should transition to Walking2
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_counter, WanderPhase::Walking2 as u8);
        assert!(unit.state_timer >= 32 && unit.state_timer <= 63);

        // Drain walking2 timer
        while unit.state_timer > 0 {
            assert!(matches!(
                tick_state(&mut unit, &mut rng),
                (TickResult::Continue, _)
            ));
        }
        // Timer=0, should transition to Idle
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Transition(PersonState::Idle), _)
        ));
    }

    #[test]
    fn moving_to_idle_when_arrived() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::GoToPoint;
        unit.movement.flags1 |= 0x1000; // Still moving
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        unit.movement.flags1 &= !0x1000; // Arrived
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Transition(PersonState::Idle), _)
        ));
    }

    #[test]
    fn drowning_drains_health_to_death() {
        let mut unit = make_unit(2, 0); // Brave, HP=1400
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Drowning;
        let initial_hp = unit.health;
        match tick_state(&mut unit, &mut rng) {
            (TickResult::Continue, _) => {}
            _ => panic!("Should continue"),
        }
        assert!(unit.health < initial_hp);
        for _ in 0..200 {
            if let (TickResult::Transition(PersonState::Dead), _) = tick_state(&mut unit, &mut rng)
            {
                assert_eq!(unit.health, 0);
                return;
            }
        }
        panic!("Should have transitioned to Dead");
    }

    #[test]
    fn dead_counts_down_then_not_alive() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Dead;
        unit.state_counter = 3;
        unit.alive = true;
        for _ in 0..3 {
            tick_state(&mut unit, &mut rng);
            assert!(unit.alive);
        }
        tick_state(&mut unit, &mut rng);
        assert!(!unit.alive);
    }

    #[test]
    fn fleeing_counts_down_to_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Fleeing;
        unit.state_timer = 2;
        unit.movement.flags1 |= 0x1000;
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        match tick_state(&mut unit, &mut rng) {
            (TickResult::Transition(PersonState::Idle), _) => {}
            _ => panic!("Expected Idle transition"),
        }
    }

    #[test]
    fn fighting_without_target_goes_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(99);
        unit.state = PersonState::Fighting;
        unit.target_unit = None;
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Transition(PersonState::Idle), _)
        ));
    }

    #[test]
    fn combat_phase_cycle() {
        let mut unit = make_unit(3, 0); // Warrior
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Fighting, &mut rng);
        unit.target_unit = Some(1);
        assert_eq!(unit.state_counter, CombatPhase::Seek as u8);

        // Seek stays in Seek (coordinator drives Seek→Approach)
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));

        // Simulate coordinator setting SwingReady
        unit.state_counter = CombatPhase::SwingReady as u8;
        unit.state_timer = 2;
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        )); // timer 2→1
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        )); // timer 1→0
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        )); // → Strike
        assert_eq!(unit.state_counter, CombatPhase::Strike as u8);

        // Strike → LungeBack
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_counter, CombatPhase::LungeBack as u8);
        assert_eq!(unit.state_timer, LUNGE_TICKS);

        // Drain LungeBack
        for _ in 0..LUNGE_TICKS {
            assert!(matches!(
                tick_state(&mut unit, &mut rng),
                (TickResult::Continue, _)
            ));
        }
        // → LungeFwd
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_counter, CombatPhase::LungeFwd as u8);

        // Drain LungeFwd
        for _ in 0..LUNGE_TICKS {
            assert!(matches!(
                tick_state(&mut unit, &mut rng),
                (TickResult::Continue, _)
            ));
        }
        // → Recovering
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_counter, CombatPhase::Recovering as u8);
        assert_eq!(unit.state_timer, RECOVERING_TICKS);

        // Drain Recovering
        for _ in 0..RECOVERING_TICKS {
            assert!(matches!(
                tick_state(&mut unit, &mut rng),
                (TickResult::Continue, _)
            ));
        }
        // → Seek
        assert!(matches!(
            tick_state(&mut unit, &mut rng),
            (TickResult::Continue, _)
        ));
        assert_eq!(unit.state_counter, CombatPhase::Seek as u8);
    }

    #[test]
    fn calculate_melee_damage_scales_with_health() {
        let mut unit = make_unit(3, 0); // Warrior, fight_damage=400
                                        // Full health: damage = 400 * 1800 / 1800 = 400
        assert_eq!(calculate_melee_damage(&unit), 400);
        // Half health: damage = 400 * 900 / 1800 = 200
        unit.health = 900;
        assert_eq!(calculate_melee_damage(&unit), 200);
        // Very low health: damage = 400 * 10 / 1800 = 2 → clamped to 32
        unit.health = 10;
        assert_eq!(calculate_melee_damage(&unit), 32); // min 0x20
    }

    #[test]
    fn apply_damage_clamps_to_zero() {
        let mut unit = make_unit(2, 0);
        unit.health = 100;
        apply_damage(&mut unit, 50);
        assert_eq!(unit.health, 50);
        apply_damage(&mut unit, 200); // More than remaining
        assert_eq!(unit.health, 0);
    }

    #[test]
    fn bloodlust_doubles_damage() {
        let mut unit = make_unit(3, 0); // Warrior, fight_damage=400
        assert_eq!(calculate_melee_damage(&unit), 400);
        unit.bloodlust = true;
        assert_eq!(calculate_melee_damage(&unit), 800);
    }

    #[test]
    fn shield_halves_incoming_damage() {
        let mut unit = make_unit(2, 0);
        unit.health = 200;
        unit.shielded = true;
        apply_damage(&mut unit, 100); // 100 >> 1 = 50
        assert_eq!(unit.health, 150);
    }

    #[test]
    fn prev_state_saved_on_transition() {
        let mut unit = make_unit(2, 0);
        unit.state = PersonState::Idle;
        let mut rng = GameRng::new(1);
        enter_state(&mut unit, PersonState::Wander, &mut rng);
        assert_eq!(unit.prev_state, PersonState::Idle);
        assert_eq!(unit.state, PersonState::Wander);
    }

    // --- New state handler tests (02-04) ---

    #[test]
    fn enter_building_transitions_to_inside_building() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.building_handle = Some(h(5));
        enter_state(&mut unit, PersonState::EnterBuilding, &mut rng);
        assert_eq!(unit.state, PersonState::EnterBuilding);
        assert_eq!(unit.state_timer, 0);

        // Tick 29 times — should still be continuing
        for _ in 0..29 {
            let (result, action) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
            assert_eq!(action, DeferredAction::None);
        }

        // Tick 30 — should transition to InsideBuilding with AddToBuilding action
        let (result, action) = tick_state(&mut unit, &mut rng);
        assert!(matches!(
            result,
            TickResult::Transition(PersonState::InsideBuilding)
        ));
        assert_eq!(
            action,
            DeferredAction::AddToBuilding {
                person: h(0),
                building: h(5)
            }
        );
    }

    #[test]
    fn enter_building_no_handle_no_deferred_action() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.building_handle = None;
        enter_state(&mut unit, PersonState::EnterBuilding, &mut rng);

        // Drain to completion
        for _ in 0..30 {
            tick_state(&mut unit, &mut rng);
        }
        // At tick 30, should transition but with no deferred action
        // (already consumed the 30th tick above)
        // Actually let's just check that building_handle None produces DeferredAction::None
        let mut unit2 = make_unit(2, 0);
        unit2.building_handle = None;
        unit2.state = PersonState::EnterBuilding;
        unit2.state_timer = 29;
        let (result, action) = tick_state(&mut unit2, &mut rng);
        assert!(matches!(
            result,
            TickResult::Transition(PersonState::InsideBuilding)
        ));
        assert_eq!(action, DeferredAction::None);
    }

    #[test]
    fn housing_stays_indefinitely() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Housing, &mut rng);
        assert_eq!(unit.state, PersonState::Housing);
        assert_eq!(unit.state_timer, 0);

        // Tick 100 times — should always continue
        for _ in 0..100 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
        }
    }

    #[test]
    fn training_countdown_to_wait_outside() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.building_handle = Some(h(10));
        enter_state(&mut unit, PersonState::Training, &mut rng);
        assert_eq!(unit.state, PersonState::Training);
        assert_eq!(unit.state_timer, 256); // default training time

        // Tick 255 times — should still be training
        for _ in 0..255 {
            let (result, action) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
            assert_eq!(action, DeferredAction::None);
        }

        // Tick 256 — timer reaches 0, should transition to WaitOutside
        let (result, action) = tick_state(&mut unit, &mut rng);
        assert!(matches!(
            result,
            TickResult::Transition(PersonState::WaitOutside)
        ));
        assert_eq!(action, DeferredAction::SpawnAtBuilding { building: h(10) });
    }

    #[test]
    fn training_custom_timer() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        // Pre-set a custom timer before enter_state
        unit.state_timer = 50;
        enter_state(&mut unit, PersonState::Training, &mut rng);
        // Should keep the custom timer (non-zero is preserved)
        assert_eq!(unit.state_timer, 50);
    }

    #[test]
    fn wait_outside_clears_building_and_transitions_to_idle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.building_handle = Some(h(7));
        enter_state(&mut unit, PersonState::WaitOutside, &mut rng);
        assert_eq!(unit.state, PersonState::WaitOutside);
        assert_eq!(unit.building_handle, None); // cleared on enter
        assert_eq!(unit.state_timer, 0);

        // Tick 19 times — should continue
        for _ in 0..19 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
        }

        // Tick 20 — should transition to Idle
        let (result, _) = tick_state(&mut unit, &mut rng);
        assert!(matches!(result, TickResult::Transition(PersonState::Idle)));
    }

    #[test]
    fn gathering_emits_find_nearest_tree_when_no_target() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::Gathering, &mut rng);
        assert_eq!(unit.state, PersonState::Gathering);
        assert_eq!(unit.wood_carried, 0);
        assert_eq!(unit.state_timer, 0); // needs tree target
        assert_eq!(unit.gather_target, None);

        // First tick: state_timer=0 emits FindNearestTree
        let (result, action) = tick_state(&mut unit, &mut rng);
        assert!(matches!(result, TickResult::Continue));
        assert_eq!(action, DeferredAction::FindNearestTree { unit_index: 0 });
    }

    #[test]
    fn gathering_navigates_to_tree_and_transitions() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.movement.position = WorldCoord::new(0, 0);
        enter_state(&mut unit, PersonState::Gathering, &mut rng);

        // Simulate coordinator setting tree target (200 units away on x-axis)
        unit.gather_target = Some(WorldCoord::new(200, 0));
        unit.state_timer = 1; // has target

        // Tick until arrival (200 / 4 = 50 ticks max)
        let mut transitioned = false;
        for _ in 0..60 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            if matches!(result, TickResult::Transition(PersonState::GatheringWood)) {
                transitioned = true;
                break;
            }
        }
        assert!(
            transitioned,
            "Should transition to GatheringWood when near tree"
        );
    }

    #[test]
    fn gathering_wood_chops_then_carrying() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        enter_state(&mut unit, PersonState::GatheringWood, &mut rng);
        assert_eq!(unit.state, PersonState::GatheringWood);
        assert_eq!(unit.state_timer, 60); // chop time

        // Tick 59 times — should continue
        for _ in 0..59 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
        }

        // Tick 60 — chop done, should set wood_carried and transition to CarryingWood
        let (result, _) = tick_state(&mut unit, &mut rng);
        assert!(matches!(
            result,
            TickResult::Transition(PersonState::CarryingWood)
        ));
        assert_eq!(unit.wood_carried, 1);
    }

    #[test]
    fn carrying_wood_deposits_and_loops() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.building_handle = Some(h(3));
        unit.wood_carried = 1;
        enter_state(&mut unit, PersonState::CarryingWood, &mut rng);
        assert_eq!(unit.state, PersonState::CarryingWood);
        assert_eq!(unit.state_timer, 0);

        // Tick 39 times — should continue
        for _ in 0..39 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
        }

        // Tick 40 — should deposit wood and loop back to Gathering
        let (result, action) = tick_state(&mut unit, &mut rng);
        assert!(matches!(
            result,
            TickResult::Transition(PersonState::Gathering)
        ));
        assert_eq!(
            action,
            DeferredAction::DepositWood {
                building: h(3),
                amount: 1
            }
        );
        assert_eq!(unit.wood_carried, 0);
    }

    #[test]
    fn wood_gathering_full_cycle() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.building_handle = Some(h(3));
        unit.movement.position = WorldCoord::new(0, 0);
        enter_state(&mut unit, PersonState::Gathering, &mut rng);

        // Phase 1: Gathering — first tick emits FindNearestTree
        let (_, action) = tick_state(&mut unit, &mut rng);
        assert_eq!(action, DeferredAction::FindNearestTree { unit_index: 0 });

        // Simulate coordinator setting tree target (close by, 100 units away)
        unit.gather_target = Some(WorldCoord::new(100, 0));
        unit.state_timer = 1;

        // Navigate to tree until transition
        for _ in 0..50 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            if matches!(result, TickResult::Transition(PersonState::GatheringWood)) {
                break;
            }
        }
        // Simulate coordinator transition
        enter_state(&mut unit, PersonState::GatheringWood, &mut rng);

        // Phase 2: GatheringWood (chop) — 60 ticks
        for _ in 0..60 {
            tick_state(&mut unit, &mut rng);
        }
        assert_eq!(unit.wood_carried, 1);
        // Simulate coordinator transition
        enter_state(&mut unit, PersonState::CarryingWood, &mut rng);

        // Phase 3: CarryingWood (walk to building) — 40 ticks
        for _ in 0..39 {
            tick_state(&mut unit, &mut rng);
        }
        let (result, action) = tick_state(&mut unit, &mut rng);
        assert!(matches!(
            result,
            TickResult::Transition(PersonState::Gathering)
        ));
        assert_eq!(
            action,
            DeferredAction::DepositWood {
                building: h(3),
                amount: 1
            }
        );
    }

    #[test]
    fn deferred_action_enum_values() {
        // Verify all DeferredAction variants can be constructed
        let none = DeferredAction::None;
        let add = DeferredAction::AddToBuilding {
            person: h(0),
            building: h(1),
        };
        let remove = DeferredAction::RemoveFromBuilding {
            person: h(0),
            building: h(2),
        };
        let deposit = DeferredAction::DepositWood {
            building: h(3),
            amount: 5,
        };
        let spawn = DeferredAction::SpawnAtBuilding { building: h(4) };
        assert_eq!(none, DeferredAction::None);
        assert_eq!(
            add,
            DeferredAction::AddToBuilding {
                person: h(0),
                building: h(1)
            }
        );
        assert_eq!(
            remove,
            DeferredAction::RemoveFromBuilding {
                person: h(0),
                building: h(2)
            }
        );
        assert_eq!(
            deposit,
            DeferredAction::DepositWood {
                building: h(3),
                amount: 5
            }
        );
        assert_eq!(spawn, DeferredAction::SpawnAtBuilding { building: h(4) });
        let find_tree = DeferredAction::FindNearestTree { unit_index: 5 };
        assert_eq!(find_tree, DeferredAction::FindNearestTree { unit_index: 5 });
    }

    #[test]
    fn unit_new_fields_default() {
        let unit = make_unit(2, 0);
        assert_eq!(unit.building_handle, None);
        assert_eq!(unit.wood_carried, 0);
        assert_eq!(unit.guard_position, None);
        assert_eq!(unit.gather_target, None);
    }

    #[test]
    fn inside_building_stays_until_coordinator_transitions() {
        let mut unit = make_unit(2, 0);
        let mut rng = GameRng::new(42);
        unit.state = PersonState::InsideBuilding;
        for _ in 0..50 {
            let (result, _) = tick_state(&mut unit, &mut rng);
            assert!(matches!(result, TickResult::Continue));
        }
    }
}
