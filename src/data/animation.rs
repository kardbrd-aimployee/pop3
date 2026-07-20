use core::mem::size_of;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::data::constants::*;
use crate::data::types::{from_reader, BinDeserializer, ImageInfo};

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VeleRaw {
    pub sprite_index: u16,
    pub coord_x: i16,
    pub coord_y: i16,
    pub flags: u16,
    pub next_index: u16,
}

impl BinDeserializer for VeleRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VeleRaw, { size_of::<VeleRaw>() }, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VfraRaw {
    pub index: u16,
    pub width: u8,
    pub height: u8,
    pub f3: u8,
    pub f4: u8,
    pub next_vfra: u16,
}

impl BinDeserializer for VfraRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VfraRaw, { size_of::<VfraRaw>() }, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VstartRaw {
    pub index: u16,
    pub f1: u8,
    pub f2: u8,
}

impl BinDeserializer for VstartRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<VstartRaw, { size_of::<VstartRaw>() }, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct AnimationsData {
    pub vele: Vec<VeleRaw>,
    pub vfra: Vec<VfraRaw>,
    pub vstart: Vec<VstartRaw>,
}

impl AnimationsData {
    pub fn from_reader<R: Read>(
        reader_vele: &mut R,
        reader_vfra: &mut R,
        reader_vstart: &mut R,
    ) -> Self {
        AnimationsData {
            vele: VeleRaw::from_reader_vec(reader_vele),
            vfra: VfraRaw::from_reader_vec(reader_vfra),
            vstart: VstartRaw::from_reader_vec(reader_vstart),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let mut file_vele = File::options()
            .read(true)
            .open(path.join("VELE-0.ANI"))
            .unwrap();
        let mut file_vfra = File::options()
            .read(true)
            .open(path.join("VFRA-0.ANI"))
            .unwrap();
        let mut file_vstart = File::options()
            .read(true)
            .open(path.join("VSTART-0.ANI"))
            .unwrap();
        Self::from_reader(&mut file_vele, &mut file_vfra, &mut file_vstart)
    }
}

use crate::data::psfb::ContainerPSFB;

/******************************************************************************/

pub const DIRS_PER_ANIM: usize = 8;
pub const STORED_DIRECTIONS: usize = 5;
pub const NUM_TRIBES: usize = 4;

/// Animation shape table — maps animation_id → (vstart_base, sprite_type).
/// Extracted from DAT_0059f638 in the original binary.
/// The sprite_type selects which body layers are composited (different per subtype).
/// vstart_base is the starting VSTART sequence index for direction 0.
pub const ANIM_SHAPE_TABLE: [(u16, u8); 161] = [
    /*   0 */ (8, 13),
    /*   1 */ (0, 13),
    /*   2 */ (16, 13),
    /*   3 */ (32, 13),
    /*   4 */ (24, 13),
    /*   5 */ (0, 13),
    /*   6 */ (0, 13),
    /*   7 */ (0, 13),
    /*   8 */ (0, 13),
    /*   9 */ (0, 13),
    /*  10 */ (0, 13),
    /*  11 */ (0, 13),
    /*  12 */ (0, 13),
    /*  13 */ (0, 13),
    /*  14 */ (0, 13),
    /*  15 brave idle  */ (48, 14),
    /*  16 warr idle  */ (48, 15),
    /*  17 prea idle   */ (48, 16),
    /*  18 spy idle   */ (48, 17),
    /*  19 fw idle     */ (48, 18),
    /*  20 sham idle  */ (424, 14),
    /*  21 brave walk  */ (40, 14),
    /*  22 warr walk  */ (40, 15),
    /*  23 prea walk   */ (40, 16),
    /*  24 spy walk   */ (40, 17),
    /*  25 fw walk     */ (40, 18),
    /*  26 sham walk  */ (616, 13),
    /*  27 brave die   */ (88, 14),
    /*  28 warr die   */ (88, 15),
    /*  29 prea die    */ (88, 16),
    /*  30 fw die     */ (88, 17),
    /*  31 spy die     */ (88, 18),
    /*  32 brave actn  */ (64, 14),
    /*  33 warr actn  */ (64, 15),
    /*  34 prea actn   */ (64, 16),
    /*  35 spy actn   */ (64, 17),
    /*  36 fw actn     */ (64, 18),
    /*  37 sham actn  */ (744, 14),
    /*  38 brave celeb */ (96, 14),
    /*  39 warr celeb */ (96, 15),
    /*  40 prea celeb  */ (96, 16),
    /*  41 spy celeb  */ (96, 17),
    /*  42 fw celeb    */ (96, 18),
    /*  43 brave spidl */ (80, 14),
    /*  44 warr spidl */ (80, 15),
    /*  45 prea spidl  */ (80, 16),
    /*  46 spy spidl  */ (80, 17),
    /*  47 fw spidl    */ (80, 18),
    /*  48 brave spwlk */ (72, 14),
    /*  49 warr spwlk */ (72, 15),
    /*  50 prea spwlk  */ (72, 16),
    /*  51 spy spwlk  */ (72, 17),
    /*  52 fw spwlk    */ (72, 18),
    /*  53 brave wrk1  */ (104, 14),
    /*  54 warr wrk1  */ (104, 15),
    /*  55 prea wrk1   */ (104, 16),
    /*  56 spy wrk1   */ (104, 17),
    /*  57 fw wrk1     */ (104, 18),
    /*  58 brave wrk2  */ (112, 14),
    /*  59 warr wrk2  */ (112, 15),
    /*  60 prea wrk2   */ (112, 16),
    /*  61 spy wrk2   */ (112, 17),
    /*  62 fw wrk2     */ (112, 18),
    /*  63 brave wrk3  */ (120, 14),
    /*  64 warr wrk3  */ (120, 15),
    /*  65 prea wrk3   */ (120, 16),
    /*  66 spy wrk3   */ (120, 17),
    /*  67 fw wrk3     */ (120, 18),
    /*  68 brave wrk4  */ (128, 14),
    /*  69 warr wrk4  */ (128, 15),
    /*  70 prea wrk4   */ (128, 16),
    /*  71 spy wrk4   */ (128, 17),
    /*  72 fw wrk4     */ (128, 18),
    /*  73 brave wrk5  */ (144, 14),
    /*  74 warr wrk5  */ (144, 15),
    /*  75 prea wrk5   */ (144, 16),
    /*  76 spy wrk5   */ (144, 17),
    /*  77 fw wrk5     */ (144, 18),
    /*  78 brave vhcl  */ (152, 14),
    /*  79 warr vhcl  */ (152, 15),
    /*  80 prea vhcl   */ (152, 16),
    /*  81 spy vhcl   */ (152, 17),
    /*  82 fw vhcl     */ (152, 18),
    /*  83 brave swim  */ (160, 14),
    /*  84 warr swim  */ (160, 15),
    /*  85 prea swim   */ (160, 16),
    /*  86 spy swim   */ (160, 17),
    /*  87 fw swim     */ (160, 18),
    /*  88 brave carry */ (168, 14),
    /*  89 warr carry */ (168, 15),
    /*  90 prea carry  */ (168, 16),
    /*  91 spy carry  */ (168, 17),
    /*  92 fw carry    */ (168, 18),
    /*  93 */ (0, 13),
    /*  94 sham spec   */ (176, 19),
    /*  95 */ (0, 13),
    /*  96 */ (0, 13),
    /*  97 */ (0, 13),
    /*  98 */ (0, 13),
    /*  99 */ (0, 13),
    /* 100 brave spec  */ (136, 19),
    /* 101 fw spec    */ (136, 22),
    /* 102 */ (0, 14),
    /* 103 */ (0, 14),
    /* 104 */ (0, 14),
    /* 105 */ (0, 14),
    /* 106 sham wrk1   */ (456, 14),
    /* 107 sham vhcl  */ (488, 14),
    /* 108 wild vhcl   */ (248, 13),
    /* 109 sham ??    */ (520, 14),
    /* 110 brave ride  */ (296, 14),
    /* 111 warr ride  */ (296, 15),
    /* 112 prea ride   */ (296, 16),
    /* 113 spy ride   */ (296, 17),
    /* 114 fw ride     */ (296, 18),
    /* 115 brave dig   */ (304, 14),
    /* 116 warr dig   */ (304, 15),
    /* 117 prea dig    */ (304, 16),
    /* 118 spy dig    */ (304, 17),
    /* 119 fw dig      */ (304, 18),
    /* 120 brave bld   */ (320, 14),
    /* 121 warr bld   */ (320, 15),
    /* 122 prea bld    */ (320, 16),
    /* 123 spy bld    */ (320, 17),
    /* 124 fw bld      */ (320, 18),
    /* 125 sham swim   */ (552, 14),
    /* 126 sham dig   */ (680, 14),
    /* 127 sham carry  */ (352, 14),
    /* 128 sham bld   */ (360, 14),
    /* 129 sham ride   */ (584, 14),
    /* 130 wild ride  */ (248, 13),
    /* 131 brave sit1  */ (384, 14),
    /* 132 warr sit1  */ (384, 15),
    /* 133 prea sit1   */ (384, 16),
    /* 134 spy sit1   */ (384, 17),
    /* 135 fw sit1     */ (384, 18),
    /* 136 brave sit2  */ (392, 14),
    /* 137 warr sit2  */ (392, 15),
    /* 138 prea sit2   */ (392, 16),
    /* 139 spy sit2   */ (392, 17),
    /* 140 fw sit2     */ (392, 18),
    /* 141 brave sit3  */ (400, 14),
    /* 142 warr sit3  */ (400, 15),
    /* 143 prea sit3   */ (400, 16),
    /* 144 spy sit3   */ (400, 17),
    /* 145 fw sit3     */ (400, 18),
    /* 146 brave sit4  */ (408, 14),
    /* 147 warr sit4  */ (408, 15),
    /* 148 prea sit4   */ (408, 16),
    /* 149 spy sit4   */ (408, 17),
    /* 150 fw sit4     */ (408, 18),
    /* 151-155 unused  */ (0, 14),
    (0, 14),
    (0, 14),
    (0, 14),
    (0, 14),
    /* 156 brave run   */ (416, 14),
    /* 157 warr run   */ (416, 15),
    /* 158 prea run    */ (416, 16),
    /* 159 spy run    */ (416, 17),
    /* 160 fw run      */ (416, 18),
];

/// Resolve animation_id → (vstart_base, sprite_type) using the shape table.
pub fn anim_shape(anim_id: u16) -> (usize, u8) {
    if (anim_id as usize) < ANIM_SHAPE_TABLE.len() {
        let (vstart, typ) = ANIM_SHAPE_TABLE[anim_id as usize];
        (vstart as usize, typ)
    } else {
        (0, 13)
    }
}

/// Idle animation indices from g_PersonAnimationTable (RE'd from original binary)
/// Format: (subtype, animation_index)
pub const UNIT_IDLE_ANIMS: [(u8, usize); 6] = [
    (PERSON_SUBTYPE_BRAVE, 15),
    (PERSON_SUBTYPE_WARRIOR, 16),
    (PERSON_SUBTYPE_PREACHER, 17),
    (PERSON_SUBTYPE_SPY, 18),
    (PERSON_SUBTYPE_FIREWARRIOR, 19),
    (PERSON_SUBTYPE_SHAMAN, 20),
];

/// Native animation IDs needed by the currently implemented person states.
///
/// The order is idle, walk, die, action, celebrate, chop, swim, carry, dig,
/// build, and run. Keeping every supported action in the runtime atlas avoids
/// silently sampling the first idle columns when a state changes animation.
/// Shamans are split between direct sprites and composited action sprites and
/// are declared separately below.
pub const UNIT_RUNTIME_ANIMS: [(u8, &[usize]); 5] = [
    (
        PERSON_SUBTYPE_BRAVE,
        &[15, 21, 27, 32, 38, 73, 83, 88, 115, 120, 156],
    ),
    (
        PERSON_SUBTYPE_WARRIOR,
        &[16, 22, 28, 33, 39, 74, 84, 89, 116, 121, 157],
    ),
    (
        PERSON_SUBTYPE_PREACHER,
        &[17, 23, 29, 34, 40, 75, 85, 90, 117, 122, 158],
    ),
    (
        PERSON_SUBTYPE_SPY,
        &[18, 24, 30, 35, 41, 76, 86, 91, 118, 123, 159],
    ),
    (
        PERSON_SUBTYPE_FIREWARRIOR,
        &[19, 25, 31, 36, 42, 77, 87, 92, 119, 124, 160],
    ),
];

/// Shaman animations stored as composited VELE chains. Idle/walk remain in
/// `SHAMAN_ANIMS` because those two are complete, direct per-tribe sprites.
pub const SHAMAN_RUNTIME_COMPOSITED_ANIMS: &[usize] = &[37, 125, 126, 127, 128];

/// Type-specific VELE layer selector for each person subtype.
pub fn unit_combo_for_subtype(subtype: u8) -> Option<(u16, u16)> {
    match subtype {
        PERSON_SUBTYPE_WARRIOR => Some((2, 2)),
        PERSON_SUBTYPE_PREACHER => Some((3, 1)),
        PERSON_SUBTYPE_SPY => Some((2, 3)),
        PERSON_SUBTYPE_FIREWARRIOR => Some((2, 1)),
        _ => None,
    }
}

/******************************************************************************/

pub enum ElementRotate {
    NoRotate,
    RotateHorizontal,
    RotateVertical,
}

#[derive(Debug, Copy, Clone)]
pub struct AnimationElement {
    pub sprite_index: usize,
    pub coord_x: i16,
    pub coord_y: i16,
    pub tribe: u8,
    pub flags: u16,
    pub uvar5: u16,
    pub original_flags: u16,
}

#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub index: usize,
    pub width: usize,
    pub height: usize,
    pub sprites: Vec<AnimationElement>,
}

