# Contract: VIC-II Device Implementation

**Feature**: C64 Emulator Demo **Component**: `Vic2Device` (src/devices/vic2.rs)
**Trait**: `Device` **Date**: 2025-11-20

This contract defines the behavior of the VIC-II video chip emulation for 40×25
character text mode.

---

## Overview

The `Vic2Device` implements the `Device` trait to emulate the MOS 6567/6569
VIC-II video interface chip. Phase 1 focuses on text mode display registers
only—sprites, bitmap modes, and raster interrupts are deferred.

---

## Device Registration

**Address Range**: $D000-$D3FF (1KB, mirrored every 64 bytes) **Size**: 1024
bytes (`size()` returns `0x0400`)

**Registration**:

```rust
let vic2 = Box::new(Vic2Device::new());
mapped_memory.add_device(0xD000, vic2)?;
```

---

## Register Map (Phase 1: Text Mode Only)

### Critical Registers (Fully Implemented)

| Offset | Address | Name               | Read | Write | Default | Description                        |
| ------ | ------- | ------------------ | ---- | ----- | ------- | ---------------------------------- |
| $11    | $D011   | Control Register 1 | R/W  | R/W   | $1B     | DEN, RSEL, YSCROLL, ECM, BMM, RST8 |
| $16    | $D016   | Control Register 2 | R/W  | R/W   | $C8     | RES, MCM, CSEL, XSCROLL            |
| $18    | $D018   | Memory Setup       | R/W  | R/W   | $15     | Screen & character memory pointers |
| $20    | $D020   | Border Color       | R/W  | R/W   | $0E     | Border color (4-bit)               |
| $21    | $D021   | Background Color 0 | R/W  | R/W   | $06     | Background color (4-bit)           |
| $22    | $D022   | Background Color 1 | R/W  | R/W   | $00     | Extended color mode (deferred)     |
| $23    | $D023   | Background Color 2 | R/W  | R/W   | $00     | Extended color mode (deferred)     |
| $24    | $D024   | Background Color 3 | R/W  | R/W   | $00     | Extended color mode (deferred)     |

### Status Registers (Basic Implementation)

| Offset | Address | Name             | Read | Write | Default | Description                                    |
| ------ | ------- | ---------------- | ---- | ----- | ------- | ---------------------------------------------- |
| $12    | $D012   | Raster Counter   | R    | R/W   | $00     | Current raster line (low 8 bits)               |
| $19    | $D019   | Interrupt Status | R    | W     | $00     | IRQ flags (read clears, write ignored Phase 1) |
| $1A    | $D01A   | Interrupt Enable | R/W  | R/W   | $00     | IRQ mask (stub Phase 1)                        |

### Stubbed Registers (Return 0, Ignore Writes)

| Offset Range | Address Range | Name                                          | Phase 1 Behavior                  |
| ------------ | ------------- | --------------------------------------------- | --------------------------------- |
| $00-$0F      | $D000-$D00F   | Sprite X/Y Positions                          | Always return $00, writes ignored |
| $10          | $D010         | Sprite MSB X                                  | Always return $00, writes ignored |
| $13-$14      | $D013-$D014   | Light Pen X/Y                                 | Always return $00, writes ignored |
| $15          | $D015         | Sprite Enable                                 | Always return $00, writes ignored |
| $17          | $D017         | Sprite Y Expansion                            | Always return $00, writes ignored |
| $1B-$1F      | $D01B-$D01F   | Sprite Priority/Multicolor/X Expand/Collision | Always return $00, writes ignored |
| $25-$2E      | $D025-$D02E   | Sprite Multicolor & Colors                    | Always return $00, writes ignored |

---

## Device Trait Implementation

### `read(&self, offset: u16) -> u8`

**Behavior**:

```rust
impl Device for Vic2Device {
    fn read(&self, offset: u16) -> u8 {
        // Handle register mirroring (repeat every 64 bytes)
        let register = (offset & 0x3F) as u8;

        match register {
            0x11 => self.control_register_1,
            0x12 => self.raster_counter,
            0x16 => self.control_register_2,
            0x18 => self.memory_pointers,
            0x19 => self.interrupt_status,
            0x1A => self.interrupt_enable,
            0x20 => self.border_color,
            0x21 => self.background_color_0,
            0x22 => self.background_color_1,
            0x23 => self.background_color_2,
            0x24 => self.background_color_3,

            // Sprite registers (stubbed)
            0x00..=0x0F | 0x10 | 0x13..=0x15 | 0x17 | 0x1B..=0x1F | 0x25..=0x2E => 0x00,

            // Unimplemented/unused registers
            _ => 0xFF,  // Open bus behavior
        }
    }
}
```

**Register Mirroring**:

- VIC-II has incomplete address decoding
- Registers repeat every 64 bytes in $D000-$D3FF range
- Example: $D011, $D051, $D091, etc. all map to Control Register 1

---

