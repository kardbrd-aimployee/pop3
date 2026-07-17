//! Shared level-special configuration from the original `LEVLSPC2.DAT`.
//!
//! The game copies the first 0x38-byte player template from this file into a
//! new game's player state (`popTB.exe` `0x0046215a`).  In particular, the
//! dword at template offset four becomes the construction-command capability
//! mask read by the house-tab setup callback at `0x00435ef0`.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

/// Bytes copied by the original into its shared player-state template.
pub const SHARED_PLAYER_TEMPLATE_LEN: usize = 0x38;

/// Offset of the house-tab construction capability bits in that template.
const CONSTRUCTION_CAPABILITY_MASK_OFFSET: usize = 4;

/// The source-backed initial player configuration shared by all levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelSpecialData {
    shared_player_template: [u8; SHARED_PLAYER_TEMPLATE_LEN],
}

impl LevelSpecialData {
    /// Decode the prefix consumed by the original new-game initialization.
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        let Some(source) = data.get(..SHARED_PLAYER_TEMPLATE_LEN) else {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "LEVLSPC2.DAT is {} bytes; expected at least {SHARED_PLAYER_TEMPLATE_LEN}",
                    data.len()
                ),
            ));
        };
        let mut shared_player_template = [0; SHARED_PLAYER_TEMPLATE_LEN];
        shared_player_template.copy_from_slice(source);
        Ok(Self {
            shared_player_template,
        })
    }

    /// Load the original shared level-special file from a game-data root.
    pub fn from_base(base: &Path) -> io::Result<Self> {
        let data = fs::read(base.join("levels").join("levlspc2.dat"))?;
        Self::from_bytes(&data)
    }

    /// Return the native construction-command capability bitfield for a new
    /// game. This is the exact dword the original HUD reads at player+4.
    pub fn initial_construction_capabilities(&self) -> u32 {
        let bytes = self.shared_player_template
            [CONSTRUCTION_CAPABILITY_MASK_OFFSET..CONSTRUCTION_CAPABILITY_MASK_OFFSET + 4]
            .try_into()
            .expect("construction capability field has a fixed four-byte width");
        u32::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::{LevelSpecialData, SHARED_PLAYER_TEMPLATE_LEN};

    #[test]
    fn parses_native_construction_capability_mask() {
        let mut bytes = vec![0_u8; SHARED_PLAYER_TEMPLATE_LEN];
        bytes[4..8].copy_from_slice(&0x0000_0012_u32.to_le_bytes());

        let special = LevelSpecialData::from_bytes(&bytes).unwrap();

        assert_eq!(special.initial_construction_capabilities(), 0x12);
    }

    #[test]
    fn rejects_truncated_shared_player_template() {
        let error = LevelSpecialData::from_bytes(&[0; SHARED_PLAYER_TEMPLATE_LEN - 1]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    }
}
