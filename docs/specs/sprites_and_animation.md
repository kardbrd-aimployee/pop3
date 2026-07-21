# Sprites and Animation

## Animation System

### Animation Tables

| Address    | Name                  | Description                    |
|------------|-----------------------|--------------------------------|
| 0x0059fb30 | g_PersonAnimationTable| Animation lookup per state     |
| 0x0059f638 | g_AnimationFrameData  | Frame timing/count data        |
| 0x0059f800 | g_SpriteOffsets       | Sprite sheet offsets           |

### Person_SelectAnimation (0x004fed30)

Selects a logical animation-table row from the person state, then resolves it
for the unit subtype. The selector defaults to the walk row for navigation
states. At `0x004fed91` it compares the speed word at person `+0x5f` with zero;
a walk-class state with zero speed uses the idle row even if movement target
flags remain set.

The table contains 26 rows and 9 subtype columns (column 0 is the common/wild
fallback). The verified rows are retained in
`src/engine/units/animation.rs::PERSON_ANIMATION_TABLE`; they include idle,
walk, ride, action, die, celebrate, five work variants, vehicle, swim, carry,
dig, build, four seated variants, and run.

The state switch itself only chooses an initial default. Its notable direct
results are idle rows for states `0x01`, `0x03`, `0x0c`, `0x13`, `0x19`,
`0x1d`, `0x27`, and `0x2c`; action row 3 for `0x0b` and `0x0e`; direct logical
IDs 2 and 3 for `0x0f` and `0x10`; vehicle row 12 for `0x18`; and run row 25
for `0x1a`, `0x1f`, and `0x28`. All other in-range states initially use walk
row 1. Later behavior handlers explicitly replace that default for work,
death, combat, swimming, and other actions. This distinction is important:
the name of a person state alone is not an authoritative work-animation ID.

**Lookup Formula:**
```c
anim_index = g_PersonAnimationTable[animation_row * 9 + subtype]
```

### Person_SetAnimation (0x004feed0)

The logical animation ID is not stored directly in the native person object.
It indexes a four-byte shape record at `0x0059f638`:

```c
struct PersonAnimationShape {
    uint16_t vstart_index;
    uint16_t render_type;
};
```

`Person_SetAnimation` passes those values and `person + 0x33` to
`Animation_SetupFromBank @ 0x004b0ad0`. The resolved native track is:

| Person offset | Size | Meaning |
|---------------|------|---------|
| `+0x33` | 2 | Resolved VSTART index |
| `+0x35` | 2 | Native track control flags |
| `+0x37` | 2 | Track timing accumulator |
| `+0x39` | 1 | Current frame |
| `+0x3a` | 1 | Render/animation bank type |

`Animation_UpdateObjectTrack @ 0x004b0b80` advances this resolved track. Bit
`0x02` at `+0x35` causes its normal update path to return early, so it must not
be documented as a generic "playing" bit. Other control bits are contextual
and should only be named once their call sites are verified.

### Directional Sprites

Most units have 8 directional sprites:
- 0x000: East
- 0x100: NE
- 0x200: North
- 0x300: NW
- 0x400: West
- 0x500: SW
- 0x600: South
- 0x700: SE

Angles 0x400-0x7FF use mirrored sprites of 0x000-0x3FF.

### Sprite System

| Address    | Name              | Description                    |
|------------|-------------------|--------------------------------|
| 0x00476fa0 | Sprite_Decompress | RLE sprite decompression       |
| 0x00477200 | Sprite_Draw       | Draw sprite to surface         |
| 0x00477400 | Sprite_DrawTinted | Draw with color tint           |

### Sprite File Format (.spr)

- Header: sprite count, offsets table
- Per-sprite: width, height, RLE data
- 8-bit indexed color
- Palette from level or global pal*.dat

---

## Appendix AO: Sprite Format and RLE Decompression

### Sprite Data Buffers

**Primary Buffers (DAT_005a7d*):**
| Buffer | Purpose | Size Pointer |
|--------|---------|--------------|
| DAT_005a7d80 | Pixel data (RLE compressed) | DAT_005a7d94 |
| DAT_005a7d84 | Frame records (6 bytes each) | DAT_005a7d98 |
| DAT_005a7d88 | Animation sequences | DAT_005a7d9c |
| DAT_005a7d90 | Frame chain links (2 bytes each) | DAT_005a7da4 |

### Sprite File Format

**Files loaded by Animation_LoadAllData (0x00452530):**
1. **VSTART-0.ANI** - Initial frame records (8-byte entries)
2. **VFRA-0.ANI** - Frame sequence chains (2-byte links)
3. **VSPR-0.INF** - Sprite bank info
4. **VELE-0.ANI** - Animation velocity data (10-byte entries)

### Frame Record Structure (6 bytes)

```c
struct FrameRecord {
    uint16_t frame_id;      // +0x00: Frame sprite ID
    uint8_t  anim_type;     // +0x02: Animation type flag
    uint8_t  height;        // +0x03: Height/dimension
    uint16_t width_offset;  // +0x04: Width or offset
};
```

### RLE Compression Format

