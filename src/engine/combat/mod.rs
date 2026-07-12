pub mod damage;
pub mod death;
pub mod knockback;
pub mod projectile;

pub use damage::{apply_combat_damage, fight_damage_for_subtype, melee_damage};
pub use death::{process_death, should_drum_tower_fire, DeathActions, DRUM_TOWER_RANGE};
pub use knockback::{apply_knockback, decay_knockback};
pub use projectile::{drum_tower_shot, tick_projectile, ProjectileResult, ShotData};
