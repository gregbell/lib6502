# Data Model: C64 Emulator Demo

**Feature**: Commodore 64 Emulator Web Demo
**Branch**: `007-c64-emulator-demo`
**Date**: 2025-11-20

This document defines the key data structures and entities for the C64 emulator implementation.

---

## Overview

The C64 emulator consists of three layers:
1. **Rust/WASM Backend**: CPU core + hardware devices (VIC-II, CIA, memory)
2. **JavaScript Bridge**: WASM interface + emulation loop
3. **Frontend Display**: Canvas rendering + keyboard capture

---

## 1. Rust Backend Entities

### 1.1 Vic2Device

**Purpose**: Emulates VIC-II video chip for 40×25 character text mode display.

**State Fields**:
```rust
pub struct Vic2Device {
    // VIC-II Control Registers
    control_register_1: u8,      // $D011: Display enable, 25-row mode, YSCROLL
    control_register_2: u8,      // $D016: 40-column mode, XSCROLL
    memory_pointers: u8,         // $D018: Screen/character memory locations

    // Color Registers
    border_color: u8,            // $D020: Border color (4-bit palette index)
    background_color_0: u8,      // $D021: Main background color
    background_color_1: u8,      // $D022: Extended color (deferred)
    background_color_2: u8,      // $D023: Extended color (deferred)
    background_color_3: u8,      // $D024: Extended color (deferred)

    // Status Registers
    raster_counter: u8,          // $D012: Current raster line (low byte)
    interrupt_status: u8,        // $D019: IRQ flags (stub for Phase 1)
    interrupt_enable: u8,        // $D01A: IRQ mask (stub for Phase 1)

    // Memory References (computed from registers)
    screen_memory_base: u16,     // Computed from $D018 bits 4-7
    character_rom_base: u16,     // Computed from $D018 bits 1-3
}
```

**Key Methods**:
```rust
impl Device for Vic2Device {
    fn read(&self, offset: u16) -> u8;    // Handle register reads ($D000-$D3FF)
    fn write(&mut self, offset: u16, value: u8);  // Handle register writes
    fn size(&self) -> u16 { 0x0400 }      // 1KB address space (mirrored)
}

impl Vic2Device {
    pub fn new() -> Self;                 // Initialize with default values
    pub fn get_screen_base(&self) -> u16; // Return screen RAM address
    pub fn get_char_base(&self) -> u16;   // Return character ROM address
    pub fn border_color(&self) -> u8;     // Get current border color
    pub fn background_color(&self) -> u8; // Get current background color
}
```

**Default Values** (C64 boot state):
- `control_register_1`: `0x1B` (DEN=1, RSEL=1, YSCROLL=3, text mode)
- `control_register_2`: `0xC8` (CSEL=1 for 40 columns, XSCROLL=0)
- `memory_pointers`: `0x15` (screen at $0400, char ROM at $D000)
- `border_color`: `0x0E` (light blue)
- `background_color_0`: `0x06` (blue)

**Register Address Mapping**:
- Registers mirror every 64 bytes in $D000-$D3FF range
- Offset & 0x3F gives actual register index
- Sprite registers ($D000-$D00F, $D010, $D015-$D01F, $D025-$D02E) return 0

---

### 1.2 CiaDevice

**Purpose**: Emulates CIA #1 chip for keyboard scanning and timer interrupts.

**State Fields**:
```rust
pub struct CiaDevice {
    // I/O Ports (keyboard matrix)
    port_a: u8,                  // $DC00: Output (keyboard column select)
    port_b: u8,                  // $DC01: Input (keyboard row read)
    data_direction_a: u8,        // $DC02: DDRA (fixed at 0xFF for keyboard)
    data_direction_b: u8,        // $DC03: DDRB (fixed at 0x00 for keyboard)

    // Timer A (60Hz system interrupt)
    timer_a_latch: u16,          // $DC04-$DC05: Reload value (write)
    timer_a_counter: u16,        // $DC04-$DC05: Current value (read)
    control_register_a: u8,      // $DC0E: Timer A control (start/stop/mode)

    // Timer B (deferred - not used by KERNAL)
    timer_b_latch: u16,          // $DC06-$DC07: Reload value
    timer_b_counter: u16,        // $DC06-$DC07: Current value
    control_register_b: u8,      // $DC0F: Timer B control

    // Interrupt Control
    interrupt_control: u8,       // $DC0D: IRQ mask/status
    interrupt_pending: bool,     // Internal: Timer A underflow flag

    // Keyboard Matrix State (from JavaScript)
    keyboard_matrix: [u8; 8],    // 8 rows × 8 bits each (active low)
}
```

