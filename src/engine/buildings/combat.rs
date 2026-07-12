use super::types::*;
use crate::engine::objects::ObjectHandle;

pub const MAX_BUILDING_FIGHTERS: usize = 6;

/// Combat action emitted by building combat tick.
#[derive(Debug, PartialEq)]
pub enum BuildingCombatAction {
    None,
    AttackTarget {
        attacker_slot: usize,
        target: ObjectHandle,
        damage: u16,
    },
}

/// Tick building combat. For each fighting occupant, select target and emit attack.
/// Original: Building_ProcessFightingPersons at 0x438610
pub fn tick_building_combat(
    building: &mut BuildingData,
    _building_handle: ObjectHandle,
) -> Vec<BuildingCombatAction> {
    let mut actions = Vec::new();
    if building.state != BuildingState::Active {
        return actions;
    }
    if building.num_fighting == 0 {
        return actions;
    }

    // Each occupied fighter slot attacks the building's target
    if let Some(target) = building.target_person {
        for (i, slot) in building.occupant_slots.iter().enumerate() {
            if slot.is_some() && i < building.num_fighting as usize {
                actions.push(BuildingCombatAction::AttackTarget {
                    attacker_slot: i,
                    target,
                    damage: 100, // base building combat damage
                });
            }
        }
    }
    actions
}

/// Set number of fighting occupants (up to occupant_count).
pub fn set_fighters(building: &mut BuildingData, count: u8) {
    building.num_fighting = count.min(building.occupant_count);
}

/// Set the building's combat target.
pub fn set_building_target(building: &mut BuildingData, target: Option<ObjectHandle>) {
    building.target_person = target;
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn h(slot: u16) -> ObjectHandle {
        ObjectHandle::new(slot, 1)
    }

    fn make_combat_building() -> BuildingData {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.building_subtype = BuildingSubtype::DrumTower;
        b.behavior_flags = 0x08;
        b.occupant_slots[0] = Some(h(10));
        b.occupant_slots[1] = Some(h(20));
        b.occupant_slots[2] = Some(h(30));
        b.occupant_count = 3;
        b.num_fighting = 2;
        b.target_person = Some(h(99));
        b
    }

    #[test]
    fn max_building_fighters_is_six() {
        assert_eq!(MAX_BUILDING_FIGHTERS, 6);
    }

    #[test]
    fn combat_emits_attacks_per_fighter() {
        let mut b = make_combat_building();
        let actions = tick_building_combat(&mut b, h(1));
        // 2 fighters, slots 0 and 1
        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[0],
            BuildingCombatAction::AttackTarget {
                attacker_slot: 0,
                target: h(99),
                damage: 100
            }
        );
        assert_eq!(
            actions[1],
            BuildingCombatAction::AttackTarget {
                attacker_slot: 1,
                target: h(99),
                damage: 100
            }
        );
    }

    #[test]
    fn combat_no_actions_without_target() {
        let mut b = make_combat_building();
        b.target_person = None;
        let actions = tick_building_combat(&mut b, h(1));
        assert!(actions.is_empty());
    }

    #[test]
    fn combat_no_actions_zero_fighters() {
        let mut b = make_combat_building();
        b.num_fighting = 0;
        let actions = tick_building_combat(&mut b, h(1));
        assert!(actions.is_empty());
    }

    #[test]
    fn combat_no_actions_non_active() {
        let mut b = make_combat_building();
        b.state = BuildingState::Init;
        let actions = tick_building_combat(&mut b, h(1));
        assert!(actions.is_empty());
    }

    #[test]
    fn set_fighters_capped_at_occupant_count() {
        let mut b = BuildingData::default();
        b.occupant_count = 3;
        set_fighters(&mut b, 5);
        assert_eq!(b.num_fighting, 3);
    }

    #[test]
    fn set_fighters_allows_less_than_count() {
        let mut b = BuildingData::default();
        b.occupant_count = 4;
        set_fighters(&mut b, 2);
        assert_eq!(b.num_fighting, 2);
    }

    #[test]
    fn set_building_target_sets_and_clears() {
        let mut b = BuildingData::default();
        set_building_target(&mut b, Some(h(42)));
        assert_eq!(b.target_person, Some(h(42)));
        set_building_target(&mut b, None);
        assert_eq!(b.target_person, None);
    }

    #[test]
    fn combat_all_six_fighters() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.target_person = Some(h(50));
        for i in 0..6 {
            b.occupant_slots[i] = Some(h((i + 1) as u16));
        }
        b.occupant_count = 6;
        b.num_fighting = 6;
        let actions = tick_building_combat(&mut b, h(1));
        assert_eq!(actions.len(), 6);
    }
}
