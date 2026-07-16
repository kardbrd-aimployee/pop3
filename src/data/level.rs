use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use crate::data::types::BinDeserializer;
use crate::data::units::{ModelType, TribeConfigRaw, UnitRaw};

/******************************************************************************/

const LEVEL_UNIT_SLOTS: usize = 2000;

pub struct LevelPaths {
    pub palette: PathBuf,
    pub disp0: PathBuf,
    pub bigf0: PathBuf,
    pub cliff0: PathBuf,
    pub fade0: PathBuf,
    pub bl320: PathBuf,
    pub bl160: PathBuf,
    pub watdisp: PathBuf,
    pub sky: PathBuf,
}

fn mk_based_path(base: &Path, s: String) -> PathBuf {
    let mut base = base.to_path_buf();
    base.push(s);
    base
}

impl LevelPaths {
    pub fn from_base(base: &Path, key: &str) -> Self {
        let key_upper = key.to_uppercase();
        Self {
            palette: mk_based_path(base, format!("pal0-{key}.dat")),
            disp0: mk_based_path(base, format!("disp0-{key}.dat")),
            bigf0: mk_based_path(base, format!("bigf0-{key}.dat")),
            cliff0: mk_based_path(base, format!("cliff0-{key}.dat")),
            fade0: mk_based_path(base, format!("fade0-{key}.dat")),
            bl320: mk_based_path(base, format!("BL320-{key_upper}.DAT")),
            bl160: mk_based_path(base, format!("BL160-{key_upper}.DAT")),
            watdisp: mk_based_path(base, "watdisp.dat".to_string()),
            sky: mk_based_path(base, format!("sky0-{key}.dat")),
        }
    }

    pub fn from_default_dir(base: &Path, key: &str) -> Self {
        let data_dir = base.join("data");
        Self::from_base(&data_dir, key)
    }

    pub fn dat_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.dat"))
    }

    pub fn hdr_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.hdr"))
    }
}

pub struct ObjectPaths {
    pub objs0_dat: PathBuf,
    pub objs0_ver: PathBuf,
    pub pnts0: PathBuf,
    pub facs0: PathBuf,
    pub morph0: PathBuf,
    pub shapes: PathBuf,
}

impl ObjectPaths {
    pub fn from_base(base: &Path, key: &str) -> Self {
        Self {
            //objs0_dat: mk_based_path(base, format!("objs0-{key}.dat")),
            objs0_dat: mk_based_path(base, format!("OBJS0-{key}.DAT")),
            objs0_ver: mk_based_path(base, format!("objs0-{key}.ver")),
            pnts0: mk_based_path(base, format!("PNTS0-{key}.DAT")),
            facs0: mk_based_path(base, format!("FACS0-{key}.DAT")),
            morph0: mk_based_path(base, format!("morph0-{key}.dat")),
            shapes: mk_based_path(base, "SHAPES.DAT".to_string()),
        }
    }

