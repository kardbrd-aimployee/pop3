/// String table parser for the original game's LANGUAGE/lang00.dat binary format.
///
/// Format: u32le count, then count x u32le offsets (from file start),
/// then null-terminated ASCII strings at the given offsets.
/// The original English string table contains 0x526 (1318) strings.

pub struct StringTable {
    strings: Vec<String>,
}

impl StringTable {
    /// Parse string table from raw binary data (LANGUAGE/lang00.dat format).
    /// Format: u32le count, then count x u32le offsets, then null-terminated ASCII strings.
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() < 4 {
            return Self {
                strings: Vec::new(),
            };
        }

        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let offsets_end = 4 + count * 4;
        if data.len() < offsets_end {
            return Self {
                strings: Vec::new(),
            };
        }

        let mut strings = Vec::with_capacity(count);
        for i in 0..count {
            let off_pos = 4 + i * 4;
            let offset = u32::from_le_bytes([
                data[off_pos],
                data[off_pos + 1],
                data[off_pos + 2],
                data[off_pos + 3],
            ]) as usize;

            if offset >= data.len() {
                strings.push(String::new());
                continue;
            }

            // Read null-terminated string from offset
            let str_data = &data[offset..];
            let end = str_data
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(str_data.len());
            let s = String::from_utf8_lossy(&str_data[..end]).into_owned();
            strings.push(s);
        }

        Self { strings }
    }

    /// Get string by index. Returns None if out of bounds.
    pub fn get(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(|s| s.as_str())
    }

    /// Number of strings loaded.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns true if the string table is empty.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_u32_le(val: u32) -> [u8; 4] {
        val.to_le_bytes()
    }

    #[test]
    fn empty_data_returns_empty_table() {
        let table = StringTable::from_bytes(&[]);
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn single_string_parses_correctly() {
        // count=1, offset=8 (past header: 4 bytes count + 4 bytes offset), "Hello\0"
        let mut data = Vec::new();
        data.extend_from_slice(&make_u32_le(1)); // count
        data.extend_from_slice(&make_u32_le(8)); // offset to string
        data.extend_from_slice(b"Hello\0");

        let table = StringTable::from_bytes(&data);
        assert_eq!(table.len(), 1);
        assert_eq!(table.get(0), Some("Hello"));
    }

    #[test]
    fn multiple_strings_parse_with_correct_indexing() {
        // count=2, offsets=[12, 18], strings "Hello\0World!\0"
        let mut data = Vec::new();
        data.extend_from_slice(&make_u32_le(2)); // count
        data.extend_from_slice(&make_u32_le(12)); // offset to "Hello"
        data.extend_from_slice(&make_u32_le(18)); // offset to "World!"
        data.extend_from_slice(b"Hello\0World!\0");

        let table = StringTable::from_bytes(&data);
        assert_eq!(table.len(), 2);
        assert_eq!(table.get(0), Some("Hello"));
        assert_eq!(table.get(1), Some("World!"));
    }

    #[test]
    fn out_of_bounds_index_returns_none() {
        let table = StringTable::from_bytes(&[]);
        assert_eq!(table.get(0), None);
        assert_eq!(table.get(100), None);
    }

    #[test]
    fn data_too_short_for_header_returns_empty() {
        // Only 2 bytes -- not enough for u32 count
        let table = StringTable::from_bytes(&[0x01, 0x00]);
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn realistic_three_strings() {
        // count=3, 3 offsets, then 3 null-terminated strings
        let header_size: u32 = 4 + 3 * 4; // 16
        let s0 = b"Brave\0";
        let s1 = b"Warrior\0";
        let s2 = b"Shaman\0";

        let off0 = header_size;
        let off1 = off0 + s0.len() as u32;
        let off2 = off1 + s1.len() as u32;

        let mut data = Vec::new();
        data.extend_from_slice(&make_u32_le(3));
        data.extend_from_slice(&make_u32_le(off0));
        data.extend_from_slice(&make_u32_le(off1));
        data.extend_from_slice(&make_u32_le(off2));
        data.extend_from_slice(s0);
        data.extend_from_slice(s1);
        data.extend_from_slice(s2);

        let table = StringTable::from_bytes(&data);
        assert_eq!(table.len(), 3);
        assert_eq!(table.get(0), Some("Brave"));
        assert_eq!(table.get(1), Some("Warrior"));
        assert_eq!(table.get(2), Some("Shaman"));
    }
}
