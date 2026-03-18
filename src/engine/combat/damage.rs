use crate::engine::objects::types::ObjectHeader;

/// Apply raw damage to an object header. Returns true if health reached 0.
pub fn apply_combat_damage(header: &mut ObjectHeader, damage: u16) -> bool {
    if header.health <= damage {
        header.health = 0;
        true
    } else {
        header.health -= damage;
        false
    }
}

/// Calculate melee damage matching original binary formula exactly.
/// damage = (FIGHT_DAMAGE[subtype] * health) / max_health, min 32
/// Original: inline in person combat tick
pub fn melee_damage(attacker_subtype: u8, attacker_health: u16, max_health: u16) -> u16 {
    let base = fight_damage_for_subtype(attacker_subtype) as u32;
    let hp = attacker_health as u32;
    let max_hp = max_health.max(1) as u32;
    let damage = (base * hp) / max_hp;
    damage.max(32) as u16
}

/// FIGHT_DAMAGE per person subtype from original binary.
/// Source: Unit Type Data Table at 0x0059FE44
pub fn fight_damage_for_subtype(subtype: u8) -> u16 {
    match subtype {
        1 => 64,   // Wild
        2 => 200,  // Brave
        3 => 400,  // Warrior
        4 => 150,  // Religious/Preacher
        5 => 200,  // Spy
        6 => 500,  // Super Warrior
        7 => 300,  // Shaman
        8 => 600,  // Angel of Death
        _ => 100,  // Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::units::ModelType;
    use crate::engine::movement::WorldCoord;

    fn make_header(health: u16, max_health: u16) -> ObjectHeader {
        ObjectHeader {
            model_type: ModelType::Person,
            subtype: 2,
            tribe: 0,
            state: 0,
            state_phase: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            object_index: 0,
            angle: 0,
            position: WorldCoord::default(),
            velocity: WorldCoord::default(),
            health,
            max_health,
            next_in_cell: None,
            prev_in_cell: None,
        }
    }

    #[test]
    fn melee_damage_warrior_full_health() {
        // Warrior (subtype 3) at full health: 400 * 1800 / 1800 = 400
        assert_eq!(melee_damage(3, 1800, 1800), 400);
    }

    #[test]
    fn melee_damage_brave_half_health() {
        // Brave (subtype 2) at half health: 200 * 700 / 1400 = 100
        assert_eq!(melee_damage(2, 700, 1400), 100);
    }

    #[test]
    fn melee_damage_minimum_floor() {
        // Brave (subtype 2) at very low health: 200 * 10 / 1400 = 1 -> clamped to 32
        assert_eq!(melee_damage(2, 10, 1400), 32);
    }

    #[test]
    fn melee_damage_all_subtypes() {
        // At full health, damage equals fight_damage
        assert_eq!(melee_damage(1, 100, 100), 64);   // Wild
        assert_eq!(melee_damage(2, 100, 100), 200);  // Brave
        assert_eq!(melee_damage(3, 100, 100), 400);  // Warrior
        assert_eq!(melee_damage(4, 100, 100), 150);  // Religious
        assert_eq!(melee_damage(5, 100, 100), 200);  // Spy
        assert_eq!(melee_damage(6, 100, 100), 500);  // Super Warrior
        assert_eq!(melee_damage(7, 100, 100), 300);  // Shaman
        assert_eq!(melee_damage(8, 100, 100), 600);  // Angel of Death
        assert_eq!(melee_damage(0, 100, 100), 100);  // Fallback
    }

    #[test]
    fn fight_damage_matches_person_type_defaults() {
        use crate::engine::units::person_state::person_type_defaults;
        for subtype in 1..=8u8 {
            let defaults = person_type_defaults(subtype);
            assert_eq!(
                fight_damage_for_subtype(subtype), defaults.fight_damage,
                "Mismatch for subtype {}", subtype
            );
        }
        // Fallback
        let defaults = person_type_defaults(0);
        assert_eq!(fight_damage_for_subtype(0), defaults.fight_damage);
    }

    #[test]
    fn apply_combat_damage_kills_when_damage_exceeds_health() {
        let mut header = make_header(100, 1000);
        let killed = apply_combat_damage(&mut header, 200);
        assert!(killed);
        assert_eq!(header.health, 0);
    }

    #[test]
    fn apply_combat_damage_kills_when_damage_equals_health() {
        let mut header = make_header(100, 1000);
        let killed = apply_combat_damage(&mut header, 100);
        assert!(killed);
        assert_eq!(header.health, 0);
    }

    #[test]
    fn apply_combat_damage_reduces_health_when_not_killed() {
        let mut header = make_header(500, 1000);
        let killed = apply_combat_damage(&mut header, 200);
        assert!(!killed);
        assert_eq!(header.health, 300);
    }

    #[test]
    fn melee_damage_zero_max_health_no_panic() {
        // Should not divide by zero
        let result = melee_damage(3, 100, 0);
        assert!(result >= 32);
    }
}
