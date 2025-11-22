# Contract: Rust WASM to JavaScript Interface

**Feature**: C64 Emulator Demo **Component Boundary**: Rust/WASM backend ↔
JavaScript frontend **Date**: 2025-11-20

This contract defines the interface between the Rust WASM emulator and
JavaScript display/input layer.

---

## Overview

The JavaScript frontend communicates with the Rust WASM backend through
`wasm-bindgen` exported functions. The Rust side exposes a `C64Emulator` struct
with methods for initialization, execution control, state queries, and input
handling.

---

## WASM Module Export

**Module Name**: `lib6502` **Build Command**:
`wasm-pack build --target web --features wasm` **Output**: `pkg/lib6502.js`,
`pkg/lib6502_bg.wasm`

**JavaScript Import**:

```javascript
import init, { C64Emulator } from "./pkg/lib6502.js";

// Initialize WASM module
await init();

// Create emulator instance
const emulator = new C64Emulator();
```

---

## C64Emulator Interface

### Constructor

#### `new C64Emulator()`

**Rust Signature**:

```rust
#[wasm_bindgen(constructor)]
pub fn new() -> Result<C64Emulator, JsValue>
```

**JavaScript Usage**:

```javascript
const emulator = new C64Emulator();
```

**Returns**: Emulator instance **Throws**: `Error` if initialization fails
(e.g., out of memory)

**Behavior**:

- Creates empty `MappedMemory` instance
- Initializes CPU with reset vector at $FFFC
- **Does NOT load ROMs** (call `load_roms()` separately)

---

### Initialization Methods

#### `load_roms(basic_rom, kernal_rom, chargen_rom)`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn load_roms(
    &mut self,
    basic_rom: &[u8],
    kernal_rom: &[u8],
    chargen_rom: &[u8]
) -> Result<(), JsValue>
```

**JavaScript Usage**:

```javascript
const basicData = await fetch("roms/basic.bin").then((r) => r.arrayBuffer());
const kernalData = await fetch("roms/kernal.bin").then((r) => r.arrayBuffer());
const chargenData = await fetch("roms/chargen.bin").then((r) =>
  r.arrayBuffer(),
);

