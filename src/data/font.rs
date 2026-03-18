/// Font data parser for the original 8x8 bitmap font, with integer scaling
/// to produce 16x16 (scale=2) and 24x24 (scale=3) variants.

/// A single glyph bitmap: 1 byte per pixel (0=transparent, 255=opaque).
pub struct FontGlyph {
    pub width: u32,
    pub height: u32,
    pub bitmap: Vec<u8>,
}

/// Collection of 96 ASCII glyphs (32..127) at a given size.
pub struct FontData {
    pub glyph_width: u32,
    pub glyph_height: u32,
    glyphs: Vec<FontGlyph>,
}

impl FontData {
    /// Create FontData from the FONT_8X8 constant (96 glyphs, 8 bytes each,
    /// MSB = leftmost pixel).
    pub fn from_8x8_bitmap(data: &[[u8; 8]; 96]) -> Self {
        let mut glyphs = Vec::with_capacity(96);
        for glyph_rows in data.iter() {
            let mut bitmap = vec![0u8; 64];
            for (y, &row_bits) in glyph_rows.iter().enumerate() {
                for x in 0..8u32 {
                    if row_bits & (0x80 >> x) != 0 {
                        bitmap[(y * 8 + x as usize)] = 255;
                    }
                }
            }
            glyphs.push(FontGlyph {
                width: 8,
                height: 8,
                bitmap,
            });
        }
        Self {
            glyph_width: 8,
            glyph_height: 8,
            glyphs,
        }
    }

    /// Get glyph for an ASCII character. Returns blank glyph for out-of-range chars.
    /// Printable ASCII 32..127 maps to glyph indices 0..95.
    /// Control chars (0..31) and chars >= 128 get a blank fallback.
    pub fn glyph(&self, ch: u8) -> &FontGlyph {
        if ch >= 32 && ch <= 127 {
            let idx = (ch - 32) as usize;
            if idx < self.glyphs.len() {
                return &self.glyphs[idx];
            }
        }
        // Glyph 0 is space (all blank) -- use as fallback
        &self.glyphs[0]
    }

    /// Create a scaled version where each pixel becomes a scale x scale block.
    /// scale=1 -> 8x8, scale=2 -> 16x16, scale=3 -> 24x24.
    pub fn scaled(&self, scale: u32) -> Self {
        let new_w = self.glyph_width * scale;
        let new_h = self.glyph_height * scale;
        let mut new_glyphs = Vec::with_capacity(self.glyphs.len());

        for g in &self.glyphs {
            let mut bitmap = vec![0u8; (new_w * new_h) as usize];
            for y in 0..g.height {
                for x in 0..g.width {
                    let val = g.bitmap[(y * g.width + x) as usize];
                    if val > 0 {
                        // Fill scale x scale block
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let nx = x * scale + sx;
                                let ny = y * scale + sy;
                                bitmap[(ny * new_w + nx) as usize] = val;
                            }
                        }
                    }
                }
            }
            new_glyphs.push(FontGlyph {
                width: new_w,
                height: new_h,
                bitmap,
            });
        }

        Self {
            glyph_width: new_w,
            glyph_height: new_h,
            glyphs: new_glyphs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::hud::FONT_8X8;

    #[test]
    fn from_8x8_bitmap_creates_96_glyphs() {
        let font = FontData::from_8x8_bitmap(&FONT_8X8);
        // 96 glyphs for ASCII 32..127
        assert_eq!(font.glyphs.len(), 96);
        assert_eq!(font.glyph_width, 8);
        assert_eq!(font.glyph_height, 8);
    }

    #[test]
    fn glyph_a_is_non_empty() {
        let font = FontData::from_8x8_bitmap(&FONT_8X8);
        let g = font.glyph(b'A');
        assert_eq!(g.width, 8);
        assert_eq!(g.height, 8);
        // 'A' glyph must have some opaque pixels
        assert!(g.bitmap.iter().any(|&p| p == 255));
    }

    #[test]
    fn control_chars_return_blank() {
        let font = FontData::from_8x8_bitmap(&FONT_8X8);
        for ch in 0..32u8 {
            let g = font.glyph(ch);
            assert!(g.bitmap.iter().all(|&p| p == 0), "control char {} not blank", ch);
        }
    }

    #[test]
    fn char_above_127_returns_fallback_blank() {
        let font = FontData::from_8x8_bitmap(&FONT_8X8);
        let g = font.glyph(200);
        assert!(g.bitmap.iter().all(|&p| p == 0));
    }

    #[test]
    fn scaled_2_returns_16x16() {
        let font = FontData::from_8x8_bitmap(&FONT_8X8);
        let scaled = font.scaled(2);
        assert_eq!(scaled.glyph_width, 16);
        assert_eq!(scaled.glyph_height, 16);
        let g = scaled.glyph(b'A');
        assert_eq!(g.width, 16);
        assert_eq!(g.height, 16);
        assert_eq!(g.bitmap.len(), 16 * 16);
        // Must still have opaque pixels
        assert!(g.bitmap.iter().any(|&p| p == 255));
    }

    #[test]
    fn scaled_3_returns_24x24() {
        let font = FontData::from_8x8_bitmap(&FONT_8X8);
        let scaled = font.scaled(3);
        assert_eq!(scaled.glyph_width, 24);
        assert_eq!(scaled.glyph_height, 24);
        let g = scaled.glyph(b'A');
        assert_eq!(g.width, 24);
        assert_eq!(g.height, 24);
        assert_eq!(g.bitmap.len(), 24 * 24);
        assert!(g.bitmap.iter().any(|&p| p == 255));
    }
}
