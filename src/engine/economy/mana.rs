// Mana generation rates and pool management.
// Original binary constants from things-to-implement.md section 22.

// TODO: implement constants and functions

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
