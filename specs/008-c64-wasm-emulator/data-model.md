# Data Model: Commodore 64 WASM Emulator

**Date**: 2025-01-22
**Feature Branch**: `008-c64-wasm-emulator`

## Overview

This document defines the core entities, their relationships, and state representations for the C64 emulator. All entities follow the lib6502 Device trait pattern for memory-mapped access.

---

## Core Entities

### C64System

The top-level emulator container orchestrating all components.

```rust
pub struct C64System {
    // CPU
    cpu: CPU<C64Memory>,

    // Timing
    region: Region,             // PAL or NTSC
    cycles_per_frame: u32,      // 19656 (PAL) or 17095 (NTSC)
    current_scanline: u16,
    cycle_in_scanline: u16,

    // State
    running: bool,
    frame_count: u64,
}

pub enum Region {
    PAL,   // 985248 Hz, 50 Hz, 312 scanlines, 63 cycles/line
    NTSC,  // 1022727 Hz, 60 Hz, 263 scanlines, 65 cycles/line
}
```

**Relationships**:
- Contains CPU with C64Memory
- C64Memory contains all hardware devices

**State Transitions**:
- `Idle` → `Running` (on start)
- `Running` → `Paused` (on pause/focus loss)
- `Paused` → `Running` (on resume)
- Any → `Reset` → `Idle` (on reset)

---

### C64Memory

Custom memory bus implementing C64 banking logic.

```rust
pub struct C64Memory {
    // RAM (always present)
    ram: Box<[u8; 65536]>,

    // ROMs
    basic_rom: Box<[u8; 8192]>,      // 8KB at $A000-$BFFF
    kernal_rom: Box<[u8; 8192]>,     // 8KB at $E000-$FFFF
    char_rom: Box<[u8; 4096]>,       // 4KB at $D000-$DFFF (when visible)

    // 6510 I/O Port
    port: Port6510,

    // Hardware Devices
    vic: VicII,
    sid: Sid6581,
    cia1: Cia6526,
    cia2: Cia6526,
    color_ram: [u8; 1024],           // 4-bit color RAM at $D800

    // VIC-II Bank Selection (from CIA2)
    vic_bank: u8,                    // 0-3
}
```

**Invariants**:
- RAM is always fully addressable (underlying ROMs)
- port.data bits 0-2 determine banking configuration
- VIC-II bank updated on CIA2 port A writes

---

### Port6510

The 6510 processor's built-in I/O port.

```rust
pub struct Port6510 {
    ddr: u8,         // $00 - Data Direction Register
    data: u8,        // $01 - Data Register
}
```

**Field Constraints**:
- `ddr`: Any u8 (0=input, 1=output per bit)
- `data`: Any u8, but bits 0-2 control banking:
  - Bit 0: LORAM (1=BASIC ROM visible)
  - Bit 1: HIRAM (1=KERNAL ROM visible)
  - Bit 2: CHAREN (0=CHAR ROM, 1=I/O)

**Default Values**:
- `ddr`: 0x2F (bits 0-2,5 as outputs)
- `data`: 0x37 (BASIC + KERNAL + I/O visible)

---

### VicII

MOS 6569 (PAL) / 6567 (NTSC) Video Interface Chip.

```rust
pub struct VicII {
    // Registers ($D000-$D02E = 47 registers)
    registers: [u8; 47],

    // Internal State
    current_raster: u16,           // 0-311 (PAL) or 0-262 (NTSC)
    cycle_in_line: u8,             // 0-62 (PAL) or 0-64 (NTSC)

    // Sprite State
    sprite_data_pointers: [u8; 8],
    sprite_collision_ss: u8,       // Sprite-sprite collision flags
    sprite_collision_sb: u8,       // Sprite-background collision flags

    // Output
    framebuffer: Box<[[u8; 320]; 200]>,  // Indexed color (0-15)

    // Interrupt
    irq_pending: bool,
}
```

**Register Map Summary**:

| Offset | Name | Description |
|--------|------|-------------|
| $00-$0F | SPRn X/Y | Sprite 0-7 coordinates |
| $10 | MSIGX | Sprite X MSBs (bit 8) |
| $11 | CR1 | Control: Y-scroll, DEN, BMM, ECM, RST8 |
| $12 | RASTER | Raster counter / compare |
| $15 | SPENA | Sprite enable bits |
| $16 | CR2 | Control: X-scroll, CSEL, MCM |
| $17 | YEXP | Sprite Y expansion |
| $18 | MEMPTR | Memory pointers |
| $19 | IRR | Interrupt request register |
| $1A | IMR | Interrupt mask register |
| $1B | SPBGPR | Sprite-background priority |
| $1C | SPMC | Sprite multicolor mode |
| $1D | XEXP | Sprite X expansion |
| $1E | SSCOL | Sprite-sprite collision |
| $1F | SBCOL | Sprite-background collision |
| $20 | EXTCOL | Border color |
| $21-$24 | BGCOLn | Background colors 0-3 |
| $25-$26 | SPMCn | Sprite multicolor colors |
| $27-$2E | SPCOLn | Sprite colors 0-7 |