**Decompression happens on-the-fly during rendering (NOT pre-decompressed):**
- **Transparent pixels:** Run of 0xFF bytes (palette index 255)
- **Opaque runs:** Count byte followed by pixel values
- **Literal pixels:** Direct pixel values

**Rendering via Sprite_RenderObject (0x00411c90):**
1. Gets object from g_ObjectPtrArray
2. Reads frame record from DAT_005a7d84
3. Retrieves RLE data from DAT_005a7d80
4. Decompresses while blitting to framebuffer
5. Applies palette lookups from 0x5a0028

### Palette System

**Color Tables:**
- Base palette: 0x5a0028 (4 bytes per entry, BGRA)
- Secondary: 0x5a0039 (color offsets)

**Palette Selection:**
```c
palette_index = object[0x2b];
palette_offset = (palette_index + palette_index * 8) * 2;
color = palette_table[palette_offset * 4 + 0x5a0028];
```

---

## Appendix AX: Sprite Loading System

### Sprite Bank Loading Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Sprite_LoadBank | 0x00450990 | Main sprite bank loader |
| Sprite_LoadResources | 0x0041db20 | Resource initialization |
| Sprite_InitAnimationTables | 0x00451b50 | Animation table setup |
| Sprite_SetResolutionParams | 0x00451ff0 | Resolution-based config |

### Sprite File Paths

**Main Sprite Banks:**
- `data/hspr0-0.dat` - Primary sprite bank (0x0059d9f0)
- Two resolution variants (indexed 0 or 1)

**UI/Feature Sprites (data/fenew/):**
- Enemy sprites: `ettru.spr`, `ettee.spr`, `ettwe.spr`
- Buildings: `fesd33/20/15ru.spr`, `feti33/20ru.spr`, `felo33/20ru.spr`
- UI elements: `feslider.spr`, `feboxes.spr`, `fecursor.spr`, `igmslidr.spr`
- Language-specific: `felgsdja.spr` (Japanese), `felgsdch.spr` (Chinese)

**Animation Data Files:**
- `DATA/VSTART-0.ANI` (0x0057c708) - Animation start frames
- `DATA/VFRA-0.ANI` (0x0057c6f8) - Frame sequence data
- `DATA/VSPR-0.INF` (0x0059e9b0) - Sprite information
- `DATA/VELE-0.ANI` (0x0059e890) - Vehicle animations

### Sprite Bank File Format

```
+0x00: Header - File identification
+0x???: Frame Table - Index of all frames
        - Offset within file
        - Width and height
        - Hotspot/pivot coordinates
        - Compression flags
+0x???: Pixel Data - RLE-compressed 8-bit indexed pixels
```

### Frame Data Structure (6 bytes)

```c
struct FrameData {
    uint16_t frame_offset;    // +0x00: Offset into frame table
    uint8_t  frame_width;     // +0x02: Width in pixels
    uint8_t  frame_height;    // +0x03: Height in pixels
    uint16_t flags;           // +0x04: Animation/rendering flags
};
```

### Global Sprite Data

| Address | Size | Purpose |
|---------|------|---------|
| DAT_005a7d50 | 0xc8000 | Main sprite frame buffer (800KB) |
| DAT_005a7d80 | Dynamic | Animation start indices |
| DAT_005a7d84 | Dynamic | Frame lookup table |
| DAT_005a7d88 | Dynamic | Vehicle animation data |
| DAT_005a7d8c | Dynamic | Sprite info data |
| DAT_005a7d90 | Dynamic | Frame offset indices |
| DAT_0057c588 | 4 bytes | Current bank ID (-1=unloaded) |

### Sprite Loading Flow

```
Game Startup
  → Sprite_LoadBank(bank_id, width, height)
    → FUN_0041e790() - Load sprite files from disk
    → Buffer_ClearRegion() - Clear sprite buffer
    → Sprite_SetResolutionParams() - Configure for display
    → Sprite_InitAnimationTables() - Build animation tables
    → Animation_LoadAllData() - Load VSTART/VFRA/VSPR files
    → Sprite_LoadResources(1) - Initialize GPU resources
```

### Resolution Parameters

Two resolution modes:
- **Low resolution** (<0x4b000 pixels): Aggressive caching
- **High resolution**: Better quality, larger buffers

---

## Appendix AZ: Animation System

### Animation Loading

**Function:** `Animation_LoadAllData()` @ 0x00452530

Loads three main tables:
- `DAT_005a7d80` - Frame records (6 bytes each)
- `DAT_005a7d84` - Animation lookup table
- `DAT_005a7d90` - Velocity/sequence data

### Animation State Machine Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Person_SetAnimation | 0x004feed0 | Set unit animation |
| Person_SetAnimationByState | 0x004fee80 | State→animation mapping |
| Person_SelectAnimation | 0x004fed30 | Select animation by state |
| Animation_SetupFromBank | 0x004b0ad0 | Setup animation context |

### Animation Context Structure (11 bytes at object +0x33)

```c
struct AnimationContext {
    uint16_t animation_id;    // +0x00: Animation ID
    uint8_t  frame_index;     // +0x02: Current frame
    uint16_t timing_state;    // +0x03: Timing accumulator
    uint8_t  current_frame;   // +0x05: Display frame
    uint8_t  bank_select;     // +0x07: Bank selection
    uint8_t  start_offset;    // +0x09: Starting offset
};
```

