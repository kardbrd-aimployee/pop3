// Person animation state and selection — faithful to Person_SelectAnimation (0x004fed30),
// Person_SetAnimation (0x004feed0), and Sprite_TickAnimCycles (0x004ea0e0).
//
// Animation fields in original binary live at object offsets +0x33..+0x3a.
// The animation type table at DAT_0059fb30 maps (anim_type * 9 + subtype) to VSTART indices.

use super::person_state::PersonState;

/// g_PersonAnimationTable (DAT_0059fb30): table[anim_type][subtype] → VSTART animation index.
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

/// Animation speed multiplier per subtype (DAT_0059f8db, stride 0x0b).
/// Ticks per frame = value + 1.
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

/// Per-unit animation state, matching original binary offsets +0x33..+0x3a.
#[derive(Debug, Clone, Copy)]
pub struct AnimationState {
    /// Current VSTART animation index (+0x33).
    pub animation_id: u16,
    /// Animation flags (+0x35): bit 0 = loop, bit 1 = playing.
    pub flags: u8,
    /// Frame timing accumulator (+0x37).
    pub tick_counter: u16,
    /// Current frame within animation (+0x39).
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

/// Map PersonState to animation type index for table lookup.
/// Faithful to Person_SelectAnimation (0x004fed30).
/// Returns the animation type (row in PERSON_ANIMATION_TABLE).
pub fn state_to_anim_type(state: PersonState) -> u8 {
    match state {
        // These states map to Idle (type 0)
        PersonState::Idle
        | PersonState::Moving
        | PersonState::InsideTraining
        | PersonState::Gathering
        | PersonState::Fighting
        | PersonState::InShield
        | PersonState::EnteringVehicle
        | PersonState::WaitingAtReincPillar => 0,

        // Walk (type 1) — default for movement states
        PersonState::Wander
        | PersonState::GoToPoint
        | PersonState::FollowPath
        | PersonState::GoToMarker
        | PersonState::WaitForPath
        | PersonState::WaitAtMarker
        | PersonState::EnterBuilding
        | PersonState::Building
        | PersonState::GatheringWood
        | PersonState::CarryingWood
        | PersonState::Spawning
        | PersonState::BeingSacrificed
        | PersonState::SitDown
        | PersonState::BeingConverted
        | PersonState::WaitingAfterConvert
        | PersonState::WaitingForBoat
        | PersonState::Placeholder
        | PersonState::GetOffBoat
        | PersonState::WaitingInWater
        | PersonState::Celebrating
        | PersonState::Teleporting
        | PersonState::InternalState => 1,

        // Action (type 3)
        PersonState::InsideBuilding | PersonState::InTraining => 3,

        // Death uses Vehicle type (0x0C = 12) in original
        PersonState::Dead => 6,

        // Run (type 25 = 0x19)
        PersonState::Fleeing | PersonState::Preaching | PersonState::ExitingVehicle => 25,

        // Drowning = Swim (type 16)
        PersonState::Drowning => 16,

        // Dying = Die (type 6)
        PersonState::Dying => 6,

        // WaitOutside, Training, Housing, InShieldIdle — use Walk
        PersonState::WaitOutside
        | PersonState::Training
        | PersonState::Housing
        | PersonState::InShieldIdle => 1,
    }
}

/// Look up VSTART animation index for a given state and subtype.
/// Returns None if the combination has no animation (-1 in table).
pub fn lookup_animation(state: PersonState, subtype: u8) -> Option<u16> {
    let anim_type = state_to_anim_type(state) as usize;
    if anim_type >= PERSON_ANIMATION_TABLE.len() {
        return None;
    }
    let col = (subtype as usize).min(8);
    let val = PERSON_ANIMATION_TABLE[anim_type][col];
    if val < 0 {
        None
    } else {
        Some(val as u16)
    }
}

/// Select and set animation based on current state.
/// Equivalent to Person_SelectAnimation + Person_SetAnimation.
/// `frame_counts` maps animation index → number of frames.
/// `is_moving` overrides walk→idle when unit is stationary (matches decomp wander_timer==0 check).
pub fn select_animation(
    anim: &mut AnimationState,
    state: PersonState,
    subtype: u8,
    frame_counts: &[u8],
    is_moving: bool,
) {
    let mut anim_type = state_to_anim_type(state);
    // Override: walk type but not actually moving → use idle (matches decomp)
    if anim_type == 1 && !is_moving {
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
        anim.flags = 0x03; // loop + playing
        anim.frame_count = frame_counts.get(new_id as usize).copied().unwrap_or(1);
        let speed_idx = (subtype as usize).min(ANIM_SPEED_MULTIPLIER.len() - 1);
        anim.ticks_per_frame = ANIM_SPEED_MULTIPLIER[speed_idx] + 1;
    }
}

/// Advance animation by one tick.
/// Equivalent to Sprite_TickAnimCycles (0x004ea0e0).
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
    fn lookup_aod_returns_none_for_idle() {
        // AOD (subtype 8) has -1 for idle
        assert_eq!(lookup_animation(PersonState::Idle, 8), None);
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
        select_animation(
            &mut anim,
            PersonState::Idle,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            false,
        );
        assert_eq!(anim.animation_id, 15);
        assert_eq!(anim.frame_index, 0);
        assert_eq!(anim.frame_count, 4);
        assert_eq!(anim.ticks_per_frame, 3); // brave = multiplier 2 + 1
    }

    #[test]
    fn select_animation_walk_changes_id() {
        let fc = test_frame_counts();
        let mut anim = AnimationState::default();
        select_animation(
            &mut anim,
            PersonState::Idle,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            false,
        );
        assert_eq!(anim.animation_id, 15);
        select_animation(
            &mut anim,
            PersonState::GoToPoint,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            true,
        );
        assert_eq!(anim.animation_id, 21);
        assert_eq!(anim.frame_index, 0); // reset on change
        assert_eq!(anim.frame_count, 6); // walk brave frame count
    }

    #[test]
    fn select_animation_same_id_no_reset() {
        let fc = test_frame_counts();
        let mut anim = AnimationState::default();
        select_animation(
            &mut anim,
            PersonState::Idle,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            false,
        );
        anim.frame_index = 3;
        anim.tick_counter = 2;
        // Same state, same subtype → same animation_id → no reset
        select_animation(
            &mut anim,
            PersonState::Idle,
            PERSON_SUBTYPE_BRAVE,
            &fc,
            false,
        );
        assert_eq!(anim.frame_index, 3);
        assert_eq!(anim.tick_counter, 2);
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
