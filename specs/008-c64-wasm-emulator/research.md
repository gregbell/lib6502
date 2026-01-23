# Research: Commodore 64 WASM Emulator

**Date**: 2025-01-22
**Feature Branch**: `008-c64-wasm-emulator`

## Executive Summary

This document consolidates research findings for implementing a browser-based C64 emulator using the existing lib6502 CPU core. All technical decisions have been made based on the spec requirements (frame-accurate VIC-II, 6581 SID only, high-level IEC protocol).

---

## 1. VIC-II Video Chip Emulation

### Decision: Scanline-Based Frame-Accurate Rendering

**Rationale**: Scanline-based rendering provides the best balance between accuracy (supporting raster interrupts and split-screen effects) and performance (60 FPS in browser). Cycle-exact emulation is not required per FR-020.

**Alternatives Considered**:
- Cycle-exact emulation: Too slow for browser, requires 10x more CPU
- Full-frame emulation: Misses mid-frame register changes (raster effects broken)

### Register Map ($D000-$D02E)

| Range | Purpose |
|-------|---------|
| $D000-$D010 | Sprite positions (X/Y for 8 sprites) |
| $D011 | Control reg 1: YSCROLL, DEN, BMM, ECM, RST8 |
| $D012 | Raster counter (triggers IRQ on match) |
| $D016 | Control reg 2: XSCROLL, CSEL, MCM |
| $D018 | Memory pointers (video matrix, character base) |
| $D019 | Interrupt status (clear on read) |
| $D01A | Interrupt enable mask |
| $D01E-$D01F | Collision registers (sprite-sprite, sprite-bg) |
| $D020-$D02E | Color registers (border, background, sprites) |

### Display Modes

| Mode | ECM | BMM | MCM | Resolution | Colors |
|------|-----|-----|-----|------------|--------|
| Standard Text | 0 | 0 | 0 | 320x200 | 2/cell |
| Multicolor Text | 0 | 0 | 1 | 160x200 | 4/cell |
| Standard Bitmap | 0 | 1 | 0 | 320x200 | 2/8x8 |
| Multicolor Bitmap | 0 | 1 | 1 | 160x200 | 4/8x8 |
| ECM Text | 1 | 0 | 0 | 320x200 | 2+4bg |

### Sprite Implementation

- 8 sprites, 24x21 pixels each (12x21 in multicolor mode)
- 63 bytes per sprite (pointer at Screen RAM + $3F8)
- Priority: Sprite 0 highest, sprite-to-sprite collision via hardware
- X-expansion and Y-expansion double sprite size

### Raster Interrupt

```rust
// On scanline completion
if current_raster == raster_compare {
    interrupt_flags |= 0x01;  // Set raster flag
    if interrupt_mask & 0x01 != 0 {
        irq_pending = true;
    }
}
```

### Reference Implementations

- **rust64** (Rust): Best reference for Rust patterns, cycle-based approach
- **VICE x64** (C): Fast emulation, good compatibility baseline
- **zinc64** (Rust): Modular toolkit, Device trait similar to lib6502

---

## 2. SID Audio Chip Emulation

### Decision: 6581 with Simplified Biquad Filter

**Rationale**: The 6581 is the original C64 SID variant per FR-030. A simplified biquad filter provides ~90% accuracy at ~10% of reSID CPU cost, sufficient for games.

**Alternatives Considered**:
- Full reSID emulation: Too CPU-intensive for 60 FPS browser target
- 8580 variant: Explicitly out of scope per spec

### Register Map ($D400-$D41C)

| Range | Purpose |
|-------|---------|
| $D400-$D406 | Voice 1 (freq, pulse width, control, ADSR) |
| $D407-$D40D | Voice 2 (same layout) |
| $D40E-$D414 | Voice 3 (same layout) |
| $D415-$D416 | Filter cutoff (11-bit) |
| $D417 | Filter resonance + routing |
| $D418 | Filter mode + master volume |
| $D41B-$D41C | Voice 3 readback (oscillator/envelope) |

### Voice Architecture

