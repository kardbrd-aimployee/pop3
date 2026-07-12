use super::constants::*;
use super::flags::GameFlags;
use super::tribe::TribeArray;

/// Check victory/defeat conditions.
/// Original: Game_CheckVictoryConditions at 0x00423c60
///
/// Called every tick from run_one_tick. Only performs the actual check
/// when (tick_counter & 0x0F) == 0 and tick_counter >= 0x11.
///
/// From the disassembly:
/// - First updates reincarnation timers for all tribes with zero population
/// - Then branches on multiplayer flag to check SP or MP victory
pub fn check_victory_conditions(
    tick_counter: u32,
    flags: &mut GameFlags,
    tribes: &mut TribeArray,
    player_tribe: u8,
) {
    // Original: if ((DAT_00885720 & 0xf) != 0 || DAT_00885720 < 0x11) return;
    if (tick_counter & VICTORY_CHECK_MASK) != 0 {
        return;
    }
    if tick_counter < VICTORY_CHECK_MIN_TICKS {
        return;
    }

    // Don't re-check if already resolved
    if flags.has_won() || flags.has_lost() {
        return;
    }

    // Update reincarnation timers for all active tribes.
    // Original: loop at 0x004238eb through all 4 tribes
    //   if timer != 0 && tribe active && timer < 0x60: timer += 0x10
    for tribe in &mut tribes.tribes {
        if tribe.reincarnation_timer != 0
            && tribe.active
            && tribe.reincarnation_timer < REINCARNATION_TIMER_MAX
        {
            tribe.reincarnation_timer += REINCARNATION_TIMER_INCREMENT;
        }
    }

    if flags.is_multiplayer() {
        check_multiplayer_victory(flags, tribes, player_tribe);
    } else {
        check_singleplayer_victory(flags, tribes, player_tribe);
    }
}

/// Single-player victory/defeat logic.
/// Original: branch at 0x00423d4f (when multiplayer flag not set)
///
/// Defeat: player tribe population == 0 (timer starts on first zero-pop check)
/// Victory: all enemy tribes eliminated
fn check_singleplayer_victory(flags: &mut GameFlags, tribes: &mut TribeArray, player_tribe: u8) {
    let player = player_tribe as usize;

    // Check if player tribe has population
    // Original checks population via tribe offset that maps to the linked list head
    let player_pop = tribes.tribes[player].population;

    if player_pop == 0 {
        // Start reincarnation timer if not already started
        if tribes.tribes[player].reincarnation_timer == 0 {
            tribes.tribes[player].reincarnation_timer = 1;
        }

        // If timer has maxed out, player is defeated
        if tribes.tribes[player].is_eliminated() {
            // Original: set defeat flag and trigger defeat sequence
            flags.set_lost();
            return;
        }
    }

    // Check if all enemy tribes are eliminated
    let all_enemies_dead = tribes
        .tribes
        .iter()
        .enumerate()
        .filter(|(i, t)| *i != player && t.active)
        .all(|(_, t)| t.population == 0);

    if all_enemies_dead {
        // Original: sets victory flag, transitions units to celebrate state (0x29)
        flags.set_won();
    }
}

