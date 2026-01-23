# WASM API Contract: C64 Emulator

**Version**: 1.0.0
**Date**: 2025-01-22

## Overview

This document defines the WebAssembly API exposed by the C64 emulator for JavaScript integration. The API follows the existing lib6502 WASM bindings pattern using `wasm-bindgen`.

---

## C64Emulator Class

### Constructor

```typescript
class C64Emulator {
    /**
     * Create a new C64 emulator instance.
     * ROMs must be loaded before emulation can begin.
     */
    constructor(): C64Emulator;
}
```

---

## ROM Management

### load_roms

```typescript
/**
 * Load C64 ROM files into the emulator.
 * Must be called before any emulation methods.
 *
 * @param basic - 8192-byte BASIC ROM ($A000-$BFFF)
 * @param kernal - 8192-byte KERNAL ROM ($E000-$FFFF)
 * @param charrom - 4096-byte Character ROM
 * @returns true if ROMs loaded successfully, false on validation error
 * @throws Error with specific message if ROM sizes are invalid
 */
load_roms(basic: Uint8Array, kernal: Uint8Array, charrom: Uint8Array): boolean;
```

**Validation Rules**:
- `basic.length === 8192` (8KB)
- `kernal.length === 8192` (8KB)
- `charrom.length === 4096` (4KB)

**Error Messages**:
- `"Invalid BASIC ROM size: expected 8192 bytes, got {n}"`
- `"Invalid KERNAL ROM size: expected 8192 bytes, got {n}"`
- `"Invalid Character ROM size: expected 4096 bytes, got {n}"`

### roms_loaded

```typescript
/**
 * Check if ROMs have been loaded.
 * @returns true if all required ROMs are present
 */
roms_loaded(): boolean;
```

---

## Emulation Control

### step_frame

```typescript
/**
 * Execute one complete video frame.
 * PAL: 312 scanlines × 63 cycles = 19656 cycles
 * NTSC: 263 scanlines × 65 cycles = 17095 cycles
 *
 * Updates framebuffer and generates audio samples.
 * Should be called at 50Hz (PAL) or 60Hz (NTSC).
 */
step_frame(): void;
```

### step_scanline

```typescript
/**
 * Execute one scanline worth of cycles.
 * PAL: 63 cycles, NTSC: 65 cycles
 *
 * Useful for debugging raster effects.
 * @returns Current raster line after execution
 */
step_scanline(): number;
```

### reset

```typescript
/**
 * Perform hardware reset.
 * Resets CPU, VIC-II, SID, CIAs to power-on state.
 * RAM contents are preserved (warm reset).
 */
reset(): void;
```

### hard_reset

```typescript
/**
 * Perform power cycle reset.
 * Resets all state including RAM contents.
 * Simulates power off/on cycle.
 */
hard_reset(): void;
```

### set_region

```typescript
/**
 * Set video region (PAL or NTSC).
 * Affects timing, frame rate, and scanline count.
 *
 * @param region - 0 for PAL (50Hz), 1 for NTSC (60Hz)
 */
set_region(region: number): void;
```

---

## Display

### get_framebuffer_ptr

```typescript
/**
 * Get pointer to VIC-II framebuffer in WASM memory.
 * Buffer is 320×200 bytes, indexed color (0-15).
 *
 * @returns Pointer to framebuffer start
 */
get_framebuffer_ptr(): number;
```

**Usage from JavaScript**:
```javascript
const ptr = emulator.get_framebuffer_ptr();
const framebuffer = new Uint8Array(wasm.memory.buffer, ptr, 320 * 200);
```

### get_framebuffer_size

```typescript
/**
 * Get framebuffer dimensions.
 * @returns { width: 320, height: 200 }
 */
get_framebuffer_size(): { width: number; height: number };
```

### get_border_color

```typescript
/**
 * Get current VIC-II border color.
 * @returns Color index 0-15
 */
get_border_color(): number;
```

### get_current_raster

```typescript
/**
 * Get current VIC-II raster line.
 * @returns Raster line 0-311 (PAL) or 0-262 (NTSC)
 */
get_current_raster(): number;
```

---

## Audio

### get_audio_samples