emulator.load_roms(
  new Uint8Array(basicData),
  new Uint8Array(kernalData),
  new Uint8Array(chargenData),
);
```

**Parameters**:

- `basic_rom`: `Uint8Array`, 8192 bytes (BASIC ROM, loaded at $A000-$BFFF)
- `kernal_rom`: `Uint8Array`, 8192 bytes (KERNAL ROM, loaded at $E000-$FFFF)
- `chargen_rom`: `Uint8Array`, 4096 bytes (Character ROM, accessible via VIC-II)

**Returns**: `void` **Throws**: `Error` if ROM sizes incorrect or device
registration fails

**Behavior**:

- Validates ROM sizes (8KB, 8KB, 4KB)
- Registers devices in `MappedMemory`:
  - `RamDevice` (64KB) at $0000
  - `RomDevice` (BASIC) at $A000
  - `RomDevice` (KERNAL) at $E000
  - `RomDevice` (CHARGEN) at VIC-II character memory
  - `Vic2Device` at $D000
  - `CiaDevice` at $DC00
  - Color RAM at $D800
- Performs CPU reset (jumps to KERNAL entry point)

---

#### `reset()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn reset(&mut self)
```

**JavaScript Usage**:

```javascript
emulator.reset(); // Restart C64
```

**Returns**: `void`

**Behavior**:

- Triggers CPU reset sequence
- Reads reset vector from $FFFC/$FFFD
- Sets PC to KERNAL entry point
- Clears CPU registers and flags
- **Does NOT** clear RAM (authentic C64 behavior)
- KERNAL will reinitialize VIC-II, clear screen, display "READY."

---

### Execution Control Methods

#### `run_frame()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn run_frame(&mut self) -> Result<u32, JsValue>
```

**JavaScript Usage**:

```javascript
const cyclesExecuted = emulator.run_frame();
```

**Returns**: `number` - Actual CPU cycles executed (typically ~16667 for 60Hz)
**Throws**: `Error` if unimplemented opcode encountered

**Behavior**:

- Executes CPU for one frame's worth of cycles
- Target: 16667 cycles (≈1 MHz at 60 FPS)
- Handles interrupts (CIA Timer A, VIC-II raster if enabled)
- Returns actual cycle count executed

**Timing**:

- NTSC: 16667 cycles/frame @ 60 Hz ≈ 1.0 MHz
- PAL (deferred): 20000 cycles/frame @ 50 Hz ≈ 1.0 MHz

---

#### `step()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn step(&mut self) -> Result<u32, JsValue>
```

**JavaScript Usage**:

```javascript
const cycles = emulator.step(); // Execute one instruction
```

**Returns**: `number` - CPU cycles consumed by instruction (2-7 typically)
**Throws**: `Error` if unimplemented opcode encountered

**Behavior**:

- Executes exactly one 6502 instruction
- Useful for debugging/single-stepping
- Returns cycle cost of executed instruction

---

### State Query Methods

#### `get_screen_memory()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn get_screen_memory(&self) -> Vec<u8>
```

**JavaScript Usage**:

```javascript
const screenData = emulator.get_screen_memory(); // Uint8Array[1000]
```

**Returns**: `Uint8Array`, 1000 bytes **Array Layout**: Row-major, 40 columns ×
25 rows **Value Range**: 0-255 (PETSCII character codes)

**Behavior**:

- Reads from screen RAM address (typically $0400-$07E7)
- Screen base address determined by VIC-II register $D018
- Returns snapshot of current screen content

**Index Calculation**:

```javascript
const charAtXY = screenData[y * 40 + x]; // x ∈ [0,39], y ∈ [0,24]
```

---

#### `get_color_memory()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn get_color_memory(&self) -> Vec<u8>
```

**JavaScript Usage**:

```javascript
const colorData = emulator.get_color_memory(); // Uint8Array[1000]
```

**Returns**: `Uint8Array`, 1000 bytes **Array Layout**: Parallel to screen
memory (same index = same cell) **Value Range**: 0-15 (C64 color palette
indices)

**Behavior**:

- Reads from color RAM at $D800-$DBFF (hardware-fixed address)
- Lower 4 bits per byte hold color value (upper 4 bits unused)
- Returns snapshot of current color attributes

---

#### `get_border_color()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn get_border_color(&self) -> u8
```

**JavaScript Usage**:

```javascript
const borderColor = emulator.get_border_color(); // 0-15
```

**Returns**: `number`, 0-15 (palette index)

**Behavior**:

- Reads VIC-II register $D020
- Returns lower 4 bits (color value)

---

#### `get_background_color()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn get_background_color(&self) -> u8
```

**JavaScript Usage**:

```javascript
const bgColor = emulator.get_background_color(); // 0-15
```

**Returns**: `number`, 0-15 (palette index)

**Behavior**:

- Reads VIC-II register $D021
- Returns lower 4 bits (color value)

---

### Keyboard Input Methods

#### `key_down(row, col)`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn key_down(&mut self, row: u8, col: u8)
```

**JavaScript Usage**:

```javascript
emulator.key_down(1, 0); // Press RETURN key (row 1, col 0)
```

**Parameters**:

- `row`: `number`, 0-7 (keyboard matrix row)
- `col`: `number`, 0-7 (keyboard matrix column)

**Returns**: `void`

**Behavior**:

- Sets keyboard matrix bit to 0 (active low)
- CIA $DC01 reads will reflect pressed key
- Multiple keys can be pressed simultaneously

**Validation**:

- Row/col values > 7 are ignored (silent failure)

---