### Object Animation Fields

- Offset +0x2b: Unit type
- Offset +0x2c: Current state
- Offset +0x33-0x39: Animation context (7 bytes)
- Offset +0x35: Animation flags (2 bytes)
- Offset +0x37: Animation timing
- Offset +0x39: Frame counter

### Animation Flags (Offset +0x35)

| Bit | Meaning |
|-----|---------|
| 0x01 | Loop flag |
| 0x02 | Play animation |
| 0x10 | Special variant |
| 0x80 | Reverse direction |

### State-to-Animation Mapping

Located at `DAT_0059fb30` (9 bytes per unit type)
- Index: `(unit_type * 9 + state) * 2`
- Returns animation ID for state/unit combination

**Common State Mappings:**
| States | Animation |
|--------|-----------|
| 0x01, 0x03, 0x0c, 0x13, 0x19, 0x1d, 0x27, 0x2c | 0 (Idle) |
| 0x0b, 0x0e | 3 (Action) |
| 0x0f | 2 (Attack) |
| 0x10 | 3 (Work) |
| 0x18 | 0x0c (Special) |
| 0x1a, 0x1f, 0x28 | 0x19 (Movement) |

### Velocity/Sequence Table (10 bytes per entry)

```c
struct VelocityEntry {
    uint16_t frame_duration;  // +0x00: Duration in ticks
    uint16_t frame_offset;    // +0x02: Frame data offset
    uint16_t sprite_x;        // +0x04: X offset
    uint16_t sprite_y;        // +0x06: Y offset
    uint16_t next_index;      // +0x08: Next entry (for chaining)
};
```

### Frame Advancement (Update Tick)

**Main Loop:** `Tick_UpdateObjects()` @ 0x004a7550
- Called once per game tick
- Advances animation frame counters
- Calls `Object_UpdateState()` for each object type

**Rendering:** `FUN_004e7190()`
```c
// Frame lookup
puVar7 = DAT_005a7d88 + (uint)*(ushort*)(DAT_005a7d84 + animation_id * 6) * 5;
// Iterate velocity table
do {
    if (DAT_005a7d54 < (*puVar7 + DAT_005a7d54)) {
        FUN_0050f6e0(x_pos, y_pos, sprite_data);
    }
    puVar7 = DAT_005a7d88 + (uint)puVar7[4] * 5;  // Next entry
} while (DAT_005a7d88 < puVar7);
```

### Animation Event Callbacks

**Effect Queue:** `Effect_QueueVisual()` @ 0x00453780

Queued effect structure (61 bytes):
- +0x00: Effect flag
- +0x01: Effect subtype
- +0x02-0x03: Timing parameters
- +0x04-0x0b: Position (x, z, height, object_id)
- +0x0c-0x3c: Extra parameters

**Sound Triggers (in Effect_Init):**
| Animation ID | Sound ID | Event |
|--------------|----------|-------|
| 0x08 | 0xaf | Frame trigger |
| 0x1c | 0xb2 | Specific frame |
| 0x1d | 0xa2 | Frame start |

### Animation Lookup Hierarchy

```
Object.AnimationID (short @ +0x33)
    ↓
DAT_005a7d84 lookup (6-byte frame record offset)
    ↓
Frame Record (frame_id, anim_type, height, width_offset)
    ↓
DAT_005a7d88 (velocity table) - Sequence chaining
    ↓
Sprite Data (via frame_id * 8 + DAT_005a7d50)
```

### Global Animation Data

| Address | Purpose |
|---------|---------|
| 0x005a7d80 | Frame records (6 bytes each) |
| 0x005a7d84 | Animation index lookup |
| 0x005a7d88 | Velocity/timing chain |
| 0x005a7d90 | Alt velocity table |
| 0x005a7d54 | Global frame timing accumulator |
| 0x0059fb30 | State→Animation mapping |
| 0x0059f8d8 | Unit-type animation bank pointers |
| 0x005a7d50 | Sprite base address |

---

## Appendix CC: Sprite/Object Rendering

### Object Type to Render Path

From `FUN_0046af00`:

| Object Type | Render Path |
|-------------|-------------|
| 0x01 (Person) | Full animation with shadow, effects |
| 0x02 (Building) | Static sprite with state-based frames |
| 0x05 (Spell) | Effect sprites |
| 0x0A (Triggered) | Animation sequence |

### Animation Frame Selection

For animated objects (type 0x0D):

```c
// From FUN_0046af00 sprite rendering
int sprite_index = object->sprite_base;
if (char_at(animation_table + object->anim_type * 0xB + 1) >= 2) {
    sprite_index += object->anim_frame >> 2;  // Quarter-speed
}

// Direction adjustment (8 directions)
int direction = ((g_CameraTarget->rotation - object->facing) - 0x380) & 0x700) >> 8;
sprite_index += direction;
```

### Sprite Scaling by Depth

For depth-scaled sprites (type 0x12):

```c
int scale_x = (original_width * scale_param) >> 8;
int scale_y = (original_height * scale_param) >> 8;
```

### Shadow Rendering