Each voice contains:
- 24-bit phase accumulator (16-bit frequency register)
- Waveform generator (triangle, sawtooth, pulse, noise)
- ADSR envelope generator (4-bit each: attack, decay, sustain, release)

### Waveform Generation

```rust
// Sawtooth
let sawtooth = (accumulator >> 12) & 0xFFF;

// Triangle
let msb = (accumulator >> 23) & 1;
let triangle = if msb == 1 {
    (!((accumulator >> 12) & 0x7FF) & 0x7FF) << 1
} else {
    ((accumulator >> 12) & 0x7FF) << 1
};

// Pulse
let pulse = if ((accumulator >> 12) & 0xFFF) < pulse_width { 0xFFF } else { 0 };

// Noise (23-bit LFSR)
let feedback = ((lfsr >> 22) ^ (lfsr >> 17)) & 1;
lfsr = ((lfsr << 1) | feedback) & 0x7FFFFF;
```

### ADSR Envelope

Rate periods (CPU cycles between envelope changes):

| Value | Attack | Decay/Release |
|-------|--------|---------------|
| 0 | 9 (2ms) | 9 (6ms) |
| 15 | 31251 (8s) | 31251 (24s) |

Exponential decay approximation via dividers:
- 255-93: divider 1
- 93-54: divider 2
- 54-26: divider 4
- 26-14: divider 8
- 14-6: divider 16
- 6-0: divider 30

### Filter (Simplified Biquad)

```rust
fn process(&mut self, input: f32) -> f32 {
    let f = 2.0 * (self.cutoff / SAMPLE_RATE).sin();
    let q_inv = (15.0 - self.resonance as f32) / 8.0;

    self.low += f * self.band;
    let high = input - self.low - q_inv * self.band;
    self.band += f * high;

    match self.mode {
        LowPass => self.low,
        BandPass => self.band,
        HighPass => high,
        Notch => self.low + high,
    }
}
```

### Sample Rate Conversion

Direct decimation: ~23 SID clocks per 44.1kHz sample

```rust
fn generate_sample(&mut self) -> f32 {
    for _ in 0..23 {
        self.sid_clock();
    }
    self.get_output()
}
```

### Web Audio Integration

Use AudioWorklet (not deprecated ScriptProcessorNode):

```javascript
class SidProcessor extends AudioWorkletProcessor {
    process(inputs, outputs, parameters) {
        const samples = emulator.get_audio_samples(128);
        outputs[0][0].set(samples);
        return true;
    }
}
```

---

## 3. CIA Chip Emulation

### Decision: Full Register Emulation, Simplified Serial Port

**Rationale**: Both CIAs are essential for keyboard, joystick, and timers. The serial data register (SDR) can be minimal since we use high-level IEC protocol.

### Register Map ($00-$0F, mirrored)

| Offset | Register | Description |
|--------|----------|-------------|
| $00 | PRA | Peripheral Data Register A |
| $01 | PRB | Peripheral Data Register B |
| $02 | DDRA | Data Direction A |
| $03 | DDRB | Data Direction B |
| $04-$05 | TA | Timer A (16-bit) |
| $06-$07 | TB | Timer B (16-bit) |
| $08-$0B | TOD | Time of Day (BCD) |
| $0C | SDR | Serial Data Register |
| $0D | ICR | Interrupt Control |
| $0E | CRA | Control Register A |
| $0F | CRB | Control Register B |

### Timer Implementation

- 16-bit countdown timer with latch reload on underflow
- One-shot mode: stops after underflow
- Continuous mode: reloads and continues
- Timer B can chain to Timer A underflows

### CIA1 ($DC00) - Keyboard Matrix

8x8 matrix scanned by:
1. Write column select to Port A ($DC00)
2. Read row state from Port B ($DC01)
3. Low bits indicate pressed keys

### CIA1 ($DC00) - Joystick

- Port A: Joystick 2 (bits 0-4: up/down/left/right/fire)
- Port B: Joystick 1 (bits 0-4)
- Active low (0 = pressed)