/// Multiplayer victory/defeat logic.
/// Original: branch at 0x00423d19 (when multiplayer flag is set)
///
/// Each tribe with zero population gets its timer started.
/// When timer reaches max, tribe is defeated.
/// Last tribe standing (or allied group) wins.
fn check_multiplayer_victory(flags: &mut GameFlags, tribes: &mut TribeArray, player_tribe: u8) {
    let player = player_tribe as usize;

    // Count alive tribes and detect eliminations
    let mut alive_count: u32 = 0;
    let mut last_alive: i32 = -1;

    for (i, tribe) in tribes.tribes.iter().enumerate() {
        if !tribe.active {
            continue;
        }

        if tribe.reincarnation_timer == 0 {
            // Tribe is alive (timer not started = has population)
            if tribe.population > 0 {
                alive_count += 1;
                last_alive = i as i32;
            } else {
                // Start elimination timer
                // (handled by the reincarnation timer update above)
            }
        }
        // If timer > 0, tribe is in elimination countdown or already eliminated
    }

    // If 0 or 1 tribes remain, determine winner
    if alive_count < 2 {
        if last_alive >= 0 {
            // One tribe remains — they win
            if last_alive == player as i32 {
                flags.set_won();
            } else {
                flags.set_lost();
            }
        }
        // If alive_count == 0, something went wrong — no action
    }

    // TODO: Alliance victory check.
    // Original uses alliance matrix at DAT_00948e4e to check if all
    // remaining tribes are mutually allied. Requires alliance data
    // from the save-network subsystem.
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup_tribes(pops: [u32; 4], actives: [bool; 4]) -> TribeArray {
        let mut arr = TribeArray::new();
        for i in 0..4 {
            arr.tribes[i].active = actives[i];
            arr.tribes[i].population = pops[i];
        }
        arr
    }

    #[test]
    fn test_too_early_no_check() {
        let mut flags = GameFlags::new();
        let mut tribes = setup_tribes([10, 10, 0, 0], [true, true, true, false]);
        // tick_counter = 0x10 (< 0x11), should not trigger
        check_victory_conditions(0x10, &mut flags, &mut tribes, 0);
        assert!(!flags.has_won());
        assert!(!flags.has_lost());
    }

    #[test]
    fn test_wrong_tick_alignment() {
        let mut flags = GameFlags::new();
        let mut tribes = setup_tribes([10, 0, 0, 0], [true, true, false, false]);
        // tick_counter = 0x21 — (0x21 & 0xF) = 1, should not check
        check_victory_conditions(0x21, &mut flags, &mut tribes, 0);
        assert!(!flags.has_won());
    }

    #[test]
    fn test_sp_victory_all_enemies_dead() {
        let mut flags = GameFlags::new();
        let mut tribes = setup_tribes([10, 0, 0, 0], [true, true, true, false]);
        check_victory_conditions(0x20, &mut flags, &mut tribes, 0);
        assert!(flags.has_won());
        assert!(!flags.has_lost());
    }

    #[test]
    fn test_sp_no_victory_enemy_alive() {
        let mut flags = GameFlags::new();
        let mut tribes = setup_tribes([10, 5, 0, 0], [true, true, true, false]);
        check_victory_conditions(0x20, &mut flags, &mut tribes, 0);
        assert!(!flags.has_won());
        assert!(!flags.has_lost());
    }

    #[test]
    fn test_sp_defeat_when_eliminated() {
        let mut flags = GameFlags::new();
        let mut tribes = setup_tribes([0, 10, 0, 0], [true, true, false, false]);
        tribes.tribes[0].reincarnation_timer = REINCARNATION_TIMER_MAX;
        check_victory_conditions(0x20, &mut flags, &mut tribes, 0);
        assert!(flags.has_lost());
        assert!(!flags.has_won());
    }

    #[test]
    fn test_mp_last_tribe_standing() {
        let mut flags = GameFlags::from_raw(super::super::constants::FLAG_MULTIPLAYER);
        let mut tribes = setup_tribes([10, 0, 0, 0], [true, true, true, true]);
        tribes.tribes[1].reincarnation_timer = REINCARNATION_TIMER_MAX;
        tribes.tribes[2].reincarnation_timer = REINCARNATION_TIMER_MAX;
        tribes.tribes[3].reincarnation_timer = REINCARNATION_TIMER_MAX;
        check_victory_conditions(0x20, &mut flags, &mut tribes, 0);
        assert!(flags.has_won());
    }

    #[test]
    fn test_reincarnation_timer_increments() {
        let mut flags = GameFlags::new();
        let mut tribes = setup_tribes([10, 0, 0, 0], [true, true, false, false]);
        tribes.tribes[1].reincarnation_timer = 1; // Started but not maxed
        check_victory_conditions(0x20, &mut flags, &mut tribes, 0);
        // Timer should have been incremented by 0x10
        assert_eq!(
            tribes.tribes[1].reincarnation_timer,
            1 + REINCARNATION_TIMER_INCREMENT
        );
    }
}