Objects with shadow flag render twice:
1. First pass: Shadow (offset, darkened)
2. Second pass: Main sprite

From `Sprite_RenderWithShadow @ 0x00411b70`:
```c
if (has_shadow) {
    // Setup shadow color mask
    Render_SetupColorMasks(shadow_colors);
    // Offset position for shadow
    FUN_00416000(object);
    Sprite_RenderObject(object, ...);
    // Restore normal colors
    Render_SetupColorMasks(normal_colors);
}
```

---

## Appendix CG: Animation System

### Animation Tables

| Address | Table | Purpose |
|---------|-------|---------|
| 0x005a7d50 | g_SpriteTable | Sprite frame data (8 bytes per entry) |
| 0x005a7d54 | g_SpriteData | Raw sprite pixel data |
| 0x005a7d80 | g_AnimDirTable | Direction-based animation lookup |
| 0x005a7d84 | g_AnimFrameTable | Animation frame indices (6 bytes) |
| 0x005a7d88 | g_AnimPartTable | Animation part data (10 bytes) |

### Sprite Frame Structure (8 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | data_offset | Offset into sprite pixel data |
| +0x04 | 2 | width | Sprite width in pixels |
| +0x06 | 2 | height | Sprite height in pixels |

### Animation Frame Table Entry (6 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 2 | first_part | Index of first part in AnimPartTable |
| +0x02 | 4 | reserved | Additional data |

### Animation Part Table Entry (10 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 2 | sprite_index | Index into SpriteTable |
| +0x02 | 2 | offset_x | X offset from object position |
| +0x04 | 2 | offset_y | Y offset from object position |
| +0x06 | 2 | flags | Rendering flags (flip, blend, etc.) |
| +0x08 | 2 | next_part | Index of next part (linked list) |

### Animation Flags

| Bit | Value | Meaning |
|-----|-------|---------|
| 0 | 0x01 | Flip horizontal |
| 1 | 0x02 | No shadow |
| 2 | 0x04 | Additional flip |
| 4-7 | 0xF0 | Blend mode |
| 8-9 | 0x300 | Layer order |

### Animation_RenderFrameSequence @ 0x004e7190

Renders a complete animation frame with all parts:

```c
void Animation_RenderFrameSequence(int frame_index, int x, int y, byte flags) {
    // Get first part from frame table
    AnimPart* part = &g_AnimPartTable[g_AnimFrameTable[frame_index].first_part];

    // Render each part in the linked list
    while (part != NULL) {
        int sprite_addr = g_SpriteTable[part->sprite_index];

        // Apply flags (flip, etc.)
        if (flags & 0x01) {
            // Horizontal flip - negate X offset
            x_offset = -(part->offset_x + sprite_width);
        } else {
            x_offset = part->offset_x;
        }

        // Render sprite
        if (scaled_mode) {
            Sprite_BlitScaled(x + x_offset, y + part->offset_y, sprite_addr);
        } else {
            Sprite_BlitStandard(x + x_offset, y + part->offset_y, sprite_addr);
        }

        // Move to next part
        part = &g_AnimPartTable[part->next_part];
    }
}
```

---

## Appendix CS: Sprite Data Files

### Sprite Bank Files

| Pattern | Purpose |
|---------|---------|
| `data/hspr0_0.dat` | Sprite bank 0 (game objects) |
| `data/hspr0_1.dat` | Sprite bank 1 (alternative) |

### Sprite Bank Loading

From Sprite_LoadBank @ 0x00450990:

1. Unload previous bank if different
2. Calculate buffer sizes based on resolution
3. Load sprite data from disk
4. Initialize animation tables
5. Set resolution-specific parameters
6. Initialize terrain render tables

### Resolution-Dependent Parameters

| Condition | Value |
|-----------|-------|
| resolution >= 0x4B000 (307200) | High-res mode (scale 2) |
| resolution < 0x4B000 | Low-res mode (scale 1) |

High-res (512×384+):
- Sprite scale: 0xA0 (160)
- Grid size: 10

Low-res (<512×384):
- Sprite scale: 0x50 (80)
- Grid size: 5

---

## Appendix CU: UV Coordinate Rotation Tables

### Terrain_InitializeUVRotationTables @ 0x00451110

Initializes 4 rotations of UV coordinates for terrain textures:

```c
void Terrain_InitializeUVRotationTables(void) {
    int tile_size = DAT_0087e369;  // Texture tile size
    int max_uv = tile_size * 0x10000 - 1;  // 16.16 fixed-point max

    // Rotation 0: Normal orientation
    UV_Table[0].u0 = 0;        UV_Table[0].v0 = max_uv;
    UV_Table[0].u1 = 0;        UV_Table[0].v1 = 0;
    UV_Table[0].u2 = max_uv;   UV_Table[0].v2 = 0;

    // Rotation 1: 90° clockwise
    UV_Table[1].u0 = 0;        UV_Table[1].v0 = 0;
    UV_Table[1].u1 = max_uv;   UV_Table[1].v1 = 0;
    UV_Table[1].u2 = max_uv;   UV_Table[1].v2 = max_uv;

    // Rotation 2: 180°
    UV_Table[2].u0 = max_uv;   UV_Table[2].v0 = 0;
    UV_Table[2].u1 = max_uv;   UV_Table[2].v1 = max_uv;
    UV_Table[2].u2 = 0;        UV_Table[2].v2 = max_uv;

    // Rotation 3: 270° clockwise
    UV_Table[3].u0 = max_uv;   UV_Table[3].v0 = max_uv;
    UV_Table[3].u1 = 0;        UV_Table[3].v1 = max_uv;
    UV_Table[3].u2 = 0;        UV_Table[3].v2 = 0;
}
```