**Key Methods**:
```rust
impl Device for CiaDevice {
    fn read(&self, offset: u16) -> u8;    // Handle register reads
    fn write(&mut self, offset: u16, value: u8);  // Handle register writes
    fn size(&self) -> u16 { 0x0100 }      // 256 bytes address space
}

impl CiaDevice {
    pub fn new() -> Self;                 // Initialize with defaults
    pub fn tick(&mut self, cycles: usize); // Decrement Timer A (Phase 2)
    pub fn set_key_state(&mut self, row: u8, col: u8, pressed: bool);
    pub fn get_key_state(&self, row: u8, col: u8) -> bool;
    pub fn has_interrupt(&self) -> bool;  // Check Timer A underflow
    pub fn acknowledge_interrupt(&mut self); // Clear interrupt flag
}
```

**Keyboard Matrix Encoding**:
- Each row is 8 bits (active low: 0=pressed, 1=released)
- Writing to Port A ($DC00) selects row to scan
- Reading from Port B ($DC01) returns key states for that row
- Example: Write $FE to Port A, read Port B to get row 0 keys

**Timer Operation** (Phase 1 - Functional):
- Accept writes to timer latch ($DC04-$DC05)
- Generate interrupt every ~16667 microseconds (60 Hz)
- Return latch value on reads (no actual countdown initially)
- Phase 2: Implement cycle-accurate decrement

**Default Values**:
- `port_a`: `0xFF` (no keys pressed)
- `data_direction_a`: `0xFF` (all outputs)
- `data_direction_b`: `0x00` (all inputs)
- `keyboard_matrix`: `[0xFF; 8]` (all keys released)
- `timer_a_latch`: `0x4025` (KERNAL default: 16421 cycles)

---

### 1.3 C64Memory (Helper)

**Purpose**: Convenience wrapper for setting up C64 memory map.

**Function**:
```rust
pub struct C64MemoryBuilder {
    ram: Box<RamDevice>,
    basic_rom: Box<RomDevice>,
    kernal_rom: Box<RomDevice>,
    character_rom: Box<RomDevice>,
    vic2: Box<Vic2Device>,
    cia1: Box<CiaDevice>,
}

impl C64MemoryBuilder {
    pub fn new() -> Self;
    pub fn with_basic_rom(mut self, data: Vec<u8>) -> Self;
    pub fn with_kernal_rom(mut self, data: Vec<u8>) -> Self;
    pub fn with_character_rom(mut self, data: Vec<u8>) -> Self;
    pub fn build(self) -> Result<MappedMemory, DeviceError>;
}
```

**C64 Memory Map** (64KB address space):
```
$0000-$00FF   Zero Page RAM
$0100-$01FF   Stack RAM
$0200-$03FF   BASIC/KERNAL working storage RAM
$0400-$07E7   Screen RAM (1000 bytes for 40×25 characters)
$07E8-$9FFF   User RAM
$A000-$BFFF   BASIC ROM (8KB)
$C000-$CFFF   RAM (4KB)
$D000-$D3FF   VIC-II registers (1KB, mirrored)
$D400-$D7FF   SID registers (1KB, stubbed)
$D800-$DBFF   Color RAM (1KB, 1000 bytes used)
$DC00-$DCFF   CIA #1 registers (256 bytes)
$DD00-$DDFF   CIA #2 registers (256 bytes, stubbed)
$DE00-$DFFF   I/O expansion (512 bytes, open bus)
$E000-$FFFF   KERNAL ROM (8KB)
```

**Bank Switching** (Deferred):
- C64 supports banking RAM/ROM via CPU port $00/$01
- Phase 1: Fixed configuration (ROM always visible)
- Phase 2: Implement bank switching for full compatibility

**Color RAM Special Handling**:
- $D800-$DBFF (1KB) stores 4-bit color values per screen cell
- Must be separate from character ROM at $D000-$DFFF
- Typically implemented as dedicated RAM device