**Invariants**:
- Collision registers ($1E, $1F) cleared on read
- IRR bits cleared by writing 1
- RASTER updates each scanline

---

### Sid6581

MOS 6581 Sound Interface Device.

```rust
pub struct Sid6581 {
    // Registers ($D400-$D41C = 29 bytes)
    voices: [SidVoice; 3],
    filter: SidFilter,
    volume: u8,                    // $D418 bits 0-3

    // Audio Output
    sample_buffer: Vec<f32>,       // For Web Audio consumption
    cycles_per_sample: f32,        // ~22.35 for 44.1kHz
    sample_accumulator: f32,
}

pub struct SidVoice {
    // Registers
    freq: u16,                     // 16-bit frequency
    pulse_width: u16,              // 12-bit pulse width
    control: u8,                   // Waveform select, gate, sync, ring
    attack_decay: u8,              // 4-bit each
    sustain_release: u8,           // 4-bit each

    // Internal State
    accumulator: u32,              // 24-bit phase accumulator
    lfsr: u32,                     // 23-bit noise LFSR
    envelope: SidEnvelope,
}

pub struct SidEnvelope {
    state: EnvelopeState,          // Attack/Decay/Sustain/Release
    counter: u8,                   // Current envelope value (0-255)
    rate_counter: u16,             // Rate period countdown
    exponential_counter: u8,       // Exponential decay divider
}

pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct SidFilter {
    cutoff: u16,                   // 11-bit cutoff frequency
    resonance: u8,                 // 4-bit resonance
    routing: u8,                   // Which voices through filter
    mode: FilterMode,              // LP/BP/HP/Notch

    // State variables (biquad filter)
    low: f32,
    band: f32,
}

pub enum FilterMode {
    LowPass,
    BandPass,
    HighPass,
    Notch,
}
```

**Register Map**:

| Offset | Voice | Description |
|--------|-------|-------------|
| $00-$06 | 1 | Freq, PW, Control, AD, SR |
| $07-$0D | 2 | Freq, PW, Control, AD, SR |
| $0E-$14 | 3 | Freq, PW, Control, AD, SR |
| $15-$16 | - | Filter cutoff (11-bit) |
| $17 | - | Filter resonance + routing |
| $18 | - | Filter mode + volume |
| $19-$1A | - | Paddle inputs (read-only) |
| $1B | - | Voice 3 oscillator (read-only) |
| $1C | - | Voice 3 envelope (read-only) |

---

### Cia6526

MOS 6526 Complex Interface Adapter.

```rust
pub struct Cia6526 {
    // Ports
    port_a: CiaPort,
    port_b: CiaPort,

    // Timers
    timer_a: CiaTimer,
    timer_b: CiaTimer,

    // Time of Day
    tod: TodClock,

    // Serial
    sdr: u8,

    // Interrupts
    interrupt_flags: u8,           // ICR data
    interrupt_mask: u8,            // ICR mask
    irq_pending: bool,

    // Control
    cra: u8,
    crb: u8,

    // Device-specific (for CIA1)
    keyboard_matrix: [[bool; 8]; 8],  // 8x8 key state
    joystick1: u8,                    // Port B overlay
    joystick2: u8,                    // Port A overlay
}

pub struct CiaPort {
    data: u8,                      // Output latch
    ddr: u8,                       // Data direction
    external_input: u8,            // External signals
}

pub struct CiaTimer {
    counter: u16,                  // Current value
    latch: u16,                    // Reload value
    running: bool,
    one_shot: bool,
}

pub struct TodClock {
    tenths: u8,                    // BCD 0-9
    seconds: u8,                   // BCD 00-59
    minutes: u8,                   // BCD 00-59
    hours: u8,                     // BCD 01-12, bit 7 = PM
    alarm_tenths: u8,
    alarm_seconds: u8,
    alarm_minutes: u8,
    alarm_hours: u8,
    stopped: bool,
    latched: bool,
}
```

**CIA1 vs CIA2 Differences**:

| Feature | CIA1 ($DC00) | CIA2 ($DD00) |
|---------|--------------|--------------|
| IRQ Type | IRQ | NMI |
| Port A | Keyboard cols / Joy2 | IEC bus + VIC bank |
| Port B | Keyboard rows / Joy1 | User port |

---

### Drive1541

Commodore 1541 Disk Drive (high-level emulation).

