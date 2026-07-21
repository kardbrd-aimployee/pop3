// Person animation state and selection. Logical animation selection follows
// Person_SelectAnimation (0x004fed30) and Person_SetAnimationByState
// (0x004fee80). Native Person_SetAnimation (0x004feed0) resolves that logical
// ID into a VSTART index/render type and Animation_UpdateObjectTrack
// (0x004b0b80) advances the resolved track.
//
// Animation fields in original binary live at object offsets +0x33..+0x3a.
// The animation type table at DAT_0059fb30 maps (anim_type * 9 + subtype) to
// logical animation IDs. Person_SetAnimation resolves those through the shape
// table at DAT_0059f638.

use super::person_state::PersonState;

/// g_PersonAnimationTable (DAT_0059fb30): table[anim_type][subtype] → logical animation ID.
/// 26 animation types × 9 subtypes (0=none, 1=wild, 2=brave, ..., 7=shaman, 8=aod).
/// -1 means no animation for that combination.
/// Extracted from ~/decomp_export/sections/.data.bin at offset 0x2AB30.
pub const PERSON_ANIMATION_TABLE: [[i16; 9]; 26] = [
    /*  0 Idle */ [0, 0, 15, 16, 17, 18, 19, 20, -1],
    /*  1 Walk */ [0, 1, 21, 22, 23, 24, 25, 26, -1],
    /*  2 Ride */ [0, 130, 110, 111, 112, 113, 114, 129, -1],
    /*  3 Actn */ [0, -1, 32, 33, 34, 35, 36, 37, -1],
    /*  4 SpId */ [0, -1, 43, 44, 45, 46, 47, 20, -1],
    /*  5 SpWk */ [0, -1, 48, 49, 50, 51, 52, 26, -1],
    /*  6 Die  */ [0, -1, 27, 28, 29, 30, 31, 20, -1],
    /*  7 Celb */ [0, -1, 38, 39, 40, 41, 42, 20, -1],
    /*  8 Wrk1 */ [0, -1, 53, 54, 55, 56, 57, 106, -1],
    /*  9 Wrk2 */ [0, -1, 58, 59, 60, 61, 62, 20, -1],
    /* 10 Wrk3 */ [0, -1, 63, 64, 65, 66, 67, 106, -1],
    /* 11 Wrk4 */ [0, -1, 68, 69, 70, 71, 72, 20, -1],
    /* 12 Vhcl */ [0, 108, 78, 79, 80, 81, 82, 107, -1],
    /* 13 Wrk5 */ [0, -1, 73, 74, 75, 76, 77, 20, -1],
    /* 14 Spec */ [0, -1, 100, -1, -1, 101, -1, -1, -1],
    /* 15 Sham */ [0, -1, -1, -1, -1, -1, 94, -1, -1],
    /* 16 Swim */ [0, -1, 83, 84, 85, 86, 87, 125, -1],
    /* 17  ??? */ [0, -1, -1, -1, -1, -1, -1, 107, -1],
    /* 18 Crry */ [0, 0, 88, 89, 90, 91, 92, 127, -1],
    /* 19 Dig  */ [0, 0, 115, 116, 117, 118, 119, 126, -1],
    /* 20 Bld  */ [0, 108, 120, 121, 122, 123, 124, 128, -1],
    /* 21 Sit1 */ [0, 0, 131, 132, 133, 134, 135, 20, -1],
    /* 22 Sit2 */ [0, 0, 136, 137, 138, 139, 140, 20, -1],
    /* 23 Sit3 */ [0, 0, 141, 142, 143, 144, 145, 20, -1],
    /* 24 Sit4 */ [0, 0, 146, 147, 148, 149, 150, 20, -1],
    /* 25 Run  */ [0, 1, 156, 157, 158, 159, 160, 26, -1],
];

