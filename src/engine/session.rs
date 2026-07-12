use std::collections::VecDeque;

use crate::data::level::LevelDefinition;
use crate::engine::buildings::{BuildingCatalog, BuildingSubtype};
use crate::engine::movement::WorldCoord;
use crate::engine::objects::ObjectHandle;
use crate::engine::state::flags::GameFlags;
use crate::engine::state::rng::GameRng;
use crate::engine::state::tick::TimeSource;
use crate::engine::state::victory;
use crate::engine::world::{LevelInitError, World, WorldError, WorldSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameAction {
    Select(Vec<ObjectHandle>),
    Move {
        units: Vec<ObjectHandle>,
        target: WorldCoord,
    },
    PlaceBuilding {
        subtype: BuildingSubtype,
        owner: u8,
        cell: (i32, i32),
        rotation: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionEvent {
    Applied(GameAction),
    Rejected {
        action: GameAction,
        reason: WorldError,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickReport {
    pub ticks: u32,
    pub actions: Vec<ActionEvent>,
}

/// Pool-backed person algorithms and future pathfinding caches. It owns no
/// persons; every update resolves a generational handle through `World`.
#[derive(Default)]
pub struct PersonSystem;

impl PersonSystem {
    pub fn tick(&mut self, world: &mut World) {
        world.tick_persons();
    }
}

pub struct GameSession {
    pub world: World,
    pub flags: GameFlags,
    pub rng: GameRng,
    pub game_tick: u32,
    pub game_speed: u32,
    pub player_tribe: u8,
    person_system: PersonSystem,
    actions: VecDeque<GameAction>,
    last_time_ms: Option<u64>,
}

impl GameSession {
    pub fn from_level(
        level: LevelDefinition,
        catalog: BuildingCatalog,
    ) -> Result<Self, LevelInitError> {
        Ok(Self {
            world: World::from_level(level, catalog)?,
            flags: GameFlags::new(),
            rng: GameRng::new(0),
            game_tick: 0,
            game_speed: 30,
            player_tribe: 0,
            person_system: PersonSystem,
            actions: VecDeque::new(),
            last_time_ms: None,
        })
    }

    pub fn enqueue(&mut self, action: GameAction) {
        self.actions.push_back(action);
    }

    pub fn validate_building_placement(
        &self,
        subtype: BuildingSubtype,
        cell: (i32, i32),
        rotation: u8,
    ) -> Result<(), WorldError> {
        self.world
            .validate_building_placement(subtype, cell, rotation)
    }

    pub fn update(&mut self, time: &dyn TimeSource) -> TickReport {
        if self.flags.is_paused() {
            return TickReport::default();
        }
        let now = time.now_ms();
        let Some(last) = self.last_time_ms else {
            self.last_time_ms = Some(now);
            return self.step();
        };
        let interval = 1000 / self.game_speed.max(1) as u64;
        let elapsed = now.saturating_sub(last);
        if elapsed < interval {
            return TickReport::default();
        }
        let ticks = (elapsed / interval).min(8) as u32;
        self.last_time_ms = Some(last.saturating_add(interval * ticks as u64));
        let mut report = TickReport::default();
        for _ in 0..ticks {
            let one = self.step();
            report.ticks += one.ticks;
            report.actions.extend(one.actions);
        }
        report
    }

    pub fn step(&mut self) -> TickReport {
        let mut report = TickReport {
            ticks: 1,
            actions: Vec::new(),
        };
        while let Some(action) = self.actions.pop_front() {
            let result = self.apply_action(&action);
            report.actions.push(match result {
                Ok(()) => ActionEvent::Applied(action),
                Err(reason) => ActionEvent::Rejected { action, reason },
            });
        }
        // Queued terrain work is committed atomically by actions above.
        self.person_system.tick(&mut self.world);
        self.world.tick_buildings();
        // Projectile and water systems are intentionally empty for this slice.
        self.world.synchronize_tribes();
        self.world.add_mana();
        self.game_tick = self.game_tick.wrapping_add(1);
        victory::check_victory_conditions(
            self.game_tick,
            &mut self.flags,
            &mut self.world.tribes,
            self.player_tribe,
        );
        report
    }

    fn apply_action(&mut self, action: &GameAction) -> Result<(), WorldError> {
        match action {
            GameAction::Select(handles) => {
                self.world.select(handles.clone());
                Ok(())
            }
            GameAction::Move { units, target } => {
                for handle in units {
                    let Some(object) = self.world.get(*handle) else {
                        return Err(WorldError::InvalidHandle(*handle));
                    };
                    if object.header.model_type != crate::data::units::ModelType::Person {
                        return Err(WorldError::InvalidHandle(*handle));
                    }
                }
                for handle in units {
                    if let Some(object) = self.world.get_mut_for_action(*handle) {
                        if let crate::engine::objects::GameObjectData::Person(person) =
                            &mut object.data
                        {
                            person.movement.target_pos = *target;
                            person.movement.flags1 |= 0x1000;
                        }
                    }
                }
                Ok(())
            }
            GameAction::PlaceBuilding {
                subtype,
                owner,
                cell,
                rotation,
            } => self
                .world
                .place_building(*subtype, *owner, *cell, *rotation)
                .map(|_| ()),
        }
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        self.world.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::level::{LevelDefinition, Sunlight};
    use crate::data::units::TribeConfigRaw;
    use crate::engine::buildings::PlacementError;

    struct FakeTime(u64);
    impl TimeSource for FakeTime {
        fn now_ms(&self) -> u64 {
            self.0
        }
    }

    fn session() -> GameSession {
        let mut catalog = BuildingCatalog::new();
        catalog.insert(BuildingSubtype::SmallHut, 0, vec![(0, 0)]);
        GameSession::from_level(
            LevelDefinition {
                level_number: 1,
                heights: Box::new([[100; 128]; 128]),
                sunlight: Sunlight::new(0, 0, 0),
                tribes: vec![TribeConfigRaw { data: [0; 16] }; 4],
                objects: Vec::new(),
            },
            catalog,
        )
        .unwrap()
    }

    #[test]
    fn actions_are_fifo_and_rejections_do_not_partially_mutate() {
        let mut session = session();
        session.enqueue(GameAction::PlaceBuilding {
            subtype: BuildingSubtype::SmallHut,
            owner: 0,
            cell: (4, 4),
            rotation: 0,
        });
        session.enqueue(GameAction::PlaceBuilding {
            subtype: BuildingSubtype::SmallHut,
            owner: 0,
            cell: (4, 4),
            rotation: 0,
        });
        let report = session.step();
        assert!(matches!(report.actions[0], ActionEvent::Applied(_)));
        assert!(matches!(
            report.actions[1],
            ActionEvent::Rejected {
                reason: WorldError::InvalidPlacement(PlacementError::Occupied),
                ..
            }
        ));
        assert_eq!(session.world.pool().buildings().count(), 1);
    }

    #[test]
    fn update_accumulates_sub_tick_frame_time() {
        let mut session = session();
        assert_eq!(session.update(&FakeTime(0)).ticks, 1);
        assert_eq!(session.update(&FakeTime(10)).ticks, 0);
        assert_eq!(session.update(&FakeTime(20)).ticks, 0);
        assert_eq!(session.update(&FakeTime(34)).ticks, 1);
    }
}