### UV Rotation Table Address

| Address | Rotation | Purpose |
|---------|----------|---------|
| 0x0059bf50 | 0 | U0, V0, U1, V1, U2, V2 (24 bytes) |
| 0x0059bf68 | 1 | 90° rotation |
| 0x0059bf80 | 2 | 180° rotation |
| 0x0059bf98 | 3 | 270° rotation |

Each entry is 24 bytes (6 × 4-byte fixed-point values).

### Triangle_CreateWithRotatedUVs @ 0x0046fb40

Creates a terrain triangle command with rotated UV coordinates:

```c
RenderCmd* Triangle_CreateWithRotatedUVs(int src_triangle, byte texture_id,
                                          int unused, char shade_mode) {
    if (cmd_buffer >= cmd_buffer_end) return NULL;

    RenderCmd* cmd = cmd_buffer;
    cmd_buffer += 0x44;  // 68 bytes per triangle command

    // Link into source triangle's chain
    cmd->next = src_triangle->next;
    src_triangle->next = cmd;

    cmd->type = 0x08;  // Textured triangle
    cmd->flags = 0;
    cmd->texture_index = texture_id;
    cmd->shade_value = (shade_mode == 2) ? 0x1F : 0x06;

    // Copy vertex screen coordinates (3 vertices × 5 dwords)
    memcpy(&cmd->v0, &src_triangle->v0, 20);
    memcpy(&cmd->v1, &src_triangle->v1, 20);
    memcpy(&cmd->v2, &src_triangle->v2, 20);

    // Apply rotated UV coordinates based on triangle orientation
    int rotation = src_triangle->rotation & 3;  // offset 0x45
    int uv_offset = rotation * 24;

    cmd->v0_u = UV_Table[uv_offset + 0];
    cmd->v0_v = UV_Table[uv_offset + 4];
    cmd->v1_u = UV_Table[uv_offset + 8];
    cmd->v1_v = UV_Table[uv_offset + 12];
    cmd->v2_u = UV_Table[uv_offset + 16];
    cmd->v2_v = UV_Table[uv_offset + 20];

    return cmd;
}
```

---

## Appendix CZ: Sprite Rendering System

### Sprite Vtable Architecture

The sprite rendering system uses a vtable pattern to support multiple bit depths:

```c
// DAT_009735b8 = pointer to current vtable
// Selected based on screen bit depth

void Render_SetBitDepthVtable(DisplayInfo* info) {
    DAT_009735d0 = info;
    DAT_009735ec = info->buffer_ptr;

    switch (info->bit_depth) {  // offset +0x20
        case 8:
            DAT_009735b8 = &vtable_8bit;   // DAT_009735d8
            break;
        case 16:
            DAT_009735b8 = &vtable_16bit;  // DAT_009735e0
            break;
        case 24:
            DAT_009735b8 = &vtable_24bit;  // DAT_009735e4
            break;
        case 32:
            DAT_009735b8 = &vtable_32bit;  // DAT_009735e8
            break;
    }

    Render_SetupBitMasks(info + 4);
}
```

### Vtable Layout

| Offset | 8-bit | 16-bit | 24-bit | 32-bit |
|--------|-------|--------|--------|--------|
| +0x00 | DrawPixel8 | DrawPixel16 | DrawPixel24 | DrawPixel32 |
| +0x04 | DrawLine8 | DrawLine16 | DrawLine24 | DrawLine32 |
| +0x08 | FillRect8 | FillRect16 | FillRect24 | FillRect32 |
| +0x0C | DrawChar8 | DrawChar16 | DrawChar24 | DrawChar32 |
| ... | ... | ... | ... | ... |
| +0x38 | BlitSprite8 | BlitSprite16 | BlitSprite24 | BlitSprite32 |

### Sprite_BlitStandard @ 0x0050edd0

```c
void Sprite_BlitStandard(int x, int y, void* sprite_data) {
    // Call through vtable
    vtable->BlitSprite(x, y, sprite_data);  // offset +0x38
}
```

### Sprite_BlitScaled @ 0x0050f6e0

```c
void Sprite_BlitScaled(int x, int y, void* sprite_data,
                       int scale_w, int scale_h) {
    // Setup scaling parameters
    FUN_0050f720(x, y, sprite_data[4], sprite_data[6], scale_w, scale_h);

    // Call scaling blit
    FUN_005643c0(0, 0, sprite_data);
}
```

### Palette System

Palette stored at DAT_00973640 in RGBA format (256 × 4 bytes):