/// Compatibility cadence used by the Rust logical-animation player.
///
/// This was originally inferred from early table inspection and must not be
/// confused with a native per-subtype table. The original track updater reads
/// its mode/timing fields from the 11-byte bank row selected by the resolved
/// render type. Keeping this table for now preserves the already-captured Rust
/// cadence until the native outer-loop call rate is measured.
pub const ANIM_SPEED_MULTIPLIER: [u8; 9] = [
    0, // subtype 0 (none)
    4, // subtype 1 (wild)       → 5 ticks/frame
    2, // subtype 2 (brave)      → 3 ticks/frame
    2, // subtype 3 (warrior)    → 3 ticks/frame
    4, // subtype 4 (preacher)   → 5 ticks/frame
    0, // subtype 5 (spy)        → 1 tick/frame
    0, // subtype 6 (firewarrior)→ 1 tick/frame
    2, // subtype 7 (shaman)     → 3 ticks/frame
    0, // subtype 8 (aod)        → 1 tick/frame
];

/// The initial selection made by `Person_SelectAnimation @ 0x004fed30` before
/// linked-person and vehicle overrides are applied.
///
/// Most behavior handlers replace this default later with
/// `Person_SetAnimationByState`, which is why a semantic Rust state such as
/// construction must not be assumed to map directly to the visible work row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeStateAnimation {
    TableRow(u8),
    LogicalAnimation(u16),
}

pub fn native_state_animation(state: PersonState) -> NativeStateAnimation {
    use NativeStateAnimation::{LogicalAnimation, TableRow};

    match state {
        PersonState::Idle
        | PersonState::Moving
        | PersonState::InsideTraining
        | PersonState::Gathering
        | PersonState::Fighting
        | PersonState::InShield
        | PersonState::EnteringVehicle
        | PersonState::WaitingAtReincPillar => TableRow(0),
        PersonState::InsideBuilding | PersonState::InTraining => TableRow(3),
        PersonState::WaitOutside => LogicalAnimation(2),
        PersonState::Training => LogicalAnimation(3),
        PersonState::Dead => TableRow(12),
        PersonState::Fleeing | PersonState::Preaching | PersonState::ExitingVehicle => TableRow(25),
        _ => TableRow(1),
    }
}

/// Per-unit animation state used by the Rust renderer.
///
/// This intentionally retains the logical animation ID. The native object
/// does not: Person_SetAnimation resolves it and stores a VSTART index at
/// +0x33 plus a render type at +0x3a. Keeping the logical ID lets the Rust
/// atlas select the corresponding packed animation directly.
#[derive(Debug, Clone, Copy)]
pub struct AnimationState {
    /// Current logical animation index (the index into ANIM_SHAPE_TABLE).
    pub animation_id: u16,
    /// Rust playback flags: bit 0 = loop, bit 1 = playing.
    pub flags: u8,
    /// Rust frame timing accumulator.
    pub tick_counter: u16,
    /// Current frame within the packed animation.
    pub frame_index: u8,
    /// Total frames in current animation (cached from atlas data).
    pub frame_count: u8,
    /// Ticks per frame advance (from ANIM_SPEED_MULTIPLIER).
    pub ticks_per_frame: u8,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            animation_id: 0,
            flags: 0x03, // loop + playing
            tick_counter: 0,
            frame_index: 0,
            frame_count: 1,
            ticks_per_frame: 1,
        }
    }
}