    pub fn from_default_dir(base: &Path, key: &str) -> Self {
        let data_dir = base.join("objects");
        Self::from_base(&data_dir, key)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
pub struct Sunlight {
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
}

impl Sunlight {
    pub fn new(v1: u8, v2: u8, v3: u8) -> Self {
        Sunlight { v1, v2, v3 }
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Self {
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf).unwrap();
        Self::new(buf[0], buf[1], buf[2])
    }
}

/******************************************************************************/

pub struct LevelRes {
    pub level_number: u8,
    pub paths: LevelPaths,
    pub params: GlobeTextureParams,
    pub landscape: Landscape<128>,
    pub tribes: Vec<TribeConfigRaw>,
    pub sunlight: Sunlight,
    pub units: Vec<UnitRaw>,
    /// OBJS bank number (HDR byte 97). Selects which objs0-{N}.dat to load.
    pub obj_bank: u8,
}

impl LevelRes {
    pub fn new(base: &Path, level_num: u8, level_type_opt: Option<&str>) -> LevelRes {
        let level_dir = base.join("levels");
        let (level_path, level_type, obj_bank) = read_level(&level_dir, level_num);

        let paths = match level_type_opt {
            Some(v) => LevelPaths::from_default_dir(base, v),
            None => LevelPaths::from_default_dir(base, &level_type),
        };

        let mut file = File::options().read(true).open(&level_path).unwrap();
        let landscape = Landscape::from_reader(&mut file);
        file.seek(std::io::SeekFrom::Start(0x8000)).unwrap();
        //read 0x4000
        file.seek(std::io::SeekFrom::Current(0x4000)).unwrap();
        //read 0x4000
        file.seek(std::io::SeekFrom::Current(0x4000)).unwrap();
        //read 0x4000 (land flags)
        file.seek(std::io::SeekFrom::Current(0x4000)).unwrap();
        let mut tribes = Vec::new();
        for _ in 0..4 {
            tribes.push(TribeConfigRaw::from_reader(&mut file).unwrap());
        }
        let sunlight = Sunlight::from_reader(&mut file);
        // DAT unit section is fixed-size: 2000 slots * 55 bytes each.
        // Do not read UnitRaw entries until EOF, because trailing non-unit bytes
        // in the DAT file can be misinterpreted as extra bogus units.
        let units = read_fixed_unit_slots(&mut file, LEVEL_UNIT_SLOTS);
        let params = GlobeTextureParams::from_level(&paths);
        LevelRes {
            level_number: level_num,
            paths,
            params,
            landscape,
            tribes,
            sunlight,
            units,
            obj_bank,
        }
    }
}

/// Slot identifier in the level file. It is deliberately distinct from a
/// runtime object handle: loading a level may allocate the record into any
/// pool slot and with any generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LevelObjectIndex(pub u16);

#[derive(Debug, Copy, Clone)]
pub struct LevelObjectDefinition {
    pub source: LevelObjectIndex,
    pub model_type: ModelType,
    pub subtype: u8,
    pub tribe: u8,
    pub position: [i16; 2],
    pub angle: u16,
}

/// Renderer-independent input consumed by the simulation world constructor.
pub struct LevelDefinition {
    pub level_number: u8,
    pub heights: Box<[[u16; 128]; 128]>,
    pub sunlight: Sunlight,
    pub tribes: Vec<TribeConfigRaw>,
    pub objects: Vec<LevelObjectDefinition>,
}

impl From<LevelRes> for LevelDefinition {
    fn from(level: LevelRes) -> Self {
        Self::from_resource(&level)
    }
}

impl LevelDefinition {
    pub fn from_resource(level: &LevelRes) -> Self {
        let objects = level
            .units
            .iter()
            .enumerate()
            .filter_map(|(slot, raw)| {
                let model_type = raw.model_type()?;
                if raw.loc_x() == 0 && raw.loc_y() == 0 {
                    return None;
                }
                Some(LevelObjectDefinition {
                    source: LevelObjectIndex(slot as u16),
                    model_type,
                    subtype: raw.subtype,
                    tribe: raw.tribe_index(),
                    position: [raw.loc_x() as i16, raw.loc_y() as i16],
                    angle: (raw.angle() & 0x7ff) as u16,
                })
            })
            .collect();
        Self {
            level_number: level.level_number,
            heights: Box::new(level.landscape.height),
            sunlight: level.sunlight,
            tribes: level.tribes.clone(),
            objects,
        }
    }
}

fn read_fixed_unit_slots<R: Read>(reader: &mut R, count: usize) -> Vec<UnitRaw> {
    let mut units = Vec::with_capacity(count);
    for idx in 0..count {
        let unit = UnitRaw::from_reader(reader).unwrap_or_else(|| {
            panic!(
                "Level DAT is truncated while reading unit slot {}/{}",
                idx + 1,
                count
            )
        });
        units.push(unit);
    }
    units
}

pub fn read_level(base: &Path, num: u8) -> (PathBuf, String, u8) {
    let dat_path = LevelPaths::dat_path(base, num);
    let hdr_path = LevelPaths::hdr_path(base, num);
    let hdr_data = read_bin(&hdr_path);
    let landscape_type = read_landscape_type_from_bytes(&hdr_data);
    let obj_bank = if hdr_data.len() > 97 { hdr_data[97] } else { 0 };
    (dat_path, landscape_type, obj_bank)
}

/******************************************************************************/

pub fn read_landscape_type(hdr_path: &Path) -> String {
    let hdr_data = read_bin(hdr_path);
    read_landscape_type_from_bytes(&hdr_data)
}

fn read_landscape_type_from_bytes(hdr_data: &[u8]) -> String {
    if hdr_data.len() < 97 {
        panic!("Hdr is too small {}", hdr_data.len())
    }
    let type_int = hdr_data[96];
    match type_int {
        0..=9 => {
            let v = 0x30 + type_int;
            std::char::from_u32(v as u32)
                .unwrap()
                .to_string()
                .to_lowercase()
        }
        i if i < 36 => {
            let v = 0x41 + (type_int - 10);
            std::char::from_u32(v as u32)
                .unwrap()
                .to_string()
                .to_lowercase()
        }
        _ => panic!("Wrong landscape type {type_int:?}"),
    }
}

/******************************************************************************/

pub fn read_bin(path: &Path) -> Vec<u8> {
    let mut f = OpenOptions::new().read(true).open(path).unwrap();
    let mut vec = Vec::new();
    f.read_to_end(&mut vec).unwrap();
    vec
}

#[allow(dead_code)]
fn read_bin16(path: &Path) -> Vec<u16> {
    let buf = read_bin(path);
    let mut vec = vec![0; buf.len() / 2];
    for (i, n) in (0..).zip(buf.chunks(2).take(vec.len())) {
        if n.len() == 2 {
            vec[i] = u16::from_le_bytes([n[0], n[1]]);
        }
    }
    vec
}

fn read_bin_i8(path: &Path) -> Vec<i8> {
    let buf = read_bin(path);
    let mut v = std::mem::ManuallyDrop::new(buf);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

fn read_disp(path: &Path) -> Vec<i8> {
    let mut disp = read_bin_i8(path);
    let width = 256;
    for i in 0..width {
        for j in 0..(width / 2 - 1) {
            let n = i * width + j;
            let n1 = i * width + (width - 1) - j;
            disp.swap(n, n1);
        }
    }
    disp
}

#[cfg(test)]
mod tests {
    use super::read_fixed_unit_slots;
    use crate::data::types::BinDeserializer;
    use crate::data::units::UnitRaw;
    use std::io::Cursor;

    fn unit_raw_bytes(
        subtype: u8,
        model: u8,
        tribe_index: u8,
        loc_x: u16,
        loc_y: u16,
        angle: u32,
    ) -> [u8; 55] {
        let mut bytes = [0u8; 55];
        bytes[0] = subtype;
        bytes[1] = model;
        bytes[2] = tribe_index;
        bytes[3..5].copy_from_slice(&loc_x.to_le_bytes());
        bytes[5..7].copy_from_slice(&loc_y.to_le_bytes());
        bytes[7..11].copy_from_slice(&angle.to_le_bytes());
        bytes
    }

    #[test]
    fn read_fixed_unit_slots_reads_exact_slot_count() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&unit_raw_bytes(7, 1, 0, 0x1234, 0x5678, 0x9ABC_DEF0));
        bytes.extend_from_slice(&unit_raw_bytes(2, 1, 1, 0x0102, 0x0304, 0x0506_0708));
        // Extra valid UnitRaw + trailing noise should not be consumed.
        bytes.extend_from_slice(&unit_raw_bytes(5, 1, 2, 0x2222, 0x3333, 0x4444_5555));
        bytes.extend_from_slice(&[0xAA; 13]);

        let mut cursor = Cursor::new(bytes);
        let units = read_fixed_unit_slots(&mut cursor, 2);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].subtype, 7);
        assert_eq!(units[1].subtype, 2);