#### `key_up(row, col)`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn key_up(&mut self, row: u8, col: u8)
```

**JavaScript Usage**:

```javascript
emulator.key_up(1, 0); // Release RETURN key
```

**Parameters**:

- `row`: `number`, 0-7
- `col`: `number`, 0-7

**Returns**: `void`

**Behavior**:

- Sets keyboard matrix bit to 1 (not pressed)
- CIA $DC01 reads will reflect released key

---

#### `trigger_restore()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn trigger_restore(&mut self)
```

**JavaScript Usage**:

```javascript
emulator.trigger_restore(); // Generate NMI (RESTORE key)
```

**Returns**: `void`

**Behavior**:

- Generates CPU NMI (Non-Maskable Interrupt)
- RESTORE key is NOT in keyboard matrix
- KERNAL NMI handler typically does nothing unless explicitly programmed

---

### Debug Methods (Optional for Phase 1)

#### `get_pc()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn get_pc(&self) -> u16
```

**Returns**: `number` - Current program counter

---

#### `get_a()`, `get_x()`, `get_y()`

**Rust Signature**:

```rust
#[wasm_bindgen]
pub fn get_a(&self) -> u8;
pub fn get_x(&self) -> u8;
pub fn get_y(&self) -> u8;
```

**Returns**: `number` - Current register values

---

## Error Handling

**Error Types**:

1. **Initialization Errors**: ROM size mismatch, device overlap
2. **Execution Errors**: Unimplemented opcode, illegal instruction
3. **Memory Errors**: Out of bounds (should never occur with proper device
   setup)

**JavaScript Pattern**:

```javascript
try {
  emulator.load_roms(basic, kernal, chargen);
  emulator.run_frame();
} catch (error) {
  console.error("Emulator error:", error.message);
  // Display error to user, stop emulation
}
```

---

## Performance Considerations

**Memory Copies**:

- `get_screen_memory()` and `get_color_memory()` copy data from WASM memory to
  JavaScript
- Cost: ~2KB per frame (1000 bytes × 2)
- Optimize: Only call when screen changes detected

**Cycle Budget**:

- `run_frame()` executes 16667 cycles ≈ 1-2ms on modern CPUs
- Target frame time: 16.67ms (60 FPS)
- Leaves ~14ms for JavaScript rendering and input handling

**Optimization Strategy**:

```javascript
let lastScreenHash = 0;

function emulationLoop() {
  emulator.run_frame();

  // Check if screen changed (simple hash comparison)
  const currentHash = computeScreenHash();
  if (currentHash !== lastScreenHash) {
    const screenData = emulator.get_screen_memory();
    const colorData = emulator.get_color_memory();
    updateDisplay(screenData, colorData);
    lastScreenHash = currentHash;
  }

  requestAnimationFrame(emulationLoop);
}
```

---

## Type Mappings

| Rust Type            | JavaScript Type       | Notes                            |
| -------------------- | --------------------- | -------------------------------- |
| `u8`                 | `number`              | 0-255                            |
| `u16`                | `number`              | 0-65535                          |
| `u32`                | `number`              | 0-4294967295                     |
| `&[u8]`              | `Uint8Array`          | Read-only byte array             |
| `Vec<u8>`            | `Uint8Array`          | Owned byte array                 |
| `Result<T, JsValue>` | `T` or throws `Error` | Rust errors become JS exceptions |

---

## Example: Complete Initialization

```javascript
import init, { C64Emulator } from "./pkg/lib6502.js";

async function initializeC64() {
  // 1. Load WASM module
  await init();

  // 2. Create emulator
  const emulator = new C64Emulator();

  // 3. Fetch ROMs
  const [basicData, kernalData, chargenData] = await Promise.all([
    fetch("roms/basic.bin").then((r) => r.arrayBuffer()),
    fetch("roms/kernal.bin").then((r) => r.arrayBuffer()),
    fetch("roms/chargen.bin").then((r) => r.arrayBuffer()),
  ]);

  // 4. Load ROMs (triggers reset)
  emulator.load_roms(
    new Uint8Array(basicData),
    new Uint8Array(kernalData),
    new Uint8Array(chargenData),
  );

  // 5. Emulator is ready
  return emulator;
}

// Usage
const emulator = await initializeC64();
emulator.run_frame(); // Execute first frame
```

---

## Contract Validation

**JavaScript Side Must**:

- Call `load_roms()` before `run_frame()` or `step()`
- Provide correctly-sized ROM binaries (8KB, 8KB, 4KB)
- Handle thrown errors gracefully
- Call `key_up()` for every `key_down()` to prevent stuck keys

**Rust Side Must**:

- Return screen/color memory in consistent 1000-byte format
- Enforce row-major layout for screen data
- Return color values in range 0-15 (mask upper 4 bits)
- Execute exactly requested cycle budget in `run_frame()`

---

**Contract Status**: ✅ Complete **Next**: VIC-II Device contract
