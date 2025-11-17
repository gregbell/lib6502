# Quick Start: Memory Mapping Module with UART Device Support

**Feature**: 004-memory-mapping-module
**Date**: 2025-11-17

## Overview

This guide shows how to create a 6502 system with memory-mapped devices and UART serial communication. Examples progress from simple to complex.

## Example 1: Simple RAM/ROM System

**Goal**: Create a 6502 system with 32KB RAM and 32KB ROM.

```rust
use lib6502::{CPU, MappedMemory, RamDevice, RomDevice};

fn main() {
    // Create memory mapper
    let mut memory = MappedMemory::new();

    // Add 32KB RAM at 0x0000-0x7FFF
    let ram = RamDevice::new(32768);
    memory.add_device(0x0000, Box::new(ram))
        .expect("Failed to add RAM");

    // Load ROM data
    let rom_data = vec![
        0xA9, 0x42,  // LDA #$42
        0x85, 0x10,  // STA $10
        // ... more program bytes
    ];
    let rom = RomDevice::new(rom_data);

    // Add ROM at 0x8000-0xFFFF
    memory.add_device(0x8000, Box::new(rom))
        .expect("Failed to add ROM");

    // Create CPU with mapped memory
    let mut cpu = CPU::new(memory);

    // Run program
    cpu.run_for_cycles(1000);
}
```

**Key Points**:
- `MappedMemory` implements `MemoryBus` (used by CPU)
- Devices added with base address
- No overlap allowed (returns error)
- Unmapped addresses (none in this example) return 0xFF

---

## Example 2: Adding UART for Serial I/O

**Goal**: Add UART at 0x5000, transmit bytes to terminal.

```rust
use lib6502::{CPU, MappedMemory, RamDevice, Uart6551};
use std::io::{self, Write};

fn main() {
    let mut memory = MappedMemory::new();

    // Add 16KB RAM
    memory.add_device(0x0000, Box::new(RamDevice::new(16384)))
        .expect("Failed to add RAM");

    // Create UART
    let mut uart = Uart6551::new();

    // Set callback to print transmitted bytes to terminal
    uart.set_transmit_callback(|byte| {
        print!("{}", byte as char);
        io::stdout().flush().unwrap();
    });

    // Add UART at 0x5000-0x5003
    memory.add_device(0x5000, Box::new(uart))
        .expect("Failed to add UART");

    // Load program that transmits "Hello\n"
    let mut ram = RamDevice::new(16384);
    ram.load_bytes(0x0200, &[
        0xA2, 0x00,        // LDX #$00
        // loop:
        0xBD, 0x10, 0x02,  // LDA message,X
        0xF0, 0x06,        // BEQ done
        0x8D, 0x00, 0x50,  // STA $5000  ; UART data register
        0xE8,              // INX
        0x4C, 0x02, 0x02,  // JMP loop
        // done:
        0x00,              // BRK
        // message:
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x0A, 0x00,  // "Hello\n\0"
    ]);

    memory.add_device(0x0000, Box::new(ram)).unwrap();

    let mut cpu = CPU::new(memory);
    cpu.set_pc(0x0200);

    // Run until BRK
    while !cpu.is_halted() {
        cpu.step().unwrap();
    }

    // Output: Hello
}
```

**Key Points**:
- UART occupies 4 bytes (registers at base+0/1/2/3)
- Writing to base+0 (data register) triggers callback
- Callback receives transmitted byte immediately

---

## Example 3: UART Echo Program

**Goal**: Read from UART, echo back to transmit.

```rust
use lib6502::{CPU, MappedMemory, RamDevice, Uart6551};
use std::sync::{Arc, Mutex};

fn main() {
    let mut memory = MappedMemory::new();
    memory.add_device(0x0000, Box::new(RamDevice::new(16384))).unwrap();

    // Create UART with shared reference for input injection
    let uart = Arc::new(Mutex::new(Uart6551::new()));

    // Clone reference for transmit callback
    let uart_clone = uart.clone();
    uart.lock().unwrap().set_transmit_callback(move |byte| {
        print!("{}", byte as char);
        // In real browser app, this would call term.write()
    });

    memory.add_device(0x5000, Box::new(uart.lock().unwrap())).unwrap();

    // Load echo program
    // Pseudocode:
    // loop:
    //   wait for RDRF flag (bit 3 of status register)
    //   read data register
    //   write back to data register
    //   repeat

    let mut cpu = CPU::new(memory);

    // Simulate user typing 'A'
    uart.lock().unwrap().receive_byte(b'A');

    cpu.run_for_cycles(1000);

    // UART receives 'A', program echoes it back, prints to terminal
}
```

