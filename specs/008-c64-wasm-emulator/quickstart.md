# Quickstart: C64 WASM Emulator Development

**Date**: 2025-01-22
**Feature Branch**: `008-c64-wasm-emulator`

## Prerequisites

- Rust 1.75+ with `wasm32-unknown-unknown` target
- wasm-pack (for building WASM)
- Node.js 18+ (for development server)
- Modern browser with WebAssembly support

## Setup

```bash
# Clone and checkout branch
git clone https://github.com/example/lib6502-c64
cd lib6502-c64
git checkout 008-c64-wasm-emulator

# Install wasm-pack if not present
cargo install wasm-pack

# Add WASM target
rustup target add wasm32-unknown-unknown

# Build the library
cargo build

# Run tests
cargo test
```

## Project Structure

```
lib6502-c64/
├── src/
│   ├── lib.rs                # Public API
│   ├── cpu.rs                # 6502/6510 CPU
│   ├── devices/
│   │   └── c64/              # C64 chip implementations
│   │       ├── vic_ii.rs     # Video chip
│   │       ├── sid.rs        # Audio chip
│   │       ├── cia.rs        # Timer/I/O chips
│   │       └── ...
│   ├── c64/
│   │   ├── mod.rs            # C64 system
│   │   └── keyboard.rs       # Key mapping
│   └── wasm/
│       └── c64_api.rs        # WASM bindings
├── c64-demo/                 # Web frontend
│   ├── index.html
│   └── components/
└── specs/008-c64-wasm-emulator/
    ├── spec.md
    ├── plan.md
    ├── research.md
    └── data-model.md
```

## Building WASM

```bash
# Development build (fast, larger)
wasm-pack build --target web --dev

# Release build (optimized)
wasm-pack build --target web --release

# Output in pkg/ directory
ls pkg/
# lib6502_bg.wasm
# lib6502.js
# lib6502.d.ts
```

## Running the Demo

```bash
# Start development server (requires Python or Node)
cd c64-demo
python3 -m http.server 8080
# or
npx serve .

# Open browser
open http://localhost:8080
```

## Development Workflow

### Adding a New Device

1. Create device file in `src/devices/c64/`:

```rust
// src/devices/c64/vic_ii.rs
use crate::devices::Device;
use std::any::Any;

pub struct VicII {
    registers: [u8; 47],
    // ... state fields
}

impl Device for VicII {
    fn size(&self) -> u16 { 64 }  // With mirroring

    fn read(&self, offset: u16) -> u8 {
        self.registers[(offset % 47) as usize]
    }

    fn write(&mut self, offset: u16, value: u8) {
        let reg = (offset % 47) as usize;
        self.registers[reg] = value;
        // Handle side effects
    }

    fn has_interrupt(&self) -> bool {
        // Return true if IRQ pending
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
```

2. Export from module:

```rust
// src/devices/c64/mod.rs
mod vic_ii;
pub use vic_ii::VicII;
```

3. Add tests:

```rust
// tests/c64_vic_ii_tests.rs
use lib6502::devices::c64::VicII;
use lib6502::Device;

#[test]
fn test_vic_register_read_write() {
    let mut vic = VicII::new();
    vic.write(0x20, 0x06);  // Border color
    assert_eq!(vic.read(0x20), 0x06);
}
```

### Adding WASM API Methods

1. Add to `src/wasm/c64_api.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl C64Emulator {
    #[wasm_bindgen]
    pub fn get_border_color(&self) -> u8 {
        self.system.vic.read(0x20) & 0x0F
    }
}
```

2. Rebuild WASM:

```bash
wasm-pack build --target web
```

3. Use from JavaScript:

```javascript
const color = emulator.get_border_color();
console.log(`Border color: ${color}`);
```

## Testing

```bash
# Run all fast tests
cargo test

# Run specific test
cargo test vic_ii

# Run with output
cargo test -- --nocapture

# Run ignored tests (Klaus functional)
cargo test -- --ignored
```

## Common Tasks

### Load and Run PRG

```javascript
// Load PRG file
const prg = await fetch('game.prg').then(r => r.arrayBuffer());
const loadAddr = emulator.load_prg(new Uint8Array(prg));
console.log(`Loaded at $${loadAddr.toString(16)}`);

// Auto-run BASIC program
emulator.inject_basic_run();

// Or run machine code directly
emulator.write_memory(0x0801, 0x4C);  // JMP
emulator.write_memory(0x0802, loadAddr & 0xFF);
emulator.write_memory(0x0803, loadAddr >> 8);
```

### Handle Keyboard Input

```javascript
document.addEventListener('keydown', (e) => {
    emulator.key_down_pc(e.code);
    e.preventDefault();
});

document.addEventListener('keyup', (e) => {
    emulator.key_up_pc(e.code);
});
```

### Render Display

```javascript
const canvas = document.getElementById('screen');
const ctx = canvas.getContext('2d');
const imageData = ctx.createImageData(320, 200);

function render() {
    const ptr = emulator.get_framebuffer_ptr();
    const buffer = new Uint8Array(wasm.memory.buffer, ptr, 320 * 200);

    for (let i = 0; i < 64000; i++) {
        const rgb = C64_PALETTE[buffer[i]];
        imageData.data[i * 4 + 0] = rgb[0];
        imageData.data[i * 4 + 1] = rgb[1];
        imageData.data[i * 4 + 2] = rgb[2];
        imageData.data[i * 4 + 3] = 255;
    }

    ctx.putImageData(imageData, 0, 0);
}
```

### Play Audio

```javascript
const audioCtx = new AudioContext();
await audioCtx.audioWorklet.addModule('sid-processor.js');
const sidNode = new AudioWorkletNode(audioCtx, 'sid-processor');
sidNode.connect(audioCtx.destination);

// In AudioWorklet processor:
process(inputs, outputs) {
    const samples = emulator.get_audio_samples(128);
    outputs[0][0].set(samples);
    return true;
}
```

## Debugging Tips

### Memory Viewer

```javascript
// Dump zero page
for (let i = 0; i < 256; i += 16) {
    let line = i.toString(16).padStart(4, '0') + ': ';
    for (let j = 0; j < 16; j++) {
        line += emulator.read_memory(i + j).toString(16).padStart(2, '0') + ' ';
    }
    console.log(line);
}
```

### VIC-II State

```javascript
const regs = emulator.get_vic_registers();
console.log('VIC-II Registers:');
console.log(`  $D011 (CR1): ${regs[0x11].toString(16)}`);
console.log(`  $D012 (Raster): ${regs[0x12]}`);
console.log(`  $D020 (Border): ${regs[0x20]}`);
```

### Breakpoints

```javascript
function stepUntil(addr) {
    while (emulator.get_cpu_state().pc !== addr) {
        emulator.step_scanline();
    }
    console.log('Hit breakpoint at', addr.toString(16));
}
```

## Resources

- [C64 Memory Map](https://sta.c64.org/cbm64mem.html)
- [VIC-II Reference](https://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt)
- [SID Reference](https://www.oxyron.de/html/registers_sid.html)
- [CIA Reference](https://www.oxyron.de/html/registers_cia.html)
- [Codebase64](https://codebase64.org/)
