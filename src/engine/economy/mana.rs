// Mana generation rates and pool management.
// Original binary constants are documented in docs/specs/spells.md (Appendix N).

/// Per-unit mana generation rates (per tick).
pub const MANA_F_BRAVE: u32 = 1;
pub const MANA_F_WARR: u32 = 1;
pub const MANA_F_SPY: u32 = 1;
pub const MANA_F_PREACH: u32 = 2;
pub const MANA_F_SWARR: u32 = 1;
pub const MANA_F_SHAMEN: u32 = 1;

/// Per-housing-level mana generation rates.
pub const MANA_F_HUT_LEVEL_1: u32 = 1;
pub const MANA_F_HUT_LEVEL_2: u32 = 2;
pub const MANA_F_HUT_LEVEL_3: u32 = 3;

/// Maximum mana pool per tribe. Original: 0xF4240.
pub const MAX_MANA: u32 = 1_000_000;

/// Spell mana costs indexed by spell panel position (0-15).
/// Values are placeholder estimates until extracted from constant.dat.
/// Order matches the spell bar: Burn, Blast, Lightning, Whirlwind,
/// Plague, Invisibility, Firestorm, Hypnotism,
/// Ghost Army, Erosion, Swamp, Land Bridge,
/// Angel of Death, Earthquake, Flatten, Volcano.
pub const SPELL_MANA_COSTS: [u32; 16] = [
    20_000, 80_000, 60_000, 40_000, 50_000, 30_000, 100_000, 40_000, 60_000, 30_000, 30_000,
    40_000, 150_000, 50_000, 20_000, 120_000,
];

/// Compute spell charges (number of casts affordable) for all 16 spells.
pub fn compute_spell_charges(current_mana: u32) -> [u8; 16] {
    let mut charges = [0u8; 16];
    for (i, &cost) in SPELL_MANA_COSTS.iter().enumerate() {
        if cost > 0 {
            charges[i] = (current_mana / cost).min(7) as u8;
        }
    }
    charges
}

/// Returns mana generation rate for a person subtype (per tick).
/// Subtypes: 1=Wild(0), 2=Brave, 3=Warrior, 4=Preacher, 5=Spy, 6=SuperWarrior, 7=Shaman
pub fn mana_rate_for_person(subtype: u8) -> u32 {
    match subtype {
        2 => MANA_F_BRAVE,
        3 => MANA_F_WARR,
        4 => MANA_F_PREACH,
        5 => MANA_F_SPY,
        6 => MANA_F_SWARR,
        7 => MANA_F_SHAMEN,
        _ => 0, // Wild and unknown generate no mana
    }
}

/// Returns mana generation rate for a housing level (1, 2, or 3).
pub fn mana_rate_for_housing(hut_level: u8) -> u32 {
    match hut_level {
        1 => MANA_F_HUT_LEVEL_1,
        2 => MANA_F_HUT_LEVEL_2,
        3 => MANA_F_HUT_LEVEL_3,
        _ => 0,
    }
}

/// Add mana to a tribe's pool, capped at MAX_MANA.
pub fn add_mana(current: &mut u32, amount: u32) {
    *current = (*current + amount).min(MAX_MANA);
}

/// Deduct mana from pool. Returns false if insufficient.
pub fn deduct_mana(current: &mut u32, amount: u32) -> bool {
    if *current >= amount {
        *current -= amount;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Constant tests --

    #[test]
    fn test_max_mana_value() {
        assert_eq!(MAX_MANA, 1_000_000);
    }

    #[test]
    fn test_mana_f_brave() {
        assert_eq!(MANA_F_BRAVE, 1);
    }

    #[test]
    fn test_mana_f_preach() {
        assert_eq!(MANA_F_PREACH, 2);
    }

    #[test]
    fn test_mana_f_warr() {
        assert_eq!(MANA_F_WARR, 1);
    }

    #[test]
    fn test_mana_f_spy() {
        assert_eq!(MANA_F_SPY, 1);
    }

    #[test]
    fn test_mana_f_swarr() {
        assert_eq!(MANA_F_SWARR, 1);
    }

    #[test]
    fn test_mana_f_shamen() {
        assert_eq!(MANA_F_SHAMEN, 1);
    }

    #[test]
    fn test_mana_hut_level_rates() {
        assert_eq!(MANA_F_HUT_LEVEL_1, 1);
        assert_eq!(MANA_F_HUT_LEVEL_2, 2);
        assert_eq!(MANA_F_HUT_LEVEL_3, 3);
    }

    // -- Function tests --

    #[test]
    fn test_mana_rate_for_person_brave() {
        assert_eq!(mana_rate_for_person(2), 1); // Brave
    }

    #[test]
    fn test_mana_rate_for_person_warrior() {
        assert_eq!(mana_rate_for_person(3), 1); // Warrior
    }

    #[test]
    fn test_mana_rate_for_person_preacher() {
        assert_eq!(mana_rate_for_person(4), 2); // Preacher
    }

    #[test]
    fn test_mana_rate_for_person_spy() {
        assert_eq!(mana_rate_for_person(5), 1); // Spy
    }

    #[test]
    fn test_mana_rate_for_person_super_warrior() {
        assert_eq!(mana_rate_for_person(6), 1); // Super Warrior / Firewarrior
    }

    #[test]
    fn test_mana_rate_for_person_shaman() {
        assert_eq!(mana_rate_for_person(7), 1); // Shaman
    }

    #[test]
    fn test_mana_rate_for_person_wild_is_zero() {
        assert_eq!(mana_rate_for_person(1), 0); // Wild generates no mana
    }

    #[test]
    fn test_mana_rate_for_person_unknown_is_zero() {
        assert_eq!(mana_rate_for_person(0), 0);
        assert_eq!(mana_rate_for_person(99), 0);
    }

    #[test]
    fn test_mana_rate_for_housing() {
        assert_eq!(mana_rate_for_housing(1), 1);
        assert_eq!(mana_rate_for_housing(2), 2);
        assert_eq!(mana_rate_for_housing(3), 3);
        assert_eq!(mana_rate_for_housing(0), 0);
        assert_eq!(mana_rate_for_housing(4), 0);
    }

    #[test]
    fn test_add_mana_normal() {
        let mut mana = 500;
        add_mana(&mut mana, 100);
        assert_eq!(mana, 600);
    }

    #[test]
    fn test_add_mana_caps_at_max() {
        let mut mana = MAX_MANA - 10;
        add_mana(&mut mana, 20);
        assert_eq!(mana, MAX_MANA);
    }

    #[test]
    fn test_add_mana_already_at_max() {
        let mut mana = MAX_MANA;
        add_mana(&mut mana, 100);
        assert_eq!(mana, MAX_MANA);
    }

    #[test]
    fn test_deduct_mana_sufficient() {
        let mut mana = 500;
        assert!(deduct_mana(&mut mana, 200));
        assert_eq!(mana, 300);
    }

    #[test]
    fn test_deduct_mana_insufficient() {
        let mut mana = 100;
        assert!(!deduct_mana(&mut mana, 200));
        assert_eq!(mana, 100); // unchanged
    }

    #[test]
    fn test_deduct_mana_exact() {
        let mut mana = 100;
        assert!(deduct_mana(&mut mana, 100));
        assert_eq!(mana, 0);
    }
}