pub struct AnimationSequence {
    pub index: usize,
    pub frames: Vec<AnimationFrame>,
}

impl AnimationElement {
    pub fn get_tribe(&self) -> u8 {
        self.tribe
    }

    pub fn is_hidden(&self) -> bool {
        false
    }

    pub fn is_common(&self) -> bool {
        !(self.is_tribe_specific() || self.is_type_specific())
    }

    pub fn is_tribe_specific(&self) -> bool {
        self.uvar5 == 1
    }

    pub fn is_type_specific(&self) -> bool {
        self.uvar5 > 1
    }

    pub fn get_rotate(&self) -> ElementRotate {
        if (self.flags & 0x1) != 0 {
            ElementRotate::RotateHorizontal
        } else if (self.flags & 0x2) != 0 {
            ElementRotate::RotateVertical
        } else {
            ElementRotate::NoRotate
        }
    }

    pub fn from_data(index: u16, vele: &[VeleRaw]) -> Vec<Self> {
        let mut sprites = Vec::new();
        let mut vele_index = index as usize;
        while vele_index != 0 {
            let vele_sprite = &vele[vele_index];
            sprites.push(AnimationElement {
                sprite_index: (vele_sprite.sprite_index as usize / 6).saturating_sub(1),
                coord_x: vele_sprite.coord_x,
                coord_y: vele_sprite.coord_y,
                tribe: (vele_sprite.flags >> 9) as u8,
                flags: vele_sprite.flags & 0x1f,
                uvar5: (vele_sprite.flags & 0x1f0) >> 4,
                original_flags: vele_sprite.flags,
            });
            vele_index = vele_sprite.next_index as usize;
            if sprites.len() > 255 {
                break;
            }
        }
        sprites
    }
}