```c
// Palette_IndexToRGBA @ 0x00402800
void Palette_IndexToRGBA(byte* output, byte palette_index) {
    int offset = palette_index * 4;

    output[0] = palette[offset + 2];  // Red
    output[1] = palette[offset + 1];  // Green
    output[2] = palette[offset + 0];  // Blue
    output[3] = 0xFF;                 // Alpha
    output[4] = palette_index;        // Original index
}
```

---

## Appendix DA: Water Animation System

### Water_AnimateMesh @ 0x0048e210

Handles water surface animation with wave displacement:

```c
void Water_AnimateMesh(void) {
    DAT_007f9180 = 0x2800;    // Wave frequency
    DAT_007f9184 = 0xCCC;     // Wave amplitude
    DAT_007f9188 = 0x10000;   // Time scale

    // Check if wave phase should advance
    if ((FUN_00459570(0xCCC) & 1) != 0) {
        DAT_007f9184 += 0x66;  // Slightly increase amplitude
    }

    DAT_005be218 = 0;
    DAT_007f97ed = 0;

    // Check if player state changed
    bool state_changed = (DAT_00885720 != DAT_007f918c);
    if (state_changed) {
        DAT_007f918c = DAT_00885720;
    }

    if (DAT_007f97eb == 0) return;  // No water objects

    // Get camera position
    int* viewport = Camera_GetViewportCoords();
    int cam_x = viewport[0];
    int cam_y = viewport[1];

    // Check if camera in water region
    int wave_start = FUN_00459570(DAT_007f9180);
    int wave_end = FUN_00459570(DAT_007f9180 + DAT_007f9184);

    bool in_water_view = (cam_x >= wave_start && cam_x < wave_end);

    // Process each water mesh segment
    for (WaterMesh* mesh = water_list; mesh != NULL;
         mesh = mesh->next) {

        if (state_changed) {
            mesh->phase++;  // Advance wave phase
        }

        // Check if mesh should be culled
        if (mesh->flags & 0x20) {
            // Calculate distance from camera
            int dist = Math_DistanceSquared(
                &player_pos, &mesh->position);

            if (dist > 0xFFF) {
                mesh->flags |= 0x08;  // Mark for removal
            }
        }

        // Handle lifetime
        if (state_changed && mesh->lifetime > 0) {
            mesh->lifetime--;
            if (mesh->lifetime == 0) {
                mesh->flags |= 0x08 | 0x10000;
            }
        }

        // Process wave animation
        if (in_water_view && IsInWaterBounds(mesh)) {
            DAT_007f97ed = 1;
            DAT_005be218 = mesh_index + 1;
            mesh->flags |= 0x04;  // Visible
        }
    }
}
```

### Water Rendering in Terrain

Water cells are detected by cell flags and rendered with:
1. Animated UV coordinates using sin/cos tables
2. Special water textures from `plstx_XXX.dat`
3. Wave displacement calculated per-frame

```c
// In Render_ProcessDepthBuckets_Main, case 0x00:
if (cell_flags & CELL_FLAG_WATER) {  // 0x04
    // Calculate animated UV offset
    int phase = (g_GameTick & 0x3F) * 0x80;

    // Per-vertex wave displacement
    for (int v = 0; v < 3; v++) {
        int u_offset = g_CosTable[(cell_uv[v] * 0x200 + phase) & 0x7FF];
        int v_offset = g_SinTable[(cell_uv[v] * 0x200 + phase) & 0x7FF];

        vertex[v].u += u_offset * 8 + 0x200000;
        vertex[v].v += v_offset * 8 + 0x200000;
    }

    // Water texture address (animated frames)
    texture_addr = DAT_00599acc +
        (cell_uv_base) * 16 +
        (g_GameTick & 1) * 0x80 +      // 2-frame animation
        (g_GameTick & 6) * 0x4000;     // 4-phase wave
}
```

---

## Appendix DB: Animation Frame Sequence Rendering

### Animation_RenderFrameSequence @ 0x004e7190

Renders a complete animation frame with all its elements:

```c
void Animation_RenderFrameSequence(uint anim_id, short screen_x,
                                    short screen_y, byte flags) {
    bool flip_x = (flags & 1) != 0;
    bool skip_outline = (flags & 2) == 0;
    byte effect_bits = (flags & 4) * 2;

    // Get first element from animation table
    AnimElement* elem = anim_elements +
        *(short*)(anim_frame_table + (anim_id & 0xFFFF) * 6) * 5;

    if (elem <= anim_elements) return;

    // Process each element in the animation
    do {
        // Skip outline elements if flag set
        if (!skip_outline ||
            (elem->flags & 0x1F0) != 0 ||
            ((elem->flags >> 8) & 0xFE) != 2) {

            uint sprite_offset = elem->sprite_id + sprite_base;
            if (sprite_offset <= sprite_base) continue;

            // Set render flags
            DAT_009735dc = (elem->flags & 0x0F) | effect_bits;

            short x_offset = elem->offset_x;
            short y_offset = elem->offset_y;

            // Handle horizontal flip
            if (flip_x) {
                x_offset = -(x_offset + *(short*)(sprite_offset + 4));
                DAT_009735dc ^= 1;  // Flip sprite flag
            }

            // Handle scaling modes
            if (DAT_008856fc == 1) {
                // Distance-based scaling
                FUN_00477420(DAT_0088421e, &x_offset, &y_offset);
                int w = *(short*)(sprite_offset + 4);
                int h = *(short*)(sprite_offset + 6);
                FUN_00477420(DAT_0088421e, &w, &h);
                Sprite_BlitScaled(screen_x + x_offset, screen_y + y_offset,
                                   sprite_offset, w, h);
            }
            else if (DAT_008856fc == 2 && DAT_00884c67 > 0x280) {
                // High-resolution scaling
                int scale_x = (DAT_00884c67 << 8) / 0x280 * 0x1E >> 5;
                int scale_y = (DAT_00884c69 << 8) / 0x1E0 * 0x1E >> 5;
                // Apply scaling and blit...
            }
            else {
                // Standard blit
                Sprite_BlitStandard(screen_x + x_offset, screen_y + y_offset,
                                     sprite_offset);
            }
        }

        // Move to next element (linked list via index)
        elem = anim_elements + elem->next_index * 5;

    } while (elem > anim_elements);

    DAT_009735dc = 0;
    DAT_008856fc = 0;
}
```