/// Map the Rust engine's semantic person states to visible animation rows.
///
/// Several Rust states intentionally cover behavior that the original game
/// expressed through a state plus a later `Person_SetAnimationByState` call.
/// This adapter therefore selects the final visible action (chop, carry,
/// build, swim, and so on), rather than only reproducing the native switch's
/// initial idle/walk default.
pub fn state_to_anim_type(state: PersonState) -> u8 {
    match state {
        // Stationary states.
        PersonState::Idle
        | PersonState::InsideTraining
        | PersonState::InShield
        | PersonState::WaitingAtReincPillar => 0,

        // Navigation states. `select_animation` changes these to idle while
        // the movement flag is clear.
        PersonState::Moving
        | PersonState::Wander
        | PersonState::GoToPoint
        | PersonState::FollowPath
        | PersonState::GoToMarker
        | PersonState::WaitForPath
        | PersonState::WaitAtMarker
        | PersonState::EnterBuilding
        | PersonState::WaitOutside
        | PersonState::Training
        | PersonState::Housing
        | PersonState::Gathering
        | PersonState::Spawning
        | PersonState::BeingConverted
        | PersonState::WaitingAfterConvert
        | PersonState::WaitingForBoat
        | PersonState::Placeholder
        | PersonState::GetOffBoat
        | PersonState::EnteringVehicle
        | PersonState::Teleporting
        | PersonState::InternalState
        | PersonState::InShieldIdle => 1,

        // Close action / melee work.
        PersonState::InsideBuilding | PersonState::InTraining | PersonState::Fighting => 3,

        // Death sequence.
        PersonState::Dying | PersonState::Dead | PersonState::BeingSacrificed => 6,

        // Celebration.
        PersonState::Celebrating => 7,

        // Tree work / chopping.
        PersonState::GatheringWood => 13,

        // Swimming and drowning.
        PersonState::Drowning | PersonState::WaitingInWater => 16,

        // Carrying the wood prop.
        PersonState::CarryingWood => 18,

        // Final construction strokes. Foundation digging is selected by the
        // authoritative construction state machine as animation row 19.
        PersonState::Building => 20,

        // The first of the four native seated variants.
        PersonState::SitDown => 21,

        // Fast movement.
        PersonState::Fleeing | PersonState::Preaching | PersonState::ExitingVehicle => 25,
    }
}

/// Resolve one native animation-table row for a person subtype.
pub fn lookup_animation_type(anim_type: u8, subtype: u8) -> Option<u16> {
    let row = PERSON_ANIMATION_TABLE.get(anim_type as usize)?;
    let value = row[(subtype as usize).min(8)];
    (value >= 0).then_some(value as u16)
}

/// Look up VSTART animation index for a given state and subtype.
/// Returns None if the combination has no animation (-1 in table).
pub fn lookup_animation(state: PersonState, subtype: u8) -> Option<u16> {
    lookup_animation_type(state_to_anim_type(state), subtype)
}

/// Select and set the Rust engine's visible animation based on current state.
/// Idle/walk selection matches `Person_SelectAnimation`; implemented action
/// states retain their explicit semantic rows until their native behavior
/// handlers have been translated in full.
/// `frame_counts` maps animation index → number of frames.
/// `movement_speed` overrides walk→idle when zero, matching the native
/// comparison against the speed word at person offset +0x5f.
pub fn select_animation(
    anim: &mut AnimationState,
    state: PersonState,
    subtype: u8,
    frame_counts: &[u8],
    movement_speed: u16,
) {
    let mut anim_type = state_to_anim_type(state);
    // Person_SelectAnimation @ 0x004fed91: a walk-class state with zero
    // speed uses the idle animation, regardless of the target/moving flags.
    if anim_type == 1 && movement_speed == 0 {
        anim_type = 0;
    }
    let col = (subtype as usize).min(8);
    let new_id = {
        let val = PERSON_ANIMATION_TABLE[anim_type as usize][col];
        if val >= 0 {
            val as u16
        } else {
            // Fallback to idle
            let idle_val = PERSON_ANIMATION_TABLE[0][col];
            if idle_val >= 0 {
                idle_val as u16
            } else {
                0
            }
        }
    };

    if new_id != anim.animation_id {
        anim.animation_id = new_id;
        anim.frame_index = 0;
        anim.tick_counter = 0;
        anim.flags = if matches!(
            state,
            PersonState::Dying | PersonState::Dead | PersonState::BeingSacrificed
        ) {
            0x02 // play once and hold the last frame
        } else {
            0x03 // loop + playing
        };
        anim.frame_count = frame_counts.get(new_id as usize).copied().unwrap_or(1);
        let speed_idx = (subtype as usize).min(ANIM_SPEED_MULTIPLIER.len() - 1);
        anim.ticks_per_frame = ANIM_SPEED_MULTIPLIER[speed_idx] + 1;
    }
}