impl AnimationFrame {
    pub fn get_permutations(
        &self,
        with_tribe: bool,
        with_type: bool,
    ) -> Vec<Vec<AnimationElement>> {
        let mut common_elems = Vec::new();
        let mut tribe_elems = Vec::new();
        let mut type_elems = Vec::new();
        for elem in &self.sprites {
            if elem.is_hidden() {
                continue;
            }
            if elem.is_common() {
                common_elems.push(*elem);
                if with_type {
                    type_elems.push(*elem);
                }
                if with_tribe {
                    tribe_elems.push(*elem);
                }
            } else if with_tribe && elem.is_tribe_specific() {
                tribe_elems.push(*elem);
            } else if with_type && elem.is_type_specific() {
                type_elems.push(*elem);
            }
        }
        let mut res = Vec::new();
        if tribe_elems.is_empty() && type_elems.is_empty() {
            res.push(common_elems);
        } else if !tribe_elems.is_empty() {
            for tribe_elem in tribe_elems {
                let mut res_tribe = common_elems.clone();
                res_tribe.push(tribe_elem);
                if type_elems.is_empty() {
                    res.push(res_tribe);
                } else {
                    for type_elem in &type_elems {
                        let mut res_type = res_tribe.clone();
                        res_type.push(*type_elem);
                        res.push(res_type);
                    }
                }
            }
        } else {
            for type_elem in &type_elems {
                let mut res_type = common_elems.clone();
                res_type.push(*type_elem);
                res.push(res_type);
            }
        }
        res
    }