### CIA2 ($DD00) - VIC Bank + IEC

- Bits 0-1: VIC-II bank selection (inverted: 11=bank0, 00=bank3)
- Bits 3-5: IEC bus outputs (ATN, CLK, DATA)
- Bits 6-7: IEC bus inputs (CLK, DATA)

### Interrupt Handling

```rust
// Reading ICR clears all flags
fn read_icr(&mut self) -> u8 {
    let flags = self.interrupt_flags;
    self.interrupt_flags = 0;
    self.irq_line = false;
    if flags & 0x1F != 0 { flags | 0x80 } else { flags }
}

// Writing ICR sets/clears mask
fn write_icr(&mut self, value: u8) {
    let mask = value & 0x1F;
    if value & 0x80 != 0 {
        self.interrupt_mask |= mask;
    } else {
        self.interrupt_mask &= !mask;
    }
}
```

---

## 4. Memory Banking

### Decision: Custom MappedMemory with ROM/RAM Overlay

**Rationale**: The C64's complex banking cannot use the standard MappedMemory approach. A custom implementation handles CPU vs VIC-II access paths and ROM write-through.

### Banking Configuration ($01 bits 0-2)

| $01 & 7 | $A000-$BFFF | $D000-$DFFF | $E000-$FFFF |
|---------|-------------|-------------|-------------|
| 0 | RAM | RAM | RAM |
| 1 | RAM | RAM | RAM |
| 2 | RAM | CHAR ROM | KERNAL |
| 3 | BASIC | CHAR ROM | KERNAL |
| 4 | RAM | RAM | RAM |
| 5 | RAM | I/O | RAM |
| 6 | RAM | I/O | KERNAL |
| 7 (default) | BASIC | I/O | KERNAL |

### Key Implementation Rules

1. **Writes to ROM areas always go to underlying RAM**
2. **VIC-II sees character ROM at $1000-$1FFF in banks 0 and 2**
3. **VIC-II never sees I/O, BASIC, or KERNAL**
4. **I/O area writes go to devices when I/O is visible, RAM otherwise**

### 6510 Port Implementation

```rust
pub struct Port6510 {
    ddr: u8,   // $00 - Data Direction
    data: u8,  // $01 - Data (bits 0-2 = LORAM/HIRAM/CHAREN)
}

impl Port6510 {
    pub fn bank_config(&self) -> u8 {
        self.data & 0x07
    }
}
```

---

## 5. 1541 Disk Drive (High-Level IEC)

### Decision: High-Level IEC Protocol Emulation

**Rationale**: Per FR-070, full 6502 drive CPU emulation is not required. High-level protocol handles .D64 file access for ~95% of software.

**Alternatives Considered**:
- Full 1541 emulation: Required for copy protection, but too complex for initial implementation
- Direct file injection: Would break disk directory functionality

### .D64 File Format

- 174,848 bytes (35 tracks, 683 sectors)
- Track 18 contains directory and BAM (Block Availability Map)
- Sectors are 256 bytes each
- File entries in directory chain starting at track 18, sector 1

### IEC Protocol Commands

| Command | Description |
|---------|-------------|
| LISTEN | Put device in listen mode |
| TALK | Put device in talk mode |
| OPEN | Open channel to file |
| CLOSE | Close channel |
| DATA | Transfer data bytes |
| UNLISTEN/UNTALK | Release bus |

### Implementation Pattern

```rust
pub struct Drive1541 {
    mounted_d64: Option<D64Image>,
    channels: [Channel; 16],
    device_number: u8,  // Usually 8
    status: DriveStatus,
}

impl Drive1541 {
    pub fn iec_command(&mut self, cmd: IecCommand) -> IecResponse {
        match cmd {
            IecCommand::Open(channel, filename) => self.open_file(channel, &filename),
            IecCommand::Read(channel) => self.read_byte(channel),
            IecCommand::Write(channel, byte) => self.write_byte(channel, byte),
            IecCommand::Close(channel) => self.close_channel(channel),
        }
    }
}
```

---

## 6. WASM Integration Architecture