/// Advance the Rust renderer's logical animation by one simulation tick.
/// Native track control bits at +0x35 have different semantics; translating
/// every native track mode remains separate from logical state selection.
pub fn tick_animation(anim: &mut AnimationState) {
    // Not playing
    if anim.flags & 0x02 == 0 {
        return;
    }
    // Single-frame animation
    if anim.frame_count <= 1 {
        return;
    }

    anim.tick_counter += 1;
    if anim.tick_counter >= anim.ticks_per_frame as u16 {
        anim.tick_counter = 0;
        anim.frame_index += 1;
        if anim.frame_index >= anim.frame_count {
            if anim.flags & 0x01 != 0 {
                anim.frame_index = 0; // loop
            } else {
                anim.frame_index = anim.frame_count - 1; // hold last frame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::state::constants::*;

    #[test]
    fn table_idle_brave() {
        assert_eq!(PERSON_ANIMATION_TABLE[0][PERSON_SUBTYPE_BRAVE as usize], 15);
    }

    #[test]
    fn table_idle_shaman() {
        assert_eq!(
            PERSON_ANIMATION_TABLE[0][PERSON_SUBTYPE_SHAMAN as usize],
            20
        );
    }

    #[test]
    fn table_walk_brave() {
        assert_eq!(PERSON_ANIMATION_TABLE[1][PERSON_SUBTYPE_BRAVE as usize], 21);
    }

    #[test]
    fn table_walk_shaman() {
        assert_eq!(
            PERSON_ANIMATION_TABLE[1][PERSON_SUBTYPE_SHAMAN as usize],
            26
        );
    }

    #[test]
    fn table_die_brave() {
        assert_eq!(PERSON_ANIMATION_TABLE[6][PERSON_SUBTYPE_BRAVE as usize], 27);
    }

    #[test]
    fn table_run_brave() {
        assert_eq!(
            PERSON_ANIMATION_TABLE[25][PERSON_SUBTYPE_BRAVE as usize],
            156
        );
    }

    #[test]
    fn lookup_idle_brave() {
        assert_eq!(
            lookup_animation(PersonState::Idle, PERSON_SUBTYPE_BRAVE),
            Some(15)
        );
    }

    #[test]
    fn lookup_idle_shaman() {
        assert_eq!(
            lookup_animation(PersonState::Idle, PERSON_SUBTYPE_SHAMAN),
            Some(20)
        );
    }

    #[test]
    fn lookup_walk_brave() {
        assert_eq!(
            lookup_animation(PersonState::GoToPoint, PERSON_SUBTYPE_BRAVE),
            Some(21)
        );
    }

    #[test]
    fn lookup_wander_brave() {
        assert_eq!(
            lookup_animation(PersonState::Wander, PERSON_SUBTYPE_BRAVE),
            Some(21)
        );
    }

    #[test]
    fn lookup_fleeing_brave() {
        // Fleeing → Run (type 25)
        assert_eq!(
            lookup_animation(PersonState::Fleeing, PERSON_SUBTYPE_BRAVE),
            Some(156)
        );
    }

    #[test]
    fn lookup_dead_brave() {
        // Dead → Die (type 6)
        assert_eq!(
            lookup_animation(PersonState::Dead, PERSON_SUBTYPE_BRAVE),
            Some(27)
        );
    }

    #[test]
    fn lookup_drowning_brave() {
        // Drowning → Swim (type 16)
        assert_eq!(
            lookup_animation(PersonState::Drowning, PERSON_SUBTYPE_BRAVE),
            Some(83)
        );
    }

    #[test]
    fn supported_actions_resolve_native_brave_rows() {
        assert_eq!(
            lookup_animation(PersonState::Fighting, PERSON_SUBTYPE_BRAVE),
            Some(32)
        );
        assert_eq!(
            lookup_animation(PersonState::GatheringWood, PERSON_SUBTYPE_BRAVE),
            Some(73)
        );
        assert_eq!(
            lookup_animation(PersonState::CarryingWood, PERSON_SUBTYPE_BRAVE),
            Some(88)
        );
        assert_eq!(
            lookup_animation(PersonState::Building, PERSON_SUBTYPE_BRAVE),
            Some(120)
        );
        assert_eq!(
            lookup_animation(PersonState::Celebrating, PERSON_SUBTYPE_BRAVE),
            Some(38)
        );
    }

    #[test]
    fn semantic_animation_rows_cover_every_rendered_specialist() {
        for subtype in PERSON_SUBTYPE_BRAVE..=PERSON_SUBTYPE_SHAMAN {
            for state in [
                PersonState::Idle,
                PersonState::GoToPoint,
                PersonState::Fighting,
                PersonState::Dying,
                PersonState::Celebrating,
                PersonState::Drowning,
                PersonState::CarryingWood,
                PersonState::Building,
                PersonState::Fleeing,
            ] {
                assert!(
                    lookup_animation(state, subtype).is_some(),
                    "subtype {subtype} has no animation for {state:?}"
                );
            }
        }
    }

    #[test]
    fn lookup_aod_returns_none_for_idle() {
        // AOD (subtype 8) has -1 for idle
        assert_eq!(lookup_animation(PersonState::Idle, 8), None);
    }

    #[test]
    fn native_state_dispatch_matches_executable_switch() {
        use NativeStateAnimation::{LogicalAnimation, TableRow};

        assert_eq!(native_state_animation(PersonState::Idle), TableRow(0));
        assert_eq!(native_state_animation(PersonState::Moving), TableRow(0));
        assert_eq!(native_state_animation(PersonState::GoToMarker), TableRow(1));
        assert_eq!(native_state_animation(PersonState::Building), TableRow(1));
        assert_eq!(
            native_state_animation(PersonState::InsideBuilding),
            TableRow(3)
        );
        assert_eq!(
            native_state_animation(PersonState::WaitOutside),
            LogicalAnimation(2)
        );
        assert_eq!(
            native_state_animation(PersonState::Training),
            LogicalAnimation(3)
        );
        assert_eq!(native_state_animation(PersonState::Dead), TableRow(12));
        assert_eq!(native_state_animation(PersonState::Fleeing), TableRow(25));
        assert_eq!(native_state_animation(PersonState::Preaching), TableRow(25));
        assert_eq!(
            native_state_animation(PersonState::ExitingVehicle),
            TableRow(25)
        );
    }

    // Frame counts for test animations: index 15 (idle brave) = 4 frames, 21 (walk brave) = 6 frames
    fn test_frame_counts() -> Vec<u8> {
        let mut fc = vec![1u8; 200];
        fc[15] = 4; // idle brave
        fc[20] = 3; // idle shaman
        fc[21] = 6; // walk brave
        fc
    }

    #[test]
    fn select_animation_sets_idle_brave() {
        let fc = test_frame_counts();
        let mut anim = AnimationState::default();
        select_animation(&mut anim, PersonState::Idle, PERSON_SUBTYPE_BRAVE, &fc, 0);
        assert_eq!(anim.animation_id, 15);
        assert_eq!(anim.frame_index, 0);
        assert_eq!(anim.frame_count, 4);
        assert_eq!(anim.ticks_per_frame, 3); // brave = multiplier 2 + 1
    }

    #[test]
    fn select_animation_walk_changes_id() {
        let fc = test_frame_counts();
        let mut anim = AnimationState::default();
        select_animation(&mut anim, PersonState::Idle, PERSON_SUBTYPE_BRAVE, &fc, 0);
        assert_eq!(anim.animation_id, 15);
        select_animation(
            &mut anim,
            PersonState::GoToPoint,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            0x30,
        );
        assert_eq!(anim.animation_id, 21);
        assert_eq!(anim.frame_index, 0); // reset on change
        assert_eq!(anim.frame_count, 6); // walk brave frame count
    }

    #[test]
    fn select_animation_same_id_no_reset() {
        let fc = test_frame_counts();
        let mut anim = AnimationState::default();
        select_animation(&mut anim, PersonState::Idle, PERSON_SUBTYPE_BRAVE, &fc, 0);
        anim.frame_index = 3;
        anim.tick_counter = 2;
        // Same state, same subtype → same animation_id → no reset
        select_animation(&mut anim, PersonState::Idle, PERSON_SUBTYPE_BRAVE, &fc, 0);
        assert_eq!(anim.frame_index, 3);
        assert_eq!(anim.tick_counter, 2);
    }

    #[test]
    fn death_animation_plays_once() {
        let mut frame_counts = test_frame_counts();
        frame_counts[27] = 4;
        let mut anim = AnimationState::default();
        select_animation(
            &mut anim,
            PersonState::Dead,
            PERSON_SUBTYPE_BRAVE,
            &frame_counts,
            0,
        );
        assert_eq!(anim.animation_id, 27);
        assert_eq!(anim.flags, 0x02);
    }

    #[test]
    fn goto_with_zero_speed_uses_idle_like_native_selector() {
        let fc = test_frame_counts();
        let mut anim = AnimationState::default();
        select_animation(
            &mut anim,
            PersonState::GoToPoint,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            0,
        );
        assert_eq!(anim.animation_id, 15);
    }

    #[test]
    fn tick_animation_advances_frame() {
        let mut anim = AnimationState {
            animation_id: 15,
            flags: 0x03,
            tick_counter: 0,
            frame_index: 0,
            frame_count: 4,
            ticks_per_frame: 1,
        };
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 1);
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 2);
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 3);
        // Wraps to 0 on loop
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 0);
    }

    #[test]
    fn tick_animation_respects_speed() {
        let mut anim = AnimationState {
            animation_id: 15,
            flags: 0x03,
            tick_counter: 0,
            frame_index: 0,
            frame_count: 4,
            ticks_per_frame: 3, // brave speed
        };
        // Frame 0 for 3 ticks
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 0);
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 0);
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 1); // advances on 3rd tick
    }

    #[test]
    fn tick_animation_not_playing() {
        let mut anim = AnimationState {
            animation_id: 15,
            flags: 0x01, // loop but NOT playing (bit 1 clear)
            tick_counter: 0,
            frame_index: 0,
            frame_count: 4,
            ticks_per_frame: 1,
        };
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 0); // no advance
    }

    #[test]
    fn tick_animation_hold_last_frame_no_loop() {
        let mut anim = AnimationState {
            animation_id: 27,
            flags: 0x02, // playing but NOT looping
            tick_counter: 0,
            frame_index: 2,
            frame_count: 3,
            ticks_per_frame: 1,
        };
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 2); // holds at last frame (frame_count - 1)
    }

    #[test]
    fn tick_animation_single_frame_noop() {
        let mut anim = AnimationState {
            animation_id: 15,
            flags: 0x03,
            tick_counter: 0,
            frame_index: 0,
            frame_count: 1,
            ticks_per_frame: 1,
        };
        tick_animation(&mut anim);
        assert_eq!(anim.frame_index, 0);
        assert_eq!(anim.tick_counter, 0);
    }

    #[test]
    fn speed_multiplier_values() {
        assert_eq!(ANIM_SPEED_MULTIPLIER[PERSON_SUBTYPE_BRAVE as usize], 2);
        assert_eq!(ANIM_SPEED_MULTIPLIER[PERSON_SUBTYPE_WARRIOR as usize], 2);
        assert_eq!(ANIM_SPEED_MULTIPLIER[PERSON_SUBTYPE_SHAMAN as usize], 2);
    }
}