### Animation Element Structure (10 bytes)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 2 | Sprite ID offset |
| +0x02 | 2 | X offset from anchor |
| +0x04 | 2 | Y offset from anchor |
| +0x06 | 2 | Flags (flip, blend mode) |
| +0x08 | 2 | Next element index |

---

## Appendix DP: Animation Frame Rendering System

### Animation Frame Sequence

**Function:** `Animation_RenderFrameSequence @ 0x004e7190`

Renders multi-element sprite animations with support for scaling, mirroring, and depth-based sizing.

**Parameters:**
- `param_1` - Animation ID (16-bit)
- `param_2` - Screen X position
- `param_3` - Screen Y position
- `param_4` - Flags (bit 0=mirror, bit 1=skip shadow, bit 2=additional flag)

**Animation Data Tables:**
- `DAT_005a7d84` - Animation index table (6 bytes per entry)
- `DAT_005a7d88` - Frame element table (10 bytes per element)
- `DAT_005a7d54` - Sprite bank base pointer

**Frame Element Structure (10 bytes):**
```c
struct FrameElement {
    ushort sprite_offset;  // +0x00: Offset in sprite bank
    short offset_x;        // +0x02: X offset from center
    short offset_y;        // +0x04: Y offset from center
    ushort flags;          // +0x06: Rendering flags (low 4 bits = blend mode)
    ushort next_element;   // +0x08: Index of next element (linked list)
};
```

**Rendering Modes (DAT_008856fc):**
| Mode | Description |
|------|-------------|
| 0x00 | Standard blit (Sprite_BlitStandard) |
| 0x01 | Distance-scaled (Render_CalculateDistanceScale + Sprite_BlitScaled) |
| 0x02 | Resolution-scaled (for resolutions > 640×480) |

**Resolution Scaling Formula (Mode 2):**
```c
scale_x = (screen_width << 8) / 640 * 30 >> 5;  // 30/32 = 0.9375
scale_y = (screen_height << 8) / 480 * 30 >> 5;
```

### Animation Setup

**Function:** `Animation_SetupFromBank @ 0x004b0ad0`

Initializes animation state from animation bank:

**Animation Bank Entry (11 bytes at DAT_0059f8d8):**
```c
struct AnimationBankEntry {
    byte dispatch_type; // +0x00: renderer dispatch family
    byte timing_step;   // +0x03: signed track timing increment
    byte update_mode;   // +0x04: Animation_UpdateObjectTrack mode (1-4)
    byte bank_arg_5;    // +0x05: render-type-specific data
    byte bank_arg_6;    // +0x06: render-type-specific data
    byte bank_arg_7;    // +0x07: render-type-specific data
    byte reset_frame;   // +0x08: nonzero forces track frame/offset reset
    ushort flags;       // +0x09: initial native track flags
};
```

---

## Appendix DR: Sprite Object Rendering (Sprite_RenderObject)

### Overview

**Function:** `Sprite_RenderObject @ 0x00411c90`

This is the largest rendering function (~1200 lines), handling all object types through a massive switch statement.

**Parameters:**
- `param_1` - Render command structure
- `param_2` - Object pointer
- `param_3` - Screen X offset
- `param_4` - Screen Y offset
- `param_5` - Additional data pointer
- `param_6` - Extra data pointer
- `param_7` - Mode flag
- `param_8` - Shadow flag

### Object Type Rendering (switch on object+0x70 - 1)

| Case | Type | Rendering |
|------|------|-----------|
| 0 | Person riding vehicle | Multi-sprite with rider animation |
| 1 | Building | Foundation + structure sprites |
| 2, 12 | Terrain object | Cell-based with special flags |
| 3 | Simple object | Single sprite with scale |
| 4 | Effect | Animated effect sprite |
| 5 | Unit | Full unit with equipment |
| 6 | Vehicle | Vehicle with passenger slots |
| 8 | Projectile | Animated projectile |
| 9, 10 | Markers | Status indicators |
| 11 | Special | Custom rendering |
| 13 | Spell effect | Effect with shading |

### Key Sprite Tables

