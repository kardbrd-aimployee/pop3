use super::constants::*;

/// Per-tribe game data.
///
/// In the original binary, tribes are stored at g_TribeArray (0x00885760),
/// each 0xC65 (3173) bytes. This struct captures the fields needed by
/// the game-state subsystem. Additional fields will be added as other
/// subsystems (person-units, buildings, AI) come online.
#[derive(Debug, Clone)]
pub struct TribeData {
    /// Tribe index (0=Blue, 1=Red, 2=Yellow, 3=Green).
    pub index: u8,

    /// Whether this tribe is active in the current level.
    /// Original: tribe struct offset +0xC20
    pub active: bool,

    /// Reincarnation/elimination timer.
    /// Original: tribe struct offset +0x949
    ///
    /// When population reaches 0, this increments by 0x10 each victory check.
    /// When it reaches REINCARNATION_TIMER_MAX (0x60), the tribe is eliminated.
    /// Non-zero population resets this to 0.
    pub reincarnation_timer: i32,

    /// Victory state flags.
    /// Original: tribe struct offset +0x941
    /// Bit 0: victory celebration triggered.
    pub victory_flags: u32,

    /// Current population count (persons alive for this tribe).
    pub population: u32,

    // -- Economy fields --

    /// Current mana pool. Capped at economy::mana::MAX_MANA (1,000,000).
    pub mana: u32,

    /// Housing capacity (recalculated from hut counts each tick).
    /// Capped at economy::population::MAX_POP_VALUE (199).
    pub max_population: u16,

    /// Total wood gathered this game (for stats tracking).
    pub wood_gathered: u32,
}

impl TribeData {
    pub fn new(index: u8) -> Self {
        Self {
            index,
            active: false,
            reincarnation_timer: 0,
            victory_flags: 0,
            population: 0,
            mana: 0,
            max_population: 0,
            wood_gathered: 0,
        }
    }

    /// Check if this tribe has been eliminated (population 0 and timer maxed).
    pub fn is_eliminated(&self) -> bool {
        self.population == 0 && self.reincarnation_timer >= REINCARNATION_TIMER_MAX
    }
}

/// Array of all tribes. Always exactly MAX_TRIBES (4).
/// Original: g_TribeArray at 0x00885760
#[derive(Debug, Clone)]
pub struct TribeArray {
    pub tribes: [TribeData; MAX_TRIBES],
}

impl TribeArray {
    pub fn new() -> Self {
        Self {
            tribes: [
                TribeData::new(0),
                TribeData::new(1),
                TribeData::new(2),
                TribeData::new(3),
            ],
        }
    }

    /// Count how many tribes are active and alive (not eliminated).
    pub fn alive_count(&self) -> usize {
        self.tribes.iter()
            .filter(|t| t.active && !t.is_eliminated())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tribe_not_eliminated() {
        let t = TribeData::new(0);
        // population is 0 but timer is also 0 (< max), so not eliminated yet
        assert!(!t.is_eliminated());
    }

    #[test]
    fn test_tribe_eliminated_when_timer_maxed() {
        let mut t = TribeData::new(0);
        t.population = 0;
        t.reincarnation_timer = REINCARNATION_TIMER_MAX;
        assert!(t.is_eliminated());
    }

    #[test]
    fn test_tribe_not_eliminated_with_population() {
        let mut t = TribeData::new(0);
        t.population = 5;
        t.reincarnation_timer = REINCARNATION_TIMER_MAX;
        assert!(!t.is_eliminated());
    }

    #[test]
    fn test_alive_count() {
        let mut arr = TribeArray::new();
        arr.tribes[0].active = true;
        arr.tribes[0].population = 10;
        arr.tribes[1].active = true;
        arr.tribes[1].population = 5;
        arr.tribes[2].active = true;
        arr.tribes[2].population = 0;
        arr.tribes[2].reincarnation_timer = REINCARNATION_TIMER_MAX;
        arr.tribes[3].active = false;
        assert_eq!(arr.alive_count(), 2);
    }
}
