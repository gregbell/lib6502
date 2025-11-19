# Research: xterm.js Serial Terminal Integration

**Feature**: 005-xterm-serial-connection
**Date**: 2025-11-18
**Purpose**: Resolve technical unknowns and make technology decisions for UART terminal integration

## Overview

This document resolves the NEEDS CLARIFICATION items from the technical context and establishes best practices for integrating xterm.js with the 6502 emulator's UART device.

## Research Items

### 1. xterm.js Version Selection

**Decision**: Use xterm.js 5.5.0 with @xterm/addon-fit 0.10.0

**Rationale**:
- Latest stable release (April 2025) from the 5.x series
- Modern @xterm scoped packages (replacing legacy xterm-addon-*)
- GPU acceleration support with `rescaleOverlappingGlyphs` option
- Well-documented CDN integration for vanilla JavaScript
- Active maintenance and browser compatibility

**Alternatives Considered**:
- **xterm.js 4.x**: Rejected - older API, missing modern features
- **xterm.js 6.x (if available)**: Not yet released; 5.5.0 is current stable

**CDN Integration**:
```html
<!-- CSS -->
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/css/xterm.css" />

<!-- JavaScript -->
<script src="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/lib/xterm.js"></script>
<script src="https://cdn.jsdelivr.net/npm/@xterm/addon-fit@0.10.0/lib/addon-fit.js"></script>
```

### 2. UART Memory Address

**Decision**: Map UART device at $A000-$A003

**Rationale**:
- Clean 4KB page boundary (A15=1, A14=0, A13=1)
- Follows hobbyist 6502 system conventions
- Simple address decode logic
- Non-conflicting with typical RAM ($0000-$7FFF) and ROM ($C000-$FFFF) regions
- Leaves room for future devices at $8000 and $C000

**Register Map**:
```
$A000: Data Register (R/W) - Transmit/Receive data
$A001: Status Register (R) - RDRF, TDRE, OVRN flags
$A002: Command Register (R/W) - Echo mode, control bits
$A003: Control Register (R/W) - Configuration
```

**Alternatives Considered**:
- **$C000**: Rejected - typically reserved for ROM in 6502 systems
- **$8000**: Rejected - could conflict with RAM expansion, less standard
- **$FE08 (BBC Micro style)**: Rejected - too high in address space, conflicts with vectors

**Memory Map**:
```
$0000-$7FFF: RAM (32KB)
$8000-$9FFF: [Future expansion]
$A000-$A003: UART (W65C51 ACIA)
$A004-$BFFF: [Future I/O devices]
$C000-$FFFF: ROM (16KB including vectors)
```

### 3. xterm.js Integration Pattern

**Decision**: Component-based architecture matching existing demo patterns

**Pattern**:
```javascript
// components/terminal.js
export class Terminal {
    constructor(containerId) {
        this.term = new Terminal({ /* config */ });
        this.fitAddon = new FitAddon.FitAddon();
        this.term.loadAddon(this.fitAddon);
        this.term.open(document.getElementById(containerId));
        this.fitAddon.fit();
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Handle user input
        this.term.onData((data) => {
            // Dispatch custom event for UART injection
            document.dispatchEvent(new CustomEvent('terminal-data', {
                detail: { data }
            }));
        });

        // Handle resize
        window.addEventListener('resize', () => {
            this.fitAddon.fit();
        });
    }

    write(text) {
        this.term.write(text);
    }

    clear() {
        this.term.clear();
    }
}
```

**Rationale**:
- Matches existing component structure (editor.js, registers.js, etc.)
- Uses CustomEvent pattern for communication (same as existing demo)
- Encapsulates xterm.js complexity
- Testable and maintainable

**Alternatives Considered**:
- **Direct xterm.js usage in app.js**: Rejected - less modular
- **Callback-based API**: Rejected - event-driven is more idiomatic for existing demo

### 4. WASM API Extension Strategy

**Decision**: Extend Emulator6502 to use MappedMemory with UART device

**Implementation Approach**:
1. Add new constructor variant: `Emulator6502::new_with_uart()`
2. Initialize MappedMemory instead of FlatMemory
3. Add RamDevice for RAM region
4. Add Uart6551 at $A000 with transmit callback
5. Store Uart reference separately for receive_byte() access