        // The next entry is still available in the stream.
        let next = UnitRaw::from_reader(&mut cursor).expect("expected unread third unit");
        assert_eq!(next.subtype, 5);
    }
}

pub fn read_pal(paths: &LevelPaths) -> Vec<u8> {
    read_bin(&paths.palette)
}

/// Build the 256-entry sky palette interpolation table per FUN_004dc3f0.
///
/// The game sorts 13 sky colors (pal[0x71..0x7E]) by perceived luminance,
/// then builds a table mapping each brightness level 0..255 to the palette
/// index of the closest sky color. This produces smooth gradients.
pub fn build_sky_interp_table(pal: &[u8]) -> [u8; 256] {
    let mut table = [0x70u8; 256];

    struct Entry {
        pal_idx: u8,
        lum: u32,
    }
    let mut entries: Vec<Entry> = Vec::with_capacity(13);
    for i in 1..=13u8 {
        let p = (0x70 + i) as usize * 4;
        let r = pal[p] as u32;
        let g = pal[p + 1] as u32;
        let b = pal[p + 2] as u32;
        entries.push(Entry {
            pal_idx: 0x70 + i,
            lum: r * 66 + g * 129 + b * 25,
        });
    }

    // Sort by luminance (ascending) — game uses selection sort, result is identical
    entries.sort_by_key(|e| e.lum);

    // Normalized luminance: (raw_lum >> 8), clamped to 255
    let norm_lums: Vec<u8> = entries
        .iter()
        .map(|e| (e.lum >> 8).min(255) as u8)
        .collect();

    let min_lum = *norm_lums.iter().min().unwrap() as i32;
    let max_lum = *norm_lums.iter().max().unwrap() as i32;
    let range = max_lum - min_lum;

    // Build 256-entry table: linearly sweep [min_lum, max_lum],
    // for each target find the sorted entry with closest luminance
    let mut acc: i32 = 0;
    for i in 0..256 {
        let target = min_lum + (acc >> 8);

        let mut best_idx = 0;
        let mut best_dist = i32::MAX;
        for (j, &lum) in norm_lums.iter().enumerate() {
            let dist = (lum as i32 - target).abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = j;
            }
        }

        table[i] = entries[best_idx].pal_idx;
        acc += range;
    }

    table
}