**Sprite Data at DAT_005a7d50:**
| Offset | Size | Purpose |
|--------|------|---------|
| +0x104 | 2 | Selection box width |
| +0x106 | 2 | Selection box height |
| +0x144 | 2 | Health bar width |
| +0x146 | 2 | Health bar height |
| +0x174 | 2 | Mana bar offset |
| +0x1a4 | 2 | Building frame width |
| +0x1a6 | 2 | Building frame height |
| +0x1e4 | 2 | Effect sprite base |
| +0x25c | 2 | Unit sprite width |
| +0x25e | 2 | Unit sprite height |
| +0x2db4 | 2 | UI element width |
| +0x2db6 | 2 | UI element height |

---

## Appendix DX: Water Wave Animation System

### Key Functions

| Function | Address | Purpose |
|----------|---------|---------|
| Water_AnimateMesh | 0x0048e210 | Main water animation loop |
| Water_UpdateWavePhase | 0x0048e990 | Update wave positions |
| Water_RenderObjects | 0x004a75f0 | Render objects on water |
| Water_SetupMesh | 0x0048e730 | Initialize water mesh |

### Water Animation Constants

```c
DAT_007f9180 = 0x2800;   // Wave amplitude base
DAT_007f9184 = 0xccc;    // Wave frequency
DAT_007f9188 = 0x10000;  // Wave height cap
```

### Wave Phase Update Algorithm

From `Water_UpdateWavePhase @ 0x0048e990`:

```c
// Wave propagation with reflection at boundaries
wave->phase += wave->velocity;

if (wave->velocity < 0) {
    // Moving toward shore
    if (wave->phase <= neighbor->phase + neighbor->height) {
        // Reflect at boundary
        wave->phase = boundary;
        // Transfer momentum to neighbor (energy dissipation)
        neighbor->velocity = (wave_type_table[wave->type] * wave->velocity) >> 8;
        wave->velocity = 0;
    }
} else {
    // Moving away from shore
    boundary = (wave->link) ? wave->link->phase : MAX_HEIGHT;
    if (wave->phase >= boundary - wave->height) {
        // Bounce back
        wave->velocity = -((wave_type_table[wave->type] * wave->velocity) >> 8);
        Sound_Play(0, 0xe4, 1);  // Wave splash sound
    }
}

// Apply acceleration toward equilibrium
wave->velocity += 0x555;  // Constant acceleration
```

### Water Object Linked List Structure (0x2D bytes per entry)

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | Animation frame counter |
| +0x04 | 4 | Wave phase position |
| +0x08 | 4 | Wave height |
| +0x0C | 4 | Wave velocity |
| +0x14 | 2 | World X position |
| +0x16 | 2 | World Y position |
| +0x20 | 1 | Wave type index |
| +0x21 | 4 | Flags |
| +0x25 | 4 | Prev link pointer |
| +0x29 | 4 | Next link pointer |

### Water Flags

| Bit | Meaning |
|-----|---------|
| 0x04 | Currently hovered by cursor |
| 0x08 | Marked for removal |
| 0x10 | Height auto-generated |
| 0x0002_0000 | Awaiting activation |
| 0x0004_0000 | At boundary |
| 0x0008_0000 | Velocity changed this frame |
| 0x0010_0000 | Has texture overlay |

---

## Appendix EB: Rotated Quad Rendering

### Render_DrawRotatedQuad @ 0x0040a560

Renders a textured quad with rotation (used for spinning icons, compass, etc.):

```c
void Render_DrawRotatedQuad(int param) {
    // Setup clipping region to panel background
    UI_RenderPanelBackground(surface, x, y, size);
    Render_SetClipRegion({0, 0, width, height});

    // Calculate scaled half-size
    int half_size = ((param->scale * param->size * 32) >> 8) / 2;

    // Generate quad corners (before rotation)
    int corners_x[4] = {-half_size, -half_size, half_size, half_size};
    int corners_y[4] = {-half_size, half_size, half_size, -half_size};

    // Get rotation angle from sin/cos tables
    int cos_val = g_CosTable[param->angle];
    int sin_val = g_SinTable[param->angle];

    // Rotate corners and offset to center position
    for (i = 0; i < 4; i++) {
        rotated_x[i] = ((sin_val * corners_y[i] - cos_val * corners_x[i]) >> 16) + center_y;
        rotated_y[i] = ((sin_val * corners_x[i] + cos_val * corners_y[i]) >> 16) + center_x;
    }

    // Build vertex structures for rasterizer
    for (i = 0; i < 4; i++) {
        verts[i].x = rotated_x[i];
        verts[i].y = rotated_y[i];
        verts[i].z = 0x200000;  // Fixed depth
    }

    // Set UV coordinates from param
    verts[0].u = param->u0 << 16 - 1;
    verts[1].u = param->u1 << 16 - 1;
    // ... etc

    // Set texture pointer
    DAT_0098a004 = texture_base + 0x400;
    DAT_0098a008 = 0;

    // Rasterize as two triangles
    Rasterizer_Main(verts[0], verts[1], verts[2], 3);  // Mode 3 = textured
    Rasterizer_Main(verts[0], verts[2], verts[3], 3);

    Render_SetClipRegion(NULL);  // Reset clipping
}
```

---