**Code Pattern** (from examples/wasm_terminal.rs):
```rust
#[wasm_bindgen]
pub struct Emulator6502 {
    cpu: CPU<MappedMemory>,
    uart: Rc<RefCell<Uart6551>>,  // Separate reference
}

#[wasm_bindgen]
impl Emulator6502 {
    #[wasm_bindgen(constructor)]
    pub fn new(on_transmit: js_sys::Function) -> Result<Emulator6502, JsValue> {
        let mut memory = MappedMemory::new();

        // Add RAM
        memory.add_device(0x0000, Box::new(RamDevice::new(32768)))?;

        // Add UART with transmit callback
        let uart = Rc::new(RefCell::new(Uart6551::new()));
        let uart_clone = Rc::clone(&uart);
        uart.borrow_mut().set_transmit_callback(move |byte| {
            let char_str = String::from_utf8(vec![byte]).unwrap_or("?".to_string());
            let _ = on_transmit.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
        });
        memory.add_device(0xA000, Box::new(uart.borrow_mut()))?;

        // Add ROM
        let rom = RomDevice::new(vec![0xEA; 16384]); // NOP filled
        memory.add_device(0xC000, Box::new(rom))?;

        let cpu = CPU::new(memory);

        Ok(Emulator6502 { cpu, uart })
    }

    pub fn receive_char(&mut self, byte: u8) {
        self.uart.borrow_mut().receive_byte(byte);
    }
}
```

**Rationale**:
- Minimal changes to existing WASM API
- Reuses existing Device trait infrastructure
- Maintains backward compatibility by keeping original new() method
- Follows pattern from examples/wasm_terminal.rs documentation

**Alternatives Considered**:
- **Modify existing new()**: Rejected - would break existing demo
- **Expose MappedMemory directly**: Rejected - too complex for JavaScript interface

### 5. Event Flow Architecture

**Decision**: Bidirectional event-driven architecture

**JavaScript → UART Flow**:
1. User types in xterm.js terminal
2. `term.onData()` fires with character
3. Terminal component dispatches `terminal-data` CustomEvent
4. App.js listener calls `emulator.receive_char(byte)`
5. WASM calls `uart.receive_byte()` to queue in 256-byte buffer
6. UART sets RDRF status flag
7. 6502 program reads from $A000 when polling $A001

**UART → JavaScript Flow**:
1. 6502 program writes to $A000
2. UART device write() method triggered
3. Transmit callback invokes JavaScript function
4. App.js receives character, calls `terminal.write(char)`
5. xterm.js displays character

**Rationale**:
- Maintains clean separation between terminal UI and emulator core
- Event-driven matches existing demo architecture
- Allows independent testing of components

## Best Practices Summary

### Terminal Configuration
- **Font**: Monospace, 12-14px for readability
- **Size**: 80x24 (classic terminal dimensions)
- **Theme**: Dark background matching demo aesthetic
- **Cursor**: Blinking block for retro feel

### Character Handling
- Convert JS strings to byte values: `data.charCodeAt(i)`
- Handle special keys:
  - Enter: `\r` (0x0D)
  - Backspace: `\x7f` (0x7F)
  - Printable ASCII: 0x20-0x7E
- Non-printable characters: Display as `?` or escape sequences

### Performance Optimization
- Batch terminal writes when possible
- Use `term.write()` instead of multiple `term.writeln()` calls
- FitAddon resize: Debounce window resize events (existing pattern)
- UART buffer: Trust 256-byte capacity, no additional buffering needed

### Testing Strategy
1. **Echo Test**: Simple LDA $A000 / STA $A000 loop
2. **Polling Test**: Check RDRF flag before read
3. **Buffering Test**: Type rapidly to test 256-byte queue
4. **Status Flags**: Verify TDRE always set, RDRF toggles correctly

## Dependencies Summary

**External (CDN)**:
- @xterm/xterm@5.5.0 (MIT license)
- @xterm/addon-fit@0.10.0 (MIT license)

**Internal (Existing)**:
- src/devices/uart.rs (W65C51 ACIA implementation)
- src/memory.rs (MappedMemory, Device trait)
- src/devices/mod.rs (RAM, ROM devices)
- src/wasm/api.rs (WASM bindings)

**No New Rust Dependencies**: All required functionality exists in lib6502

## Open Questions

None - all technical unknowns have been resolved.

## References

- xterm.js Documentation: https://xtermjs.org/docs/
- W65C51 Datasheet: Historical ACIA specifications
- 6502.org: Common memory maps for hobbyist systems
- examples/wasm_terminal.rs: Internal integration pattern documentation
