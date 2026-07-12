/// Constants from the original Populous: The Beginning binary (popTB.exe).
/// Addresses reference the Win32 x86 executable analyzed in Ghidra.

// --- Object Limits ---

/// Maximum active objects in the game world.
/// Original: 0x44D
pub const MAX_ACTIVE_OBJECTS: u32 = 1101;

/// Low priority object pool size (effects/particles).
/// Original: 0x280
pub const LOW_PRIORITY_POOL: u32 = 640;

/// Maximum number of tribes.
pub const MAX_TRIBES: usize = 4;

/// Maximum number of players.
pub const MAX_PLAYERS: usize = 4;

// --- Terrain ---

/// Map grid size in each dimension.
pub const MAP_SIZE: usize = 128;

/// Total cells in the map.
pub const MAP_CELLS: usize = MAP_SIZE * MAP_SIZE;

// --- Angles ---

/// Full rotation in the game's angle system (11-bit).
pub const ANGLE_FULL: u32 = 0x800;

/// Quarter rotation.
pub const ANGLE_QUARTER: u32 = 0x200;

// --- Timing ---

/// Default tick interval calculation base (1000ms).
pub const TICK_BASE_MS: i32 = 1000;

/// Maximum ticks to catch up in a single-player frame.
/// Observed in the disassembly at 0x004bb7d2: CMP EBX,0x4
pub const MAX_CATCHUP_TICKS: i32 = 4;

// --- Game Flags (g_GameFlags at 0x00884bf9) ---

/// Game is paused.
pub const FLAG_PAUSED: u32 = 0x02;

/// Multiplayer mode active.
pub const FLAG_MULTIPLAYER: u32 = 0x08;

/// Network is waiting for other players.
pub const FLAG_NET_WAITING: u32 = 0x20;

/// Network game is paused.
pub const FLAG_NET_PAUSED: u32 = 0x40;

/// Victory or defeat state is active (used to skip AI updates).
pub const FLAG_VICTORY_DEFEAT: u32 = 0x800000;

/// Player has won the game.
pub const FLAG_PLAYER_WON: u32 = 0x2000000;

/// Player has lost the game.
pub const FLAG_PLAYER_LOST: u32 = 0x4000000;

/// Mask to clear both victory and defeat bits.
pub const FLAG_CLEAR_VICTORY_DEFEAT: u32 = 0xF9FF_FFFF;

// --- Victory System ---

/// Tick counter mask for victory check frequency.
/// Victory is only checked when (tick_counter & 0xF) == 0 and tick >= 0x11.
/// Original: 0x00423c60
pub const VICTORY_CHECK_MASK: u32 = 0x0F;

/// Minimum ticks before victory checking begins.
pub const VICTORY_CHECK_MIN_TICKS: u32 = 0x11;

/// Maximum reincarnation timer value.
/// Original: 0x60
pub const REINCARNATION_TIMER_MAX: i32 = 0x60;

/// Reincarnation timer increment per tick.
/// Original: 0x10
pub const REINCARNATION_TIMER_INCREMENT: i32 = 0x10;

// --- Tribe Structure Offsets ---
// These document the original struct layout at g_TribeArray (0x00885760).
// Each tribe is 0xC65 (3173) bytes.

/// Size of one tribe structure in bytes.
pub const TRIBE_STRUCT_SIZE: usize = 0xC65;

/// Offset to reincarnation/elimination timer within tribe struct.
pub const TRIBE_OFF_REINCARNATION: usize = 0x949;

/// Offset to victory state flags within tribe struct.
pub const TRIBE_OFF_VICTORY_FLAGS: usize = 0x941;

/// Offset to tribe active flag within tribe struct.
pub const TRIBE_OFF_ACTIVE: usize = 0xC20;

// --- Network Packet Types ---

/// Game state sync packet.
pub const NET_PACKET_STATE_SYNC: u8 = 0x06;

/// Tick acknowledgment.
pub const NET_PACKET_TICK_ACK: u8 = 0x07;

/// Time synchronization.
pub const NET_PACKET_TIME_SYNC: u8 = 0x0D;

/// Fast tick acknowledgment.
pub const NET_PACKET_FAST_ACK: u8 = 0x0E;

/// Game state sync packet size.
pub const NET_PACKET_STATE_SYNC_SIZE: usize = 0x55;

// --- Person States ---

/// Celebration/victory dance state.
pub const PERSON_STATE_CELEBRATE: u8 = 0x29;

// --- Person Subtypes (re-exported from data layer) ---
pub use crate::data::constants::{
    PERSON_SUBTYPE_AOD, PERSON_SUBTYPE_BRAVE, PERSON_SUBTYPE_FIREWARRIOR, PERSON_SUBTYPE_PREACHER,
    PERSON_SUBTYPE_SHAMAN, PERSON_SUBTYPE_SPY, PERSON_SUBTYPE_WARRIOR, PERSON_SUBTYPE_WILD,
};

// --- RNG ---

/// LCG multiplier for the random number generator.
/// Original: g_RandomSeed = g_RandomSeed * 0x24a1 + 0x24df
pub const RNG_MULTIPLIER: u32 = 0x24A1;

/// LCG increment.
pub const RNG_INCREMENT: u32 = 0x24DF;

/// RNG bit rotation: shift right by 13, left by 19.
pub const RNG_SHIFT_RIGHT: u32 = 13;
pub const RNG_SHIFT_LEFT: u32 = 19;

// --- Tutorial Mode Values ---
// At 0x00884119

/// Tutorial mode marker value 1.
pub const TUTORIAL_MODE_2: u8 = 0x02;

/// Tutorial mode marker value 2.
pub const TUTORIAL_MODE_3: u8 = 0x03;