### Decision: Extend Existing Demo Infrastructure

**Rationale**: The existing `demo/` directory provides proven WASM bindings pattern. C64 extends this with additional display and audio APIs.

### WASM API Additions

```rust
#[wasm_bindgen]
impl C64Emulator {
    // Initialization
    pub fn new() -> C64Emulator;
    pub fn load_roms(&mut self, basic: &[u8], kernal: &[u8], charrom: &[u8]) -> bool;

    // Emulation control
    pub fn step_frame(&mut self);  // Run one full frame (~19656 cycles PAL)
    pub fn reset(&mut self);
    pub fn pause(&mut self);
    pub fn resume(&mut self);

    // Display
    pub fn get_framebuffer_ptr(&self) -> *const u8;  // 320x200 indexed color

    // Audio
    pub fn get_audio_samples(&mut self, count: u32) -> Vec<f32>;

    // Input
    pub fn key_down(&mut self, keycode: u8);
    pub fn key_up(&mut self, keycode: u8);
    pub fn set_joystick(&mut self, port: u8, state: u8);

    // File loading
    pub fn mount_d64(&mut self, data: &[u8]) -> bool;
    pub fn load_prg(&mut self, data: &[u8]) -> bool;

    // State management
    pub fn save_state(&self) -> Vec<u8>;
    pub fn load_state(&mut self, data: &[u8]) -> bool;
}
```

### Performance Budget

| Component | Target |
|-----------|--------|
| Frame time | <16.67ms (60 FPS) |
| CPU emulation | ~6ms (1M cycles) |
| VIC-II rendering | ~2ms |
| SID audio | ~1ms |
| JavaScript overhead | ~2ms |
| Headroom | ~5ms |

---

## 7. Existing lib6502 Infrastructure Assessment

### Already Implemented (Reusable)

- CPU with cycle-accurate execution
- Level-sensitive IRQ support
- Device trait for memory-mapped I/O
- MappedMemory for device routing
- WASM bindings via wasm-bindgen
- Existing demo frontend structure

### Needs Addition

- NMI support for RESTORE key
- 6510 I/O port device
- C64-specific memory banking logic
- VIC-II, SID, CIA device implementations
- 1541 disk drive emulation
- C64 keyboard matrix mapping

### Constitution Compliance

| Principle | Implementation |
|-----------|----------------|
| I. Modularity | Each chip as separate Device impl |
| II. WASM Portability | All pure Rust, no_std compatible |
| III. Cycle Accuracy | CPU accurate; VIC-II frame-accurate |
| IV. Clarity | Clear register documentation |
| V. Table-Driven | Register tables in each device |

---

## Sources

### VIC-II
- [The MOS 6567/6569 video controller (VIC-II)](https://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt)
- [rust64 GitHub](https://github.com/kondrak/rust64)
- [C64-Wiki Graphics Modes](https://www.c64-wiki.com/wiki/Graphics_Modes)

### SID
- [Oxyron SID Register Reference](https://www.oxyron.de/html/registers_sid.html)
- [reSID GitHub](https://github.com/libsidplayfp/resid)
- [resid-rs Rust Port](https://github.com/binaryfields/resid-rs)

### CIA
- [CIA 6526 Software Model](https://ist.uwaterloo.ca/~schepers/MJK/cia6526.html)
- [CIA Register Reference - Oxyron](https://www.oxyron.de/html/registers_cia.html)
- [C64 OS Keyboard Scanning](https://c64os.com/post/howthekeyboardworks)

### Memory Banking
- [Bank Switching - C64-Wiki](https://www.c64-wiki.com/wiki/Bank_Switching)
- [C64 Memory Map - sta.c64.org](https://sta.c64.org/cbm64mem.html)
- [Ultimate C64 Reference](https://www.pagetable.com/c64ref/c64mem/)

### 1541 Disk Drive
- [.D64 File Format](https://vice-emu.sourceforge.io/vice_17.html#SEC349)
- [IEC Serial Bus Protocol](https://www.pagetable.com/?p=1135)