---

## 2. JavaScript Bridge Entities

### 2.1 C64Emulator (WASM Wrapper)

**Purpose**: JavaScript interface to Rust emulator instance.

**Properties**:
```javascript
class C64Emulator {
  constructor(onScreenUpdate);

  // WASM instance and memory
  #wasmModule;              // Loaded WASM module
  #wasmMemory;              // WebAssembly.Memory instance
  #emulatorPtr;             // Pointer to Rust emulator struct

  // Callbacks
  #screenUpdateCallback;    // Called when screen RAM changes

  // State
  #running;                 // Boolean: emulation loop active
  #cyclesPerFrame;          // Number of CPU cycles per 60Hz frame
}
```

**Methods**:
```javascript
async init(basicRom, kernalRom, chargenRom);  // Load ROMs, initialize
reset();                                       // Perform C64 reset
runFrame();                                    // Execute one 60Hz frame
start();                                       // Begin emulation loop
stop();                                        // Pause emulation
keyDown(row, col);                             // Press key
keyUp(row, col);                               // Release key
getScreenMemory();                             // Get Uint8Array[1000] characters
getColorMemory();                              // Get Uint8Array[1000] colors
getBorderColor();                              // Get current border color (0-15)
getBackgroundColor();                          // Get current background color (0-15)
```

**Emulation Loop**:
```javascript
function emulationLoop() {
  if (!this.#running) return;

  // Run CPU for one frame (16667 microseconds at ~1 MHz)
  this.runFrame();  // Executes ~16667 CPU cycles

  // Check if screen changed, trigger render
  if (this.hasScreenChanged()) {
    this.#screenUpdateCallback();
  }

  requestAnimationFrame(() => this.emulationLoop());
}
```

---

### 2.2 KeyboardMatrix

**Purpose**: Map modern keyboard events to C64 matrix positions.

**Structure**:
```javascript
class KeyboardMatrix {
  #matrix;                  // Uint8Array[8]: 8 rows of key states
  #keyMap;                  // Map<string, {row, col}>: event.code → position
  #emulator;                // Reference to C64Emulator

  constructor(emulator);

  // Public API
  handleKeyDown(event);     // KeyboardEvent → matrix update
  handleKeyUp(event);       // KeyboardEvent → matrix update

  // Internal
  #mapKeyCode(code);        // event.code → {row, col} or 'RESTORE'
  #pressKey(row, col);      // Set matrix bit to 0 (active low)
  #releaseKey(row, col);    // Set matrix bit to 1
}
```

**Key Mapping Table**:
```javascript
const KEY_MAP = {
  'Enter': { row: 1, col: 0 },
  'Space': { row: 4, col: 0 },
  'Backspace': { row: 0, col: 0 },
  'Escape': { row: 7, col: 7 },    // RUN/STOP
  'KeyA': { row: 2, col: 6 },
  'KeyB': { row: 4, col: 4 },
  // ... (full mapping ~70 keys)
  'PageUp': 'RESTORE',             // Special: generates NMI
};
```

---

## 3. Frontend Display Entities

### 3.1 PetsciiDisplay

**Purpose**: Render 40×25 PETSCII character grid to Canvas.

**Properties**:
```javascript
class PetsciiDisplay {
  #canvas;                  // HTMLCanvasElement
  #ctx;                     // CanvasRenderingContext2D
  #charAtlas;               // HTMLImageElement: pre-rendered character set
  #scale;                   // Display scale factor (1, 2, 4, etc.)

  // Screen state
  #screenMemory;            // Uint8Array[1000]: character codes
  #colorMemory;             // Uint8Array[1000]: color values
  #borderColor;             // 0-15: current border color
  #backgroundColor;         // 0-15: current background color

  // Optimization
  #dirtyRegions;            // Set<string>: "x,y" coordinates of changed cells
  #offscreenCanvas;         // OffscreenCanvas for double buffering
}
```

**Methods**:
```javascript
constructor(canvasElement, scale);
async loadCharacterAtlas(url);     // Load pre-rendered CHARGEN PNG
setScreenMemory(data);             // Update from emulator
setColorMemory(data);              // Update from emulator
setBorderColor(color);             // Update border
setBackgroundColor(color);         // Update background
markDirty(x, y);                   // Flag cell for redraw
render();                          // Redraw dirty regions
renderFull();                      // Full screen redraw
```