### `write(&mut self, offset: u16, value: u8)`

**Behavior**:

```rust
impl Device for Vic2Device {
    fn write(&mut self, offset: u16, value: u8) {
        let register = (offset & 0x3F) as u8;

        match register {
            0x11 => self.control_register_1 = value,
            0x12 => self.raster_compare = value,  // Phase 2: trigger interrupt
            0x16 => self.control_register_2 = value,
            0x18 => {
                self.memory_pointers = value;
                self.update_memory_addresses();  // Recompute screen/char base
            }
            0x19 => {
                // Phase 1: Stub (Phase 2: clear IRQ flags)
            }
            0x1A => self.interrupt_enable = value,
            0x20 => self.border_color = value & 0x0F,       // Mask to 4 bits
            0x21 => self.background_color_0 = value & 0x0F,
            0x22 => self.background_color_1 = value & 0x0F,
            0x23 => self.background_color_2 = value & 0x0F,
            0x24 => self.background_color_3 = value & 0x0F,

            // Sprite registers (stubbed - accept writes, do nothing)
            0x00..=0x0F | 0x10 | 0x13..=0x15 | 0x17 | 0x1B..=0x1F | 0x25..=0x2E => {}

            // Other registers ignored
            _ => {}
        }
    }
}
```

**Color Register Masking**:

- Color values stored as 4-bit (0-15)
- Upper 4 bits ignored on write
- Reads return full 8-bit value (upper bits undefined)

---

## Register Semantics

### $D011: Control Register 1

**Bit Layout**:

```
Bit 7: RST8 - Raster compare bit 8 (with $D012 forms 9-bit value)
Bit 6: ECM  - Extended Color Mode (0=off, 1=on) [deferred]
Bit 5: BMM  - Bitmap Mode (0=text, 1=bitmap) [deferred]
Bit 4: DEN  - Display Enable (0=blank, 1=display)
Bit 3: RSEL - Row Select (0=24 rows, 1=25 rows)
Bits 0-2: YSCROLL - Fine vertical scroll (0-7 pixels)
```

**Default**: `$1B` (binary 00011011)

- RST8=0, ECM=0, BMM=0, DEN=1, RSEL=1, YSCROLL=3

**Phase 1 Behavior**:

- Bit 4 (DEN): Must be 1 for display (checked by rendering logic)
- Bit 3 (RSEL): Determines 24 vs 25 row mode
- Bits 0-2 (YSCROLL): Stored but not applied to display (defer scroll effects)
- Bits 5-6: Ignored (text mode only)

---

### $D016: Control Register 2

**Bit Layout**:

```
Bit 5: RES  - Reset (unused, always 0)
Bit 4: MCM  - Multicolor Mode (0=off, 1=on) [deferred]
Bit 3: CSEL - Column Select (0=38 cols, 1=40 cols)
Bits 0-2: XSCROLL - Fine horizontal scroll (0-7 pixels)
```

**Default**: `$C8` (binary 11001000)

- RES=1 (unused), MCM=1 (ignored), CSEL=1 (40 columns), XSCROLL=0

**Phase 1 Behavior**:

- Bit 3 (CSEL): Must be 1 for 40-column mode
- Bits 0-2 (XSCROLL): Stored but not applied (defer scroll effects)
- Bit 4: Ignored (standard color mode only)

---

### $D018: Memory Setup Register

**Bit Layout**:

```
Bits 4-7: VM13-VM10 - Screen memory base address / 1024
Bits 1-3: CB13-CB11 - Character ROM base address / 2048
Bit 0: Unused
```

**Default**: `$15` (binary 00010101)

- VM13-VM10 = 0001 → Screen at $0400 (1 × 1024 = $0400)
- CB13-CB11 = 010 → Character ROM at $1000 in VIC-II address space

**Memory Address Calculation**:

```rust
fn update_memory_addresses(&mut self) {
    // Screen memory (1KB blocks)
    let vm_bits = (self.memory_pointers >> 4) & 0x0F;
    self.screen_memory_base = (vm_bits as u16) * 1024;

    // Character ROM (2KB blocks)
    let cb_bits = (self.memory_pointers >> 1) & 0x07;
    self.character_rom_base = (cb_bits as u16) * 2048;
}
```

**VIC-II Address Space** (different from CPU address space):

- VIC-II has 14-bit address bus (16KB addressable)
- Bank switching via CIA2 determines which 16KB of 64KB RAM is visible
- Phase 1: Fixed bank (VIC-II sees $0000-$3FFF as CPU $0000-$3FFF)
- Character ROM at VIC-II $1000-$1FFF maps to CPU $D000-$DFFF

---

### $D012: Raster Counter

**Read Behavior**:

```rust
fn read_raster_counter(&self) -> u8 {
    // Return current raster line (0-312 PAL, 0-262 NTSC)
    // Phase 1: Simple incrementing counter
    self.raster_counter
}
```