**Key Points**:
- `receive_byte()` queues data in RX buffer
- Status register bit 3 (RDRF) indicates data available
- Reading data register pops byte from buffer
- Real implementation needs access to UART from both CPU and external source

---

## Example 4: Browser Terminal Integration (WASM)

**Goal**: Connect UART to xterm.js in browser for bidirectional serial communication.

**See**: `examples/wasm_terminal.rs` for complete WASM integration patterns and browser compatibility notes.

**Rust WASM Side**:

```rust
use wasm_bindgen::prelude::*;
use lib6502::{CPU, MappedMemory, Uart6551, RamDevice, RomDevice};
use std::rc::Rc;
use std::cell::RefCell;

#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU<MappedMemory>,
    uart: Rc<RefCell<Uart6551>>,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(on_transmit: js_sys::Function) -> Result<Emulator, JsValue> {
        let mut memory = MappedMemory::new();

        // Add 32KB RAM at 0x0000-0x7FFF
        memory.add_device(0x0000, Box::new(RamDevice::new(32768)))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        // Create UART with transmit callback
        let mut uart = Uart6551::new();
        uart.set_transmit_callback(move |byte| {
            let char_str = String::from_utf8(vec![byte]).unwrap_or_else(|_| "?".to_string());
            let _ = on_transmit.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
        });

        // Store UART separately for receive_byte access
        let uart = Rc::new(RefCell::new(uart));

        memory.add_device(0x8000, Box::new(uart.borrow().clone()))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        // Add 16KB ROM at 0xC000-0xFFFF with reset vector
        let mut rom = vec![0xEA; 16384]; // Fill with NOP
        rom[0x3FFC] = 0x00; // Reset vector -> 0x0200
        rom[0x3FFD] = 0x02;
        memory.add_device(0xC000, Box::new(RomDevice::new(rom)))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        let cpu = CPU::new(memory);

        Ok(Emulator { cpu, uart })
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<(), JsValue> {
        self.cpu.step().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen]
    pub fn run_cycles(&mut self, cycles: u32) -> Result<(), JsValue> {
        self.cpu.run_for_cycles(cycles as usize)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen]
    pub fn receive_byte(&mut self, byte: u8) {
        self.uart.borrow_mut().receive_byte(byte);
    }

    #[wasm_bindgen]
    pub fn load_program(&mut self, address: u16, bytes: &[u8]) {
        for (i, &byte) in bytes.iter().enumerate() {
            let addr = address.wrapping_add(i as u16);
            self.cpu.memory_mut().write(addr, byte);
        }
    }
}
```

**JavaScript Side**:

```javascript
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import init, { Emulator } from './pkg/lib6502.js';

async function main() {
    // Initialize WASM module
    await init();

    // Create terminal
    const term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(document.getElementById('terminal'));
    fitAddon.fit();

    // Create emulator with transmit callback
    const emulator = new Emulator((char) => {
        term.write(char);  // Write to terminal when UART transmits
    });

    // Handle terminal input
    term.onData((data) => {
        for (let i = 0; i < data.length; i++) {
            emulator.receive_byte(data.charCodeAt(i));
        }
    });

    // Load echo program into RAM
    // LDA $8000; STA $8000; JMP $0200
    const program = new Uint8Array([
        0xAD, 0x00, 0x80,  // LDA $8000 (read UART)
        0x8D, 0x00, 0x80,  // STA $8000 (write UART)
        0x4C, 0x00, 0x02,  // JMP $0200 (loop)
    ]);
    emulator.load_program(0x0200, program);

    // Run emulator loop at ~60 FPS
    function runEmulator() {
        try {
            emulator.run_cycles(1000);  // ~1 MHz at 60 FPS
        } catch (e) {
            console.error('Emulator error:', e);
        }
        requestAnimationFrame(runEmulator);
    }

    runEmulator();
}

main();
```

**Build & Deploy**:

