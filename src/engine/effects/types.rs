/// Effect type identifiers matching original binary ranges.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectType {
    // Spell impacts (0x01-0x1F)
    BurnFlame = 0x01,
    BlastProjectile = 0x02,
    LightningBolt = 0x03,
    // Death/combat (0x30-0x3F)
    DeathPuff = 0x30,
    BloodSpray = 0x32,
    HitSpark = 0x33,
    KnockbackTrail = 0x34,
    // Building (0x50-0x57)
    ConstructionDust = 0x50,
    DestructionCollapse = 0x51,
    BuildingFire = 0x52,
}

/// Broad category for effect grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectCategory {
    Spell,
    Combat,
    Building,
    Particle,
}

/// Returns (max_frame, flags, scale, alpha) defaults for an effect type.
///
/// - max_frame: total animation frames
/// - flags: combination of EFFECT_GRAVITY, EFFECT_LOOP, etc.
/// - scale: render scale (0x100 = 100%)
/// - alpha: transparency (0x100 = fully opaque)
pub fn effect_defaults(effect_type: u8) -> (i16, u8, i16, i16) {
    use super::{EFFECT_GRAVITY, EFFECT_LOOP};

    match effect_type {
        // Spell impacts
        0x01 => (16, EFFECT_LOOP, 0x100, 0xC0),           // BurnFlame: looping, slightly transparent
        0x02 => (12, EFFECT_GRAVITY, 0x100, 0x100),        // BlastProjectile: gravity arc
        0x03 => (8, 0, 0x100, 0x100),                      // LightningBolt: no special flags
        // Death/combat
        0x30 => (20, EFFECT_GRAVITY, 0x80, 0x80),          // DeathPuff: gravity, half size, half alpha
        0x32 => (10, EFFECT_GRAVITY, 0x60, 0xC0),          // BloodSpray: gravity, small
        0x33 => (6, 0, 0x100, 0x100),                      // HitSpark: quick flash
        0x34 => (8, 0, 0x80, 0x80),                        // KnockbackTrail: small, fading
        // Building
        0x50 => (24, 0, 0x100, 0x80),                      // ConstructionDust: long, transparent
        0x51 => (32, EFFECT_GRAVITY, 0x100, 0x100),         // DestructionCollapse: long, gravity
        0x52 => (16, EFFECT_LOOP, 0x100, 0xC0),            // BuildingFire: looping
        // Default for unknown types
        _ => (10, 0, 0x100, 0x100),
    }
}
