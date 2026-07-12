use super::constants::*;

/// Faithful LCG matching g_RandomSeed behavior (0x00885710).
///
/// Algorithm from the original binary:
/// ```text
/// seed = seed * 0x24A1 + 0x24DF
/// seed = (seed >> 13) | (seed << 19)   // rotate right by 13
/// ```
///
/// Used for AI decisions, spawn randomization, and other game logic.
/// Must be deterministic for multiplayer lockstep synchronization.
#[derive(Debug, Clone)]
pub struct GameRng {
    seed: u32,
}

impl GameRng {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// Advance the RNG and return the new seed value.
    /// Original: inline at multiple call sites throughout the binary.
    pub fn next(&mut self) -> u32 {
        self.seed = self
            .seed
            .wrapping_mul(RNG_MULTIPLIER)
            .wrapping_add(RNG_INCREMENT);
        self.seed = self.seed.rotate_right(RNG_SHIFT_RIGHT);
        self.seed
    }

    /// Return a value in [0, 100) — used for AI percentage checks.
    /// Original: `g_RandomSeed % 100` at attribute code 0x4D5.
    pub fn next_percent(&mut self) -> u32 {
        self.next() % 100
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut a = GameRng::new(12345);
        let mut b = GameRng::new(12345);
        for _ in 0..100 {
            assert_eq!(a.next(), b.next());
        }
    }

    #[test]
    fn test_different_seeds_diverge() {
        let mut a = GameRng::new(0);
        let mut b = GameRng::new(1);
        // After first step they should differ
        assert_ne!(a.next(), b.next());
    }

    #[test]
    fn test_zero_seed() {
        let mut rng = GameRng::new(0);
        // seed = 0 * 0x24A1 + 0x24DF = 0x24DF
        // rotate_right(0x24DF, 13) = (0x24DF >> 13) | (0x24DF << 19)
        // 0x24DF >> 13 = 0x1 (0x24DF = 9439, >> 13 = 1)
        // 0x24DF << 19 = 0x126F80000 & 0xFFFFFFFF = 0x26F80000
        // result = 0x26F80001
        let v = rng.next();
        assert_eq!(v, 0x26F8_0001);
    }

    #[test]
    fn test_percent_range() {
        let mut rng = GameRng::new(42);
        for _ in 0..1000 {
            let p = rng.next_percent();
            assert!(p < 100);
        }
    }
}