```typescript
/**
 * Get generated audio samples since last call.
 * Samples are 32-bit float, mono, range [-1.0, 1.0].
 * Call at regular intervals to avoid buffer overflow.
 *
 * @param max_samples - Maximum samples to retrieve
 * @returns Float32Array of audio samples
 */
get_audio_samples(max_samples: number): Float32Array;
```

**Recommended Usage**:
- Call every ~3ms (128 samples at 44.1kHz)
- Use with AudioWorklet for low-latency playback

### set_sample_rate

```typescript
/**
 * Set audio output sample rate.
 * Affects internal resampling ratio.
 *
 * @param rate - Sample rate in Hz (typically 44100 or 48000)
 */
set_sample_rate(rate: number): void;
```

### set_audio_enabled

```typescript
/**
 * Enable or disable audio generation.
 * Disabling saves CPU when audio is muted.
 *
 * @param enabled - true to enable audio
 */
set_audio_enabled(enabled: boolean): void;
```

---

## Input - Keyboard

### key_down

```typescript
/**
 * Signal key press.
 * Uses C64 matrix position (row, column).
 *
 * @param row - Matrix row 0-7
 * @param col - Matrix column 0-7
 */
key_down(row: number, col: number): void;
```

### key_up

```typescript
/**
 * Signal key release.
 *
 * @param row - Matrix row 0-7
 * @param col - Matrix column 0-7
 */
key_up(row: number, col: number): void;
```

### key_down_pc

```typescript
/**
 * Signal key press using PC keycode.
 * Automatically maps to C64 matrix position.
 *
 * @param keycode - DOM KeyboardEvent.code value
 */
key_down_pc(keycode: string): void;
```

### key_up_pc

```typescript
/**
 * Signal key release using PC keycode.
 *
 * @param keycode - DOM KeyboardEvent.code value
 */
key_up_pc(keycode: string): void;
```

### restore_key

```typescript
/**
 * Trigger RESTORE key (NMI).
 * Unlike normal keys, RESTORE triggers non-maskable interrupt.
 */
restore_key(): void;
```

---

## Input - Joystick

### set_joystick

```typescript
/**
 * Set joystick state.
 * Bits: 0=up, 1=down, 2=left, 3=right, 4=fire (active high)
 *
 * @param port - 1 or 2
 * @param state - Bitmask of active directions/fire
 */
set_joystick(port: number, state: number): void;
```

**Bit Definitions**:
```typescript
const JOY_UP    = 0x01;
const JOY_DOWN  = 0x02;
const JOY_LEFT  = 0x04;
const JOY_RIGHT = 0x08;
const JOY_FIRE  = 0x10;
```

---

## File Loading

### mount_d64

```typescript
/**
 * Mount a D64 disk image in virtual drive 8.
 *
 * @param data - Complete D64 file contents
 * @returns true if mounted successfully
 */
mount_d64(data: Uint8Array): boolean;
```

**Validation**:
- Size must be 174848 (standard) or 175531 (with error info)

### unmount_d64

```typescript
/**
 * Unmount current disk image.
 */
unmount_d64(): void;
```

### has_mounted_disk

```typescript
/**
 * Check if a disk is mounted.
 * @returns true if disk is mounted
 */
has_mounted_disk(): boolean;
```

### load_prg

```typescript
/**
 * Load PRG file directly into memory.
 * First two bytes are load address (little-endian).
 *
 * @param data - Complete PRG file contents
 * @returns Load address, or 0 on error
 */
load_prg(data: Uint8Array): number;
```

### inject_basic_run

```typescript
/**
 * Inject "RUN" command into keyboard buffer.
 * Useful after loading a BASIC program.
 */
inject_basic_run(): void;
```

---

## State Management

### save_state

```typescript
/**
 * Capture complete emulator state.
 * Includes CPU, RAM, all chip states.
 * Does NOT include mounted disk image (only hash reference).
 *
 * @returns Serialized state as Uint8Array
 */
save_state(): Uint8Array;
```

### load_state

```typescript
/**
 * Restore emulator from saved state.
 *
 * @param data - Previously saved state
 * @returns true if restored successfully
 */
load_state(data: Uint8Array): boolean;
```

### get_state_size

```typescript
/**
 * Get size of save state.
 * Useful for UI display.
 *
 * @returns Size in bytes
 */
get_state_size(): number;
```