/******************************************************************************/

pub struct GlobeTextureParams {
    pub disp0: Vec<i8>,
    pub cliff0: Vec<u8>,
    pub bigf0: Vec<u8>,
    pub fade0: Vec<u8>,
    pub static_landscape_array: Vec<u16>,
    pub palette: Vec<u8>,
    pub watdisp: Vec<u8>,
}

impl GlobeTextureParams {
    pub fn from_level(paths: &LevelPaths) -> Self {
        Self {
            bigf0: read_bin(&paths.bigf0),
            cliff0: read_bin(&paths.cliff0),
            disp0: read_disp(&paths.disp0),
            fade0: read_bin(&paths.fade0),
            static_landscape_array: Self::make_static_array(),
            palette: read_bin(&paths.palette),
            watdisp: read_bin(&paths.watdisp),
        }
    }

    pub fn make_static_array() -> Vec<u16> {
        let mut v = vec![0; 1152];
        for (i, elem) in v.iter_mut().enumerate() {
            if i < 128 {
                *elem = 0x140;
            } else if i < 362 {
                *elem = (0xd3d - (1152 - i) * 3) as u16;
            } else {
                *elem = 0x400;
            }
        }
        v
    }
}

/******************************************************************************/

pub struct Landscape<const N: usize> {
    pub height: [[u16; N]; N],
}

impl<const N: usize> Landscape<N> {
    pub fn new() -> Self {
        Self {
            height: [[0u16; N]; N],
        }
    }

    pub fn land_size(&self) -> usize {
        N
    }

    fn flip(&mut self) {
        let width = N;
        for i in 0..width {
            for j in 0..(width / 2 - 1) {
                let n1 = (width - 1) - j;
                let v = self.height[j][i];
                self.height[j][i] = self.height[n1][i];
                self.height[n1][i] = v;
            }
        }
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Self {
        let mut s = Self::new();
        let mut buf = Vec::new();
        let _file_size = reader.read_to_end(&mut buf);
        for (i, n) in (0..).zip(buf.chunks(2).take(N * N)) {
            if n.len() == 2 {
                let val = u16::from_le_bytes([n[0], n[1]]);
                s.height[i % N][i / N] = val;
            }
        }
        s.flip();
        s
    }

    pub fn from_file(path: &Path) -> Self {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader(&mut file)
    }

    pub fn is_land_adj(&self, i: usize, j: usize) -> bool {
        if self.height[i][j] > 0 {
            return false;
        }
        let i_u = (i + 1) % N;
        let j_u = (j + 1) % N;
        let i_d = if i == 0 { N - 1 } else { i - 1 };
        let j_d = if j == 0 { N - 1 } else { j - 1 };
        (self.height[i][j_d] > 0)
            || (self.height[i][j_u] > 0)
            || (self.height[i_d][j] > 0)
            || (self.height[i_u][j] > 0)
            || (self.height[i_u][j_d] > 0)
            || (self.height[i_u][j_u] > 0)
            || (self.height[i_d][j_d] > 0)
            || (self.height[i_d][j_u] > 0)
    }

    pub fn make_shores(&self) -> Self {
        let mut output = Self {
            height: self.height,
        };
        for i in 0..N {
            for j in 0..N {
                if self.height[i][j] == 0 && self.is_land_adj(i, j) {
                    output.height[i][j] = 1;
                }
            }
        }
        output
    }

    pub fn to_vec(&self) -> Vec<u32> {
        let mut vec = vec![0u32; N * N];
        for i in 0..N {
            for j in 0..N {
                vec[i * N + j] = self.height[i][j] as u32;
            }
        }
        vec
    }
}

impl<const N: usize> Default for Landscape<N> {
    fn default() -> Self {
        Self::new()
    }
}

/******************************************************************************/
