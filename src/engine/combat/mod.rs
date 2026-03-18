pub mod projectile;
pub mod knockback;
pub mod damage;
pub mod death;

pub use projectile::{ShotData, tick_projectile, ProjectileResult, drum_tower_shot};
pub use knockback::{apply_knockback, decay_knockback};
pub use damage::{apply_combat_damage, melee_damage, fight_damage_for_subtype};
pub use death::{process_death, DeathActions, should_drum_tower_fire, DRUM_TOWER_RANGE};