```rust
pub struct Drive1541 {
    mounted_image: Option<D64Image>,
    device_number: u8,             // Usually 8
    channels: [DriveChannel; 16],
    status: DriveStatus,
    error_channel: String,         // Channel 15 status
}

pub struct D64Image {
    data: Box<[u8; 174848]>,       // 35 tracks, 683 sectors
    modified: bool,
}

pub struct DriveChannel {
    active: bool,
    mode: ChannelMode,
    track: u8,
    sector: u8,
    buffer: [u8; 256],
    buffer_position: u8,
}

pub enum ChannelMode {
    Closed,
    Read,
    Write,
    Command,
}

pub struct DriveStatus {
    error_number: u8,
    track: u8,
    sector: u8,
}
```

**D64 Track Layout**:

| Tracks | Sectors/Track | Total Sectors |
|--------|---------------|---------------|
| 1-17 | 21 | 357 |
| 18-24 | 19 | 133 |
| 25-30 | 18 | 108 |
| 31-35 | 17 | 85 |
| **Total** | - | **683** |

Track 18 is reserved for directory and BAM.

---

### SaveState

Complete emulator state for save/load.

```rust
pub struct SaveState {
    version: u32,                  // Format version
    timestamp: u64,                // Unix timestamp

    // CPU State
    cpu_a: u8,
    cpu_x: u8,
    cpu_y: u8,
    cpu_sp: u8,
    cpu_pc: u16,
    cpu_flags: u8,
    cpu_cycles: u64,

    // Memory
    ram: Box<[u8; 65536]>,
    port_ddr: u8,
    port_data: u8,

    // VIC-II
    vic_registers: [u8; 47],
    vic_raster: u16,
    vic_collision_ss: u8,
    vic_collision_sb: u8,

    // SID
    sid_registers: [u8; 29],
    sid_voice_states: [SidVoiceState; 3],

    // CIAs
    cia1_state: CiaState,
    cia2_state: CiaState,
    color_ram: [u8; 1024],

    // Disk (reference only)
    mounted_d64_hash: Option<[u8; 32]>,
}
```

**Version Compatibility**:
- Version 1: Initial format
- Breaking changes increment version
- Older saves can be rejected with clear error

---

## Keyboard Mapping

### C64 Key Matrix (8x8)

| Row\Col | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---------|---|---|---|---|---|---|---|---|
| **0** | DEL | RET | → | F7 | F1 | F3 | F5 | ↓ |
| **1** | 3 | W | A | 4 | Z | S | E | LSHFT |
| **2** | 5 | R | D | 6 | C | F | T | X |
| **3** | 7 | Y | G | 8 | B | H | U | V |
| **4** | 9 | I | J | 0 | M | K | O | N |
| **5** | + | P | L | - | . | : | @ | , |
| **6** | £ | * | ; | HOME | RSHFT | = | ↑ | / |
| **7** | 1 | ← | CTRL | 2 | SPACE | C= | Q | STOP |

### PC to C64 Mapping

```rust
pub struct KeyMapping {
    pc_keycode: String,    // e.g., "KeyA", "Digit1"
    c64_row: u8,           // 0-7
    c64_col: u8,           // 0-7
    requires_shift: bool,  // Need SHIFT modifier
}
```

---

## Color Palette

### C64 16-Color Palette

| Index | Name | RGB (VICE) |
|-------|------|------------|
| 0 | Black | #000000 |
| 1 | White | #FFFFFF |
| 2 | Red | #68372B |
| 3 | Cyan | #70A4B2 |
| 4 | Purple | #6F3D86 |
| 5 | Green | #588D43 |
| 6 | Blue | #352879 |
| 7 | Yellow | #B8C76F |
| 8 | Orange | #6F4F25 |
| 9 | Brown | #433900 |
| 10 | Light Red | #9A6759 |
| 11 | Dark Grey | #444444 |
| 12 | Grey | #6C6C6C |
| 13 | Light Green | #9AD284 |
| 14 | Light Blue | #6C5EB5 |
| 15 | Light Grey | #959595 |

---

## Validation Rules

### ROM Validation (FR-106)

| ROM | Expected Size | SHA-256 (optional) |
|-----|---------------|--------------------|
| BASIC | 8192 bytes | - |
| KERNAL | 8192 bytes | - |
| CHARROM | 4096 bytes | - |

### D64 Validation

- File size: 174,848 bytes (standard) or 175,531 bytes (with error info)
- Track 18 must contain valid directory structure
- BAM at track 18, sector 0

### PRG Validation

- Minimum size: 3 bytes (2-byte load address + 1 data)
- Load address in first 2 bytes (little-endian)
- Valid load address range: $0801-$9FFF (typical)