    pub fn from_data(index: u16, vfra: &[VfraRaw], vele: &[VeleRaw]) -> Vec<Self> {
        let mut frames = Vec::new();
        let mut vfra_index = index as usize;
        while vfra_index != 0 {
            let vfra_frame = &vfra[vfra_index];
            frames.push(AnimationFrame {
                index: vfra_index,
                width: vfra_frame.width as usize,
                height: vfra_frame.height as usize,
                sprites: AnimationElement::from_data(vfra_frame.index, vele),
            });
            vfra_index = vfra_frame.next_vfra as usize;
            if frames.len() > 255 {
                break;
            }
            if vfra_index == (index as usize) {
                break;
            }
        }
        frames
    }
}

impl ImageInfo for AnimationFrame {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl AnimationSequence {
    pub fn from_data(anim_data: &AnimationsData) -> Vec<Self> {
        let mut res = Vec::<Self>::with_capacity(anim_data.vstart.len());
        for (index, vstart) in (0..).zip(&anim_data.vstart) {
            let frames = AnimationFrame::from_data(vstart.index, &anim_data.vfra, &anim_data.vele);
            res.push(AnimationSequence { index, frames });
        }
        res
    }

    pub fn get_frames(anim_seq_vec: &Vec<Self>) -> Vec<AnimationFrame> {
        let mut frames = Vec::new();
        for anim_seq in anim_seq_vec {
            frames.extend_from_slice(&anim_seq.frames);
        }
        frames
    }
}

/******************************************************************************/
// Sprite compositing helpers (used by main renderer and unit_viewer)
/******************************************************************************/

/// Check if an element should be rendered given the current tribe and unit combo.
/// Layer 0 (base): always render
/// Layer 1 (tribe): render if element's tribe matches selected tribe
/// Layer 2+ (type): render only if (uvar5, tribe) matches selected unit combo
pub fn should_render_element(
    elem: &AnimationElement,
    tribe: u8,
    unit_combo: Option<(u16, u16)>,
) -> bool {
    if elem.is_hidden() {
        return false;
    }
    if elem.is_common() {
        return true;
    }
    if elem.is_tribe_specific() {
        return elem.get_tribe() == tribe;
    }
    if elem.is_type_specific() {
        return match unit_combo {
            Some((layer, high)) => elem.uvar5 == layer && elem.tribe as u16 == high,
            None => false,
        };
    }
    true
}

/// Discover available unit combos (layer_type, element_tribe) from an animation's elements.
pub fn discover_unit_combos(sequences: &[AnimationSequence], base: usize) -> Vec<(u16, u16)> {
    let mut combos: Vec<(u16, u16)> = Vec::new();
    for dir in 0..STORED_DIRECTIONS {
        let seq_idx = base + dir;
        if seq_idx >= sequences.len() {
            continue;
        }
        for frame in &sequences[seq_idx].frames {
            for elem in &frame.sprites {
                if elem.is_type_specific() {
                    let combo = (elem.uvar5, elem.tribe as u16);
                    if !combos.contains(&combo) {
                        combos.push(combo);
                    }
                }
            }
        }
    }
    combos.sort();
    combos
}

/// Composite a single animation frame's elements into an RGBA bitmap.
pub fn composite_frame(
    elements: &[AnimationElement],
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    tribe: u8,
    unit_combo: Option<(u16, u16)>,
    frame_width: usize,
    frame_height: usize,
    offset_x: i32,
    offset_y: i32,
) -> Vec<u8> {
    let fw = frame_width;
    let fh = frame_height;
    let mut rgba = vec![0u8; fw * fh * 4];

    for elem in elements {
        if !should_render_element(elem, tribe, unit_combo) {
            continue;
        }

        let sprite_index = elem.sprite_index;
        let image = match container.get_image(sprite_index) {
            Some(img) => img,
            None => continue,
        };
        let info = match container.get_info(sprite_index) {
            Some(i) => i,
            None => continue,
        };

        let sw = info.width as usize;
        let sh = info.height as usize;

        let h_flip = matches!(elem.get_rotate(), ElementRotate::RotateHorizontal);
        let v_flip = matches!(elem.get_rotate(), ElementRotate::RotateVertical);

        let ox = (elem.coord_x as i32 - offset_x) as isize;
        let oy = (elem.coord_y as i32 - offset_y) as isize;

        for y in 0..sh {
            for x in 0..sw {
                let src_x = if h_flip { sw - 1 - x } else { x };
                let src_y = if v_flip { sh - 1 - y } else { y };
                let src = image.data[src_y * sw + src_x];
                if src == 255 {
                    continue;
                }

                let dst_x = ox + x as isize;
                let dst_y = oy + y as isize;
                if dst_x < 0 || dst_y < 0 || dst_x >= fw as isize || dst_y >= fh as isize {
                    continue;
                }

                let dst_off = (dst_y as usize * fw + dst_x as usize) * 4;
                let c = palette.get(src as usize).unwrap_or(&[255, 0, 255, 255]);
                rgba[dst_off] = c[0];
                rgba[dst_off + 1] = c[1];
                rgba[dst_off + 2] = c[2];
                rgba[dst_off + 3] = 255;
            }
        }
    }

    rgba
}

/// Compute a bounding box across ALL animations for consistent frame sizing.
/// Returns (min_x, min_y, max_x, max_y) encompassing all non-hidden elements.
pub fn compute_global_bbox(
    sequences: &[AnimationSequence],
    container: &ContainerPSFB,
) -> (i32, i32, i32, i32) {
    let mut min_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut max_x: i32 = 1;
    let mut max_y: i32 = 1;

    for seq in sequences {
        for frame in &seq.frames {
            for elem in &frame.sprites {
                if elem.is_hidden() {
                    continue;
                }
                if let Some(info) = container.get_info(elem.sprite_index) {
                    let ex = elem.coord_x as i32;
                    let ey = elem.coord_y as i32;
                    min_x = min_x.min(ex);
                    min_y = min_y.min(ey);
                    max_x = max_x.max(ex + info.width as i32);
                    max_y = max_y.max(ey + info.height as i32);
                }
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}

/// Build a sprite atlas for an animation with all 4 tribes.
/// Layout: rows = 4 tribes × 5 stored directions, cols = frames.
/// Returns (atlas_w, atlas_h, rgba, frame_w, frame_h, frames_per_dir) or None.
/// `unit_combo_override`: `None` = auto-detect first combo, `Some(x)` = use `x` as the combo.
/// `bbox_override`: when `Some`, use the provided (min_x, min_y, max_x, max_y) instead of
/// computing a per-animation bounding box. Pass a global bbox for consistent sizing across animations.
pub fn build_tribe_atlas(
    sequences: &[AnimationSequence],
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    vstart_base: usize,
    unit_combo_override: Option<Option<(u16, u16)>>,
    bbox_override: Option<(i32, i32, i32, i32)>,
) -> Option<(u32, u32, Vec<u8>, u32, u32, u32, i32)> {
    let base = vstart_base;

    // Count max frames per direction
    let mut max_frames = 0usize;
    for dir in 0..STORED_DIRECTIONS {
        let seq_idx = base + dir;
        if seq_idx >= sequences.len() {
            continue;
        }
        max_frames = max_frames.max(sequences[seq_idx].frames.len());
    }
    if max_frames == 0 {
        return None;
    }

    // Resolve unit combo BEFORE bounding box so we can filter elements
    let unit_combo = match unit_combo_override {
        Some(combo) => combo,
        None => {
            let combos = discover_unit_combos(sequences, base);
            combos.first().copied()
        }
    };

    let (min_x, min_y, max_x, max_y) = if let Some(bbox) = bbox_override {
        bbox
    } else {
        // Compute bounding box across rendered elements only.
        // Type-specific elements (uvar5 > 1) are excluded when unit_combo is None,
        // matching bevy_demo5 which filters elements before compositing.
        let mut min_x: i32 = 0;
        let mut min_y: i32 = 0;
        let mut max_x: i32 = 1;
        let mut max_y: i32 = 1;

        for dir in 0..STORED_DIRECTIONS {
            let seq_idx = base + dir;
            if seq_idx >= sequences.len() {
                continue;
            }
            for frame in &sequences[seq_idx].frames {
                for elem in &frame.sprites {
                    if elem.is_hidden() {
                        continue;
                    }
                    if elem.is_type_specific() {
                        match unit_combo {
                            Some((layer, high)) => {
                                if elem.uvar5 != layer || elem.tribe as u16 != high {
                                    continue;
                                }
                            }
                            None => continue,
                        }
                    }
                    if let Some(info) = container.get_info(elem.sprite_index) {
                        let ex = elem.coord_x as i32;
                        let ey = elem.coord_y as i32;
                        min_x = min_x.min(ex);
                        min_y = min_y.min(ey);
                        max_x = max_x.max(ex + info.width as i32);
                        max_y = max_y.max(ey + info.height as i32);
                    }
                }
            }
        }
        (min_x, min_y, max_x, max_y)
    };

    let fw = ((max_x - min_x) as u32).max(1).min(512);
    let fh = ((max_y - min_y) as u32).max(1).min(512);
    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as u32;
    let atlas_w = fw * max_frames as u32;
    let atlas_h = fh * total_rows;

    if atlas_w == 0 || atlas_h == 0 {
        return None;
    }

    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    for tribe in 0..NUM_TRIBES {
        for dir in 0..STORED_DIRECTIONS {
            let seq_idx = base + dir;
            if seq_idx >= sequences.len() {
                continue;
            }

            let dir_frames = &sequences[seq_idx].frames;
            if dir_frames.is_empty() {
                continue;
            }
            for f in 0..max_frames {
                let frame = &dir_frames[f % dir_frames.len()];

                let frame_rgba = composite_frame(
                    &frame.sprites,
                    container,
                    palette,
                    tribe as u8,
                    unit_combo,
                    fw as usize,
                    fh as usize,
                    min_x,
                    min_y,
                );

                let cell_x = f as u32 * fw;
                let row = (tribe * STORED_DIRECTIONS + dir) as u32;
                let cell_y = row * fh;
                for y in 0..fh {
                    let src_row = (y * fw * 4) as usize;
                    let dst_row = ((cell_y + y) * atlas_w + cell_x) as usize * 4;
                    let len = (fw * 4) as usize;
                    rgba[dst_row..dst_row + len]
                        .copy_from_slice(&frame_rgba[src_row..src_row + len]);
                }
            }
        }
    }

    Some((atlas_w, atlas_h, rgba, fw, fh, max_frames as u32, max_y))
}

/// Build a combined atlas for multiple animations of the same subtype.
/// `anim_ids` are animation IDs from the person animation table (not VSTART indices).
/// Resolves each through ANIM_SHAPE_TABLE to get the correct VSTART base.
/// Returns (atlas_w, atlas_h, rgba, frame_w, frame_h, total_columns, offsets)
/// where `offsets` maps: `offsets[i] = (anim_id, column_offset, frame_count)`.
pub fn build_multi_anim_atlas(
    sequences: &[AnimationSequence],
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    anim_ids: &[usize],
    unit_combo: Option<(u16, u16)>,
) -> Option<(
    u32,
    u32,
    Vec<u8>,
    u32,
    u32,
    u32,
    Vec<(usize, u32, u32)>,
    i32,
)> {
    if anim_ids.is_empty() {
        return None;
    }

    // Resolve animation IDs to VSTART bases
    let resolved: Vec<(usize, usize)> = anim_ids
        .iter()
        .map(|&id| {
            let (vb, _) = anim_shape(id as u16);
            (id, vb)
        })
        .collect();

    // Compute shared bounding box across all requested animations
    let mut bbox_min_x: i32 = 0;
    let mut bbox_min_y: i32 = 0;
    let mut bbox_max_x: i32 = 1;
    let mut bbox_max_y: i32 = 1;

    for &(_, vstart_base) in &resolved {
        for dir in 0..STORED_DIRECTIONS {
            let seq_idx = vstart_base + dir;
            if seq_idx >= sequences.len() {
                continue;
            }
            for frame in &sequences[seq_idx].frames {
                for elem in &frame.sprites {
                    if elem.is_hidden() {
                        continue;
                    }
                    if elem.is_type_specific()
                        && !unit_combo.is_some_and(|(layer, high)| {
                            elem.uvar5 == layer && elem.tribe as u16 == high
                        })
                    {
                        continue;
                    }
                    if let Some(info) = container.get_info(elem.sprite_index) {
                        let ex = elem.coord_x as i32;
                        let ey = elem.coord_y as i32;
                        bbox_min_x = bbox_min_x.min(ex);
                        bbox_min_y = bbox_min_y.min(ey);
                        bbox_max_x = bbox_max_x.max(ex + info.width as i32);
                        bbox_max_y = bbox_max_y.max(ey + info.height as i32);
                    }
                }
            }
        }
    }

    let shared_bbox = (bbox_min_x, bbox_min_y, bbox_max_x, bbox_max_y);

    // Build individual atlases with shared bbox
    let mut sub_atlases: Vec<(usize, u32, u32, Vec<u8>, u32)> = Vec::new(); // (anim_id, w, h, rgba, frames)
    let mut fw = 0u32;
    let mut fh = 0u32;

    for &(anim_id, vstart_base) in &resolved {
        if let Some((aw, ah, rgba, w, h, frames, _my)) = build_tribe_atlas(
            sequences,
            container,
            palette,
            vstart_base,
            Some(unit_combo),
            Some(shared_bbox),
        ) {
            fw = w;
            fh = h;
            sub_atlases.push((anim_id, aw, ah, rgba, frames));
        }
    }

    if sub_atlases.is_empty() {
        return None;
    }

    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as u32;
    let total_columns: u32 = sub_atlases.iter().map(|(_, _, _, _, f)| *f).sum();
    let atlas_w = fw * total_columns;
    let atlas_h = fh * total_rows;

    if atlas_w == 0 || atlas_h == 0 {
        return None;
    }

    let mut combined = vec![0u8; (atlas_w * atlas_h * 4) as usize];
    let mut col_offset = 0u32;
    let mut offsets = Vec::new();

    for (anim_idx, sub_w, _sub_h, sub_rgba, frames) in &sub_atlases {
        offsets.push((*anim_idx, col_offset, *frames));

        // Copy each row from sub-atlas into combined atlas at the right column offset
        for row in 0..atlas_h {
            let src_start = (row * sub_w * 4) as usize;
            let src_end = src_start + (*sub_w * 4) as usize;
            let dst_x = (col_offset * fw * 4) as usize;
            let dst_start = (row * atlas_w * 4) as usize + dst_x;
            if src_end <= sub_rgba.len() && dst_start + (sub_w * 4) as usize <= combined.len() {
                combined[dst_start..dst_start + (*sub_w * 4) as usize]
                    .copy_from_slice(&sub_rgba[src_start..src_end]);
            }
        }

        col_offset += frames;
    }

    Some((
        atlas_w,
        atlas_h,
        combined,
        fw,
        fh,
        total_columns,
        offsets,
        bbox_max_y,
    ))
}

/******************************************************************************/

/// Per-tribe shaman sprite start indices and frame counts.
/// Shamans use complete pre-rendered per-tribe sprites, not VELE compositing.
/// Derived from VSTART→VFRA→VELE chain: tribe offset = frames_per_dir × 5 stored directions.
pub const SHAMAN_ANIMS: [(u16, [u16; 4], usize); 2] = [
    // (anim_id, per-tribe starts, frames_per_dir)
    (20, [6879, 6899, 6919, 6939], 4), // idle: 4 unique frames × 5 dirs = 20 sprites/tribe
    (26, [7578, 7618, 7658, 7698], 8), // walk: 8 frames × 5 dirs = 40 sprites/tribe
];

/// Build a sprite atlas from direct PSFB sprite indices (non-composited).
/// Used for shamans which have complete per-tribe sprites, not VELE layers.
/// Layout matches `build_tribe_atlas`: rows = 4 tribes × 5 directions, cols = frames.
pub fn build_direct_sprite_atlas(
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    tribe_sprite_starts: &[u16; 4],
    frames_per_dir: usize,
) -> Option<(u32, u32, Vec<u8>, u32, u32, u32, i32)> {
    if frames_per_dir == 0 {
        return None;
    }

    // First pass: find max sprite dimensions across all tribes/directions/frames
    let mut max_w: u32 = 0;
    let mut max_h: u32 = 0;
    for tribe in 0..NUM_TRIBES {
        let start = tribe_sprite_starts[tribe] as usize;
        for dir in 0..STORED_DIRECTIONS {
            for f in 0..frames_per_dir {
                let idx = start + dir * frames_per_dir + f;
                if let Some(info) = container.get_info(idx) {
                    max_w = max_w.max(info.width as u32);
                    max_h = max_h.max(info.height as u32);
                }
            }
        }
    }
    if max_w == 0 || max_h == 0 {
        return None;
    }

    let fw = max_w;
    let fh = max_h;
    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as u32;
    let atlas_w = fw * frames_per_dir as u32;
    let atlas_h = fh * total_rows;
    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    for tribe in 0..NUM_TRIBES {
        let start = tribe_sprite_starts[tribe] as usize;
        for dir in 0..STORED_DIRECTIONS {
            for f in 0..frames_per_dir {
                let idx = start + dir * frames_per_dir + f;
                let image = match container.get_image(idx) {
                    Some(img) => img,
                    None => continue,
                };
                let info = match container.get_info(idx) {
                    Some(i) => i,
                    None => continue,
                };

                let sw = info.width as u32;
                let sh = info.height as u32;
                // Center sprite in cell
                let ox = (fw - sw) / 2;
                let oy = (fh - sh) / 2;
                let cell_x = f as u32 * fw;
                let row = (tribe * STORED_DIRECTIONS + dir) as u32;
                let cell_y = row * fh;

                for y in 0..sh {
                    for x in 0..sw {
                        let src = image.data[(y * sw + x) as usize];
                        if src == 255 {
                            continue;
                        } // transparent
                        let dst_x = cell_x + ox + x;
                        let dst_y = cell_y + oy + y;
                        let dst_off = ((dst_y * atlas_w + dst_x) * 4) as usize;
                        let c = palette.get(src as usize).unwrap_or(&[255, 0, 255, 255]);
                        rgba[dst_off] = c[0];
                        rgba[dst_off + 1] = c[1];
                        rgba[dst_off + 2] = c[2];
                        rgba[dst_off + 3] = 255;
                    }
                }
            }
        }
    }

    // For centered sprites, the below-foot padding is (fh - max_h) / 2
    let below_foot = ((fh - max_h) / 2) as i32;
    Some((
        atlas_w,
        atlas_h,
        rgba,
        fw,
        fh,
        frames_per_dir as u32,
        below_foot,
    ))
}

/// Build a combined atlas for multiple direct-sprite animations.
/// Each entry is (anim_id, per-tribe starts, frames_per_dir).
/// Returns the same format as `build_multi_anim_atlas`:
/// (atlas_w, atlas_h, rgba, frame_w, frame_h, total_columns, offsets)
/// where `offsets[i] = (anim_id, column_offset, frame_count)`.
pub fn build_direct_multi_anim_atlas(
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    anims: &[(u16, [u16; 4], usize)],
) -> Option<(
    u32,
    u32,
    Vec<u8>,
    u32,
    u32,
    u32,
    Vec<(usize, u32, u32)>,
    i32,
)> {
    if anims.is_empty() {
        return None;
    }

    // Compute shared max frame size across all animations
    let mut fw: u32 = 0;
    let mut fh: u32 = 0;
    for &(_, ref starts, fpd) in anims {
        for tribe in 0..NUM_TRIBES {
            let start = starts[tribe] as usize;
            for dir in 0..STORED_DIRECTIONS {
                for f in 0..fpd {
                    let idx = start + dir * fpd + f;
                    if let Some(info) = container.get_info(idx) {
                        fw = fw.max(info.width as u32);
                        fh = fh.max(info.height as u32);
                    }
                }
            }
        }
    }
    if fw == 0 || fh == 0 {
        return None;
    }

    let total_cols: u32 = anims.iter().map(|(_, _, fpd)| *fpd as u32).sum();
    let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as u32;
    let atlas_w = fw * total_cols;
    let atlas_h = fh * total_rows;
    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];
    let mut col_offset = 0u32;
    let mut offsets = Vec::new();

    for &(anim_id, ref starts, fpd) in anims {
        offsets.push((anim_id as usize, col_offset, fpd as u32));
        for tribe in 0..NUM_TRIBES {
            let start = starts[tribe] as usize;
            for dir in 0..STORED_DIRECTIONS {
                for f in 0..fpd {
                    let idx = start + dir * fpd + f;
                    let image = match container.get_image(idx) {
                        Some(img) => img,
                        None => continue,
                    };
                    let info = match container.get_info(idx) {
                        Some(i) => i,
                        None => continue,
                    };
                    let sw = info.width as u32;
                    let sh = info.height as u32;
                    let ox = (fw - sw) / 2;
                    let oy = (fh - sh) / 2;
                    let cell_x = (col_offset + f as u32) * fw;
                    let row = (tribe * STORED_DIRECTIONS + dir) as u32;
                    let cell_y = row * fh;
                    for y in 0..sh {
                        for x in 0..sw {
                            let src = image.data[(y * sw + x) as usize];
                            if src == 255 {
                                continue;
                            }
                            let dst_x = cell_x + ox + x;
                            let dst_y = cell_y + oy + y;
                            let dst_off = ((dst_y * atlas_w + dst_x) * 4) as usize;
                            let c = palette.get(src as usize).unwrap_or(&[255, 0, 255, 255]);
                            rgba[dst_off] = c[0];
                            rgba[dst_off + 1] = c[1];
                            rgba[dst_off + 2] = c[2];
                            rgba[dst_off + 3] = 255;
                        }
                    }
                }
            }
        }
        col_offset += fpd as u32;
    }

    // For centered sprites, below-foot padding = 0 for the tallest sprite
    Some((atlas_w, atlas_h, rgba, fw, fh, total_cols, offsets, 0))
}

/******************************************************************************/