```bash
# Build WASM module
wasm-pack build --target web

# Serve locally
python3 -m http.server 8000

# Open browser
open http://localhost:8000
```

**Key Points**:
- Use `Rc<RefCell<>>` for shared UART access (not `Arc<Mutex<>>` in WASM)
- Transmit callback writes directly to xterm.js terminal
- Terminal `onData` event sends keypresses to UART via `receive_byte()`
- Emulator runs in `requestAnimationFrame` loop for smooth 60 FPS
- UART buffer (256 bytes) decouples input timing from CPU execution
- See `examples/wasm_terminal.rs` for complete patterns and browser compatibility
- See `specs/004-memory-mapping-module/browser-test-plan.md` for testing checklist

---

## Common Patterns

### Checking UART Status Before Read/Write

```asm
; Wait for received data
wait_rx:
    LDA $5001       ; Read status register
    AND #$08        ; Bit 3 = RDRF (receiver full)
    BEQ wait_rx     ; Loop until data available
    LDA $5000       ; Read data register

; Wait for transmitter ready (always true in this implementation)
wait_tx:
    LDA $5001       ; Read status register
    AND #$10        ; Bit 4 = TDRE (transmitter empty)
    BEQ wait_tx     ; Loop until ready
    STA $5000       ; Write data register
```

### Handling RX Buffer Overflow

```asm
check_overflow:
    LDA $5001       ; Read status register
    AND #$04        ; Bit 2 = overrun error
    BEQ no_overflow
    ; Handle overflow: read status, then data to clear flag
    LDA $5001       ; Read status (required to clear overrun)
    LDA $5000       ; Read data (clears overrun flag)
no_overflow:
```

### Echo Mode (Hardware)

```asm
init_echo:
    LDA #$08        ; Bit 3 = echo mode
    STA $5002       ; Write command register
    ; Now received bytes automatically transmitted
```

---

## Troubleshooting

### Problem: Transmitted bytes not appearing

**Check**:
1. Is transmit callback set? (`uart.set_transmit_callback(...)`)
2. Is 6502 writing to correct address? (base+0, not base+1/2/3)
3. Is callback executing? (add println!/console.log)

**Solution**: Verify callback is registered before device added to memory.

### Problem: Received bytes not readable

**Check**:
1. Is `receive_byte()` being called?
2. Is RX buffer full? (check bit 2 of status for overflow)
3. Is program reading status before data? (best practice)

**Solution**: Check buffer capacity (default 256 bytes). Increase if needed.

### Problem: Overlapping device error

**Check**:
1. Device ranges: base to base+size-1
2. Are any ranges overlapping?

**Example**:
```rust
// WRONG: Overlap at 0x4000
memory.add_device(0x0000, Box::new(RamDevice::new(0x4000)));  // 0x0000-0x3FFF
memory.add_device(0x4000, Box::new(RomDevice::new(...)));     // 0x4000-???
```

**Solution**: Ensure ranges don't overlap. Use power-of-2 sizes for clean boundaries.

### Problem: WASM compilation fails

**Check**:
1. All dependencies `no_std` compatible?
2. Callback using `'static` lifetime?
3. No OS-level I/O (`std::fs`, `std::net`)?

**Solution**: Follow WASM portability guidelines (see constitution principle II).

---

## Testing Checklist

Before deploying:

- [ ] RAM/ROM devices work in isolation
- [ ] UART transmit callback fires
- [ ] UART receive buffers data correctly
- [ ] Status register bits reflect correct state
- [ ] Overflow flag sets when buffer full
- [ ] Echo mode works (if implemented)
- [ ] WASM compiles without errors
- [ ] Browser terminal shows transmitted text
- [ ] Keyboard input appears in UART buffer
- [ ] No panics under normal operation
- [ ] No panics when buffer full

---

## Next Steps

- See [data-model.md](./data-model.md) for complete entity definitions
- See [contracts/device_trait.md](./contracts/device_trait.md) for Device trait specification
- See [plan.md](./plan.md) for architecture and design decisions
- Run `/speckit.tasks` to generate implementation task list

## References

- W65C51N ACIA Datasheet: https://www.westerndesigncenter.com/wdc/documentation/w65c51n.pdf
- xterm.js Documentation: https://xtermjs.org/docs/
- wasm-bindgen Guide: https://rustwasm.github.io/wasm-bindgen/