**Rendering Strategy**:
```javascript
render() {
  // Clear and redraw only dirty regions
  for (const coord of this.#dirtyRegions) {
    const [x, y] = coord.split(',').map(Number);
    const charCode = this.#screenMemory[y * 40 + x];
    const color = this.#colorMemory[y * 40 + x];

    // Clear cell background
    this.#ctx.fillStyle = this.#getColor(this.#backgroundColor);
    this.#ctx.fillRect(x * 8 * this.#scale, y * 8 * this.#scale,
                        8 * this.#scale, 8 * this.#scale);

    // Draw character from atlas
    this.#drawCharacter(charCode, x, y, color);
  }

  this.#dirtyRegions.clear();
}
```

**Color Palette** (C64 16-color RGB values):
```javascript
const C64_PALETTE = [
  '#000000',  // 0: Black
  '#FFFFFF',  // 1: White
  '#880000',  // 2: Red
  '#AAFFEE',  // 3: Cyan
  '#CC44CC',  // 4: Purple
  '#00CC55',  // 5: Green
  '#0000AA',  // 6: Blue
  '#EEEE77',  // 7: Yellow
  '#DD8855',  // 8: Orange
  '#664400',  // 9: Brown
  '#FF7777',  // 10: Light Red
  '#333333',  // 11: Dark Gray
  '#777777',  // 12: Gray
  '#AAFF66',  // 13: Light Green
  '#0088FF',  // 14: Light Blue
  '#BBBBBB',  // 15: Light Gray
];
```

---

### 3.2 CharacterAtlas

**Purpose**: Pre-rendered PETSCII character set for efficient rendering.

**Format**: PNG image, 16×16 grid (256 characters)
- Dimensions: 128×128 pixels (8×8 chars × 16 rows × 16 cols)
- Color: Monochrome (white glyphs on transparent background)
- Source: Extracted from CHARGEN ROM ($D000-$DFFF in C64 memory)

**Generation** (one-time build step):
```javascript
// Convert CHARGEN ROM binary to PNG atlas
function generateCharacterAtlas(chargenData) {
  const canvas = new OffscreenCanvas(128, 128);
  const ctx = canvas.getContext('2d');

  for (let charCode = 0; charCode < 256; charCode++) {
    const x = (charCode % 16) * 8;
    const y = Math.floor(charCode / 16) * 8;

    // Extract 8×8 bitmap from CHARGEN ROM
    const offset = charCode * 8;
    for (let row = 0; row < 8; row++) {
      const byte = chargenData[offset + row];
      for (let col = 0; col < 8; col++) {
        if (byte & (1 << (7 - col))) {
          ctx.fillStyle = 'white';
          ctx.fillRect(x + col, y + row, 1, 1);
        }
      }
    }
  }

  return canvas.convertToBlob({ type: 'image/png' });
}
```

---

## 4. Data Flow

### 4.1 Initialization Sequence

```
1. User loads demo/c64/index.html
   ↓
2. JavaScript fetches WASM module + ROMs
   ↓
3. C64Emulator.init(basicRom, kernalRom, chargenRom)
   ↓
4. Rust creates MappedMemory with devices:
   - RamDevice at $0000 (64KB with ROM overlays)
   - RomDevice (BASIC) at $A000
   - RomDevice (KERNAL) at $E000
   - Vic2Device at $D000
   - CiaDevice at $DC00
   - Color RAM at $D800
   ↓
5. CPU reset to KERNAL entry point ($FFFC vector)
   ↓
6. KERNAL initializes VIC-II, clears screen
   ↓
7. BASIC cold start, displays "READY." prompt
   ↓
8. PetsciiDisplay renders screen from emulator
   ↓
9. KeyboardMatrix starts listening for input
   ↓
10. Emulation loop begins (60 FPS)
```

### 4.2 Per-Frame Data Flow