**Write Behavior** (Phase 2):

- Sets raster interrupt compare value
- Phase 1: Writes accepted but no interrupt generated

**Raster Counter Update** (called by emulation loop):

```rust
pub fn tick_raster(&mut self) {
    self.raster_counter = (self.raster_counter + 1) % 263;  // NTSC: 0-262
    // Phase 2: Check if raster_counter == interrupt_compare, set IRQ flag
}
```

---

### $D019: Interrupt Status Register

**Bit Layout**:

```
Bit 7: IRQ - Any IRQ occurred (logical OR of bits 0-3)
Bit 3: ILP - Light pen interrupt
Bit 2: IMMC - Sprite-sprite collision interrupt
Bit 1: IMBC - Sprite-background collision interrupt
Bit 0: IRST - Raster interrupt
```

**Phase 1 Behavior**:

- Read: Always return `$00` (no interrupts)
- Write: Ignored (no interrupt clearing needed yet)

**Phase 2 Behavior** (deferred):

- Read: Return current interrupt flags, **clear flags on read**
- Write: Clear individual flags (write 1 to clear)

---

### $D01A: Interrupt Enable Register

**Bit Layout**: Same as $D019 (mask for enabling interrupts)

**Phase 1 Behavior**:

- Read/write supported
- Value stored but not acted upon (no interrupt generation)

**Phase 2 Behavior** (deferred):

- Check mask bits against status register
- Generate IRQ signal if enabled interrupt occurs

---

### $D020-$D024: Color Registers

**Value Range**: 0-15 (4-bit palette index) **Storage**: 8-bit register, upper 4
bits undefined

**Read/Write**:

```rust
// Write
self.border_color = value & 0x0F;

// Read
self.border_color  // May return upper bits set, caller should mask
```

**Usage**:

- JavaScript calls `get_border_color()` / `get_background_color()`
- Display renderer maps color index to RGB palette

---

## Public API (Beyond Device Trait)

```rust
impl Vic2Device {
    /// Create new VIC-II device with C64 boot defaults
    pub fn new() -> Self;

    /// Get screen memory base address in VIC-II address space
    pub fn screen_memory_base(&self) -> u16;

    /// Get character ROM base address in VIC-II address space
    pub fn character_rom_base(&self) -> u16;

    /// Get border color (0-15)
    pub fn border_color(&self) -> u8;

    /// Get background color (0-15)
    pub fn background_color(&self) -> u8;

    /// Check if display is enabled (DEN bit in $D011)
    pub fn display_enabled(&self) -> bool;

    /// Get number of rows (24 or 25 based on RSEL bit)
    pub fn row_count(&self) -> u8;

    /// Get number of columns (38 or 40 based on CSEL bit)
    pub fn column_count(&self) -> u8;

    /// Update raster counter (called by emulation loop)
    pub fn tick_raster(&mut self);
}
```

---

## KERNAL Initialization Sequence

**SCINIT Routine** (KERNAL $FF81):

1. Write `$1B` to $D011 (enable display, 25 rows)
2. Write `$C8` to $D016 (40 columns)
3. Write `$15` to $D018 (screen at $0400, char ROM at $1000)
4. Write `$0E` to $D020 (light blue border)
5. Write `$06` to $D021 (blue background)
6. Clear screen RAM ($0400-$07E7) to `$20` (space character)
7. Clear color RAM ($D800-$DBFF) to `$0E` (light blue text)

**Expected Results After Boot**:

- `read(0x11)` returns `$1B`
- `read(0x20)` returns `$0E`
- `read(0x21)` returns `$06`
- `screen_memory_base()` returns `$0400`

---

## Phase 1 vs Phase 2 Differences

| Feature             | Phase 1            | Phase 2                |
| ------------------- | ------------------ | ---------------------- |
| Text mode registers | ✅ Full            | ✅ Full                |
| Raster counter      | ✅ Incrementing    | ✅ Cycle-accurate      |
| Raster interrupts   | ❌ Stub            | ✅ IRQ generation      |
| Scroll registers    | ✅ Store only      | ✅ Applied to display  |
| Sprite registers    | ❌ Stub            | ✅ Full sprite support |
| Bitmap mode         | ❌ Not implemented | Future                 |
| Extended color mode | ❌ Not implemented | Future                 |

---

## Contract Validation

**VIC-II Device Must**:

- Return default values after `new()`
- Mask color registers to 4 bits on write
- Mirror registers every 64 bytes
- Update memory addresses when $D018 written
- Ignore writes to sprite registers (no panic)
- Return valid screen/char base addresses

**CPU/KERNAL May Assume**:

- Screen RAM at $0400 unless $D018 changed
- Color RAM always at $D800-$DBFF (hardware-fixed, not VIC-II register)
- Border/background colors modifiable at any time
- Sprite registers safe to write (even if stubbed)

---

**Contract Status**: ✅ Complete **Next**: CIA Device contract