---

## Memory Access (Debug)

### read_memory

```typescript
/**
 * Read byte from CPU's memory view.
 * Respects current banking configuration.
 *
 * @param address - 16-bit address
 * @returns Byte value
 */
read_memory(address: number): number;
```

### write_memory

```typescript
/**
 * Write byte to CPU's memory view.
 * Respects current banking configuration.
 *
 * @param address - 16-bit address
 * @param value - Byte value
 */
write_memory(address: number, value: number): void;
```

### read_ram

```typescript
/**
 * Read byte directly from RAM (ignoring ROMs).
 *
 * @param address - 16-bit address
 * @returns Byte value
 */
read_ram(address: number): number;
```

### get_memory_page

```typescript
/**
 * Get 256-byte memory page.
 * Useful for memory viewer UI.
 *
 * @param page - Page number 0-255
 * @returns 256 bytes as Uint8Array
 */
get_memory_page(page: number): Uint8Array;
```

---

## CPU State (Debug)

### get_cpu_state

```typescript
/**
 * Get current CPU register state.
 */
get_cpu_state(): {
    a: number;      // Accumulator
    x: number;      // X index
    y: number;      // Y index
    sp: number;     // Stack pointer
    pc: number;     // Program counter
    flags: number;  // Status register (NV-BDIZC)
    cycles: bigint; // Total cycles executed
};
```

### get_bank_config

```typescript
/**
 * Get current memory banking configuration.
 *
 * @returns {
 *   loram: boolean,   // BASIC ROM visible
 *   hiram: boolean,   // KERNAL ROM visible
 *   charen: boolean,  // I/O (true) or CHAR ROM (false)
 *   vic_bank: number  // VIC-II bank 0-3
 * }
 */
get_bank_config(): {
    loram: boolean;
    hiram: boolean;
    charen: boolean;
    vic_bank: number;
};
```

---

## VIC-II State (Debug)

### get_vic_registers

```typescript
/**
 * Get all VIC-II registers.
 * @returns 47 bytes as Uint8Array
 */
get_vic_registers(): Uint8Array;
```

### read_vic_register

```typescript
/**
 * Read single VIC-II register.
 * @param offset - Register offset 0-46
 */
read_vic_register(offset: number): number;
```

---

## SID State (Debug)

### get_sid_registers

```typescript
/**
 * Get all SID registers.
 * Note: Write-only registers return last written value.
 * @returns 29 bytes as Uint8Array
 */
get_sid_registers(): Uint8Array;
```

### read_sid_register

```typescript
/**
 * Read single SID register.
 * @param offset - Register offset 0-28
 */
read_sid_register(offset: number): number;
```

---

## CIA State (Debug)

### get_cia1_registers

```typescript
/**
 * Get CIA1 registers.
 * @returns 16 bytes as Uint8Array
 */
get_cia1_registers(): Uint8Array;
```

### get_cia2_registers

```typescript
/**
 * Get CIA2 registers.
 * @returns 16 bytes as Uint8Array
 */
get_cia2_registers(): Uint8Array;
```

---

## Error Handling

All methods may throw `Error` with descriptive messages:

| Error Type | Example Message |
|------------|-----------------|
| ROM Error | "ROMs not loaded" |
| Validation | "Invalid D64 size: expected 174848, got {n}" |
| State | "Invalid save state version: expected 1, got {n}" |
| Range | "Address out of range: {addr}" |

---

## Constants Export

```typescript
// Region constants
const REGION_PAL = 0;
const REGION_NTSC = 1;

// Joystick bit masks
const JOY_UP = 0x01;
const JOY_DOWN = 0x02;
const JOY_LEFT = 0x04;
const JOY_RIGHT = 0x08;
const JOY_FIRE = 0x10;

// Display dimensions
const SCREEN_WIDTH = 320;
const SCREEN_HEIGHT = 200;

// Timing (cycles per second)
const PAL_CLOCK = 985248;
const NTSC_CLOCK = 1022727;

// File sizes
const BASIC_ROM_SIZE = 8192;
const KERNAL_ROM_SIZE = 8192;
const CHARROM_SIZE = 4096;
const D64_SIZE = 174848;
```