```
requestAnimationFrame (60 Hz)
   ↓
C64Emulator.runFrame()
   ↓
Rust: CPU executes ~16667 cycles
   ↓
CPU writes to screen RAM ($0400-$07E7)
   ↓
MappedMemory routes writes to RamDevice
   ↓
[Frame complete]
   ↓
JavaScript: getScreenMemory() → Uint8Array[1000]
JavaScript: getColorMemory() → Uint8Array[1000]
   ↓
PetsciiDisplay.setScreenMemory(data)
   ↓
Compare with previous frame, mark dirty cells
   ↓
PetsciiDisplay.render() → Canvas 2D API
   ↓
[Screen updated]
```

### 4.3 Keyboard Input Flow

```
User presses key
   ↓
Browser: keydown event
   ↓
KeyboardMatrix.handleKeyDown(event)
   ↓
Lookup event.code in KEY_MAP
   ↓
C64Emulator.keyDown(row, col)
   ↓
WASM: CiaDevice.set_key_state(row, col, true)
   ↓
Update keyboard_matrix[row] |= (1 << col)
   ↓
[Key state stored]
   ↓
CPU executes: LDA $DC01 (read CIA Port B)
   ↓
CiaDevice.read(offset=1)
   ↓
Return keyboard_matrix[current_selected_row]
   ↓
KERNAL interprets key press
   ↓
KERNAL writes PETSCII character to screen RAM
   ↓
[Character appears on display next frame]
```

---

## 5. Validation Rules

### 5.1 VIC-II Register Constraints

- Border color ($D020): Must be 0-15 (4-bit value)
- Background color ($D021): Must be 0-15 (4-bit value)
- Memory pointers ($D018): Bits 4-7 select screen base (1KB increments), bits 1-3 select character ROM (2KB increments)
- Control registers must maintain valid mode bits (text vs graphics)

### 5.2 CIA Register Constraints

- Port A/B data direction: Typically fixed for keyboard (DDRA=$FF, DDRB=$00)
- Timer values: 16-bit unsigned (0-65535)
- Keyboard matrix: Each row is 8 bits (active low)

### 5.3 Memory Map Constraints

- No device overlaps allowed (enforced by `MappedMemory::add_device`)
- ROM regions must be read-only (`RomDevice` ignores writes)
- Screen RAM must be at $0400-$07E7 (KERNAL default)
- Color RAM must be at $D800-$DBFF (hardware-fixed)

### 5.4 Display Constraints

- Screen grid: Exactly 40×25 characters (1000 cells)
- Character codes: 0-255 (PETSCII values)
- Color values: 0-15 (C64 palette indices)
- Scale factor: Integer multiples (1x, 2x, 4x) for pixel-perfect rendering

---

## 6. State Transitions

### 6.1 Emulator Lifecycle

```
[Uninitialized]
   ↓ init(roms)
[Initialized]
   ↓ start()
[Running]
   ↓ stop()
[Paused]
   ↓ start()
[Running]
   ↓ reset()
[Initialized] (back to boot state)
```

### 6.2 Key State Transitions

```
[Key Released] (matrix bit = 1)
   ↓ keyDown()
[Key Pressed] (matrix bit = 0)
   ↓ keyUp()
[Key Released] (matrix bit = 1)
```

### 6.3 Display Update States

```
[Clean] (no dirty regions)
   ↓ Screen RAM write
[Dirty] (changed cells marked)
   ↓ render()
[Rendering] (Canvas updates in progress)
   ↓ Complete
[Clean]
```

---

## 7. Performance Considerations

**Memory Footprint**:
- Rust/WASM: ~2MB compiled module
- ROMs: 20KB total (8KB + 8KB + 4KB)
- Screen/Color RAM: 2KB (1000 bytes each)
- Character Atlas: ~50KB PNG
- Total: ~2.1MB initial load

**Per-Frame Costs**:
- CPU emulation: ~16667 instruction executions
- Screen comparison: 1000 byte comparison (typically <100 changed)
- Canvas rendering: 10-100 drawImage calls (dirty regions only)
- Target: 16ms per frame (60 FPS)

**Optimization Strategies**:
- Dirty region tracking reduces Canvas calls by 80-90%
- Pre-rendered atlas eliminates dynamic character generation
- Integer scaling avoids sub-pixel interpolation overhead
- requestAnimationFrame syncs to monitor refresh

---

**Data Model Status**: ✅ Complete
**Next**: Generate contracts/ directory and quickstart.md
