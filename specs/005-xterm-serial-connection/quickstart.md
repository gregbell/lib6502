# Quickstart Guide: xterm.js Serial Terminal Integration

**Feature**: 005-xterm-serial-connection
**Date**: 2025-11-18
**Audience**: Developers implementing the terminal integration

## Overview

This guide provides step-by-step instructions for implementing xterm.js serial terminal support in the 6502 emulator demo. Follow these steps in order to ensure proper integration.

## Prerequisites

- Rust 1.75+ installed
- wasm-pack installed (`cargo install wasm-pack`)
- Basic HTTP server for local testing (e.g., `python3 -m http.server`)
- Modern web browser (Chrome 85+, Firefox 78+, Safari 14+)

## Implementation Steps

### Phase 1: WASM Backend (Rust)

#### Step 1.1: Modify Emulator6502 Constructor

**File**: `src/wasm/api.rs`

**Action**: Replace `FlatMemory` with `MappedMemory` and add UART device

**Code**:
```rust
use crate::{
    assemble, disassemble, DisassemblyOptions, MappedMemory, RamDevice,
    RomDevice, Uart6551, MemoryBus, CPU, Device
};
use std::rc::Rc;
use std::cell::RefCell;

#[wasm_bindgen]
pub struct Emulator6502 {
    cpu: CPU<MappedMemory>,
    uart: Rc<RefCell<Uart6551>>,  // NEW: Store UART reference
    program_start: u16,
    program_end: u16,
}

#[wasm_bindgen]
impl Emulator6502 {
    #[wasm_bindgen(constructor)]
    pub fn new(on_transmit: js_sys::Function) -> Result<Emulator6502, JsValue> {
        let mut memory = MappedMemory::new();

        // Add 32KB RAM at $0000
        memory.add_device(0x0000, Box::new(RamDevice::new(32768)))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        // Create UART with transmit callback
        let uart = Rc::new(RefCell::new(Uart6551::new()));
        let uart_for_callback = Rc::clone(&uart);
        uart.borrow_mut().set_transmit_callback(move |byte| {
            let char_str = String::from_utf8(vec![byte]).unwrap_or_else(|_| "?".to_string());
            let _ = on_transmit.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
        });

        // Add UART at $A000
        memory.add_device(0xA000, Box::new(uart.borrow().clone()))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        // Add 16KB ROM at $C000 (with reset vector)
        let mut rom = vec![0xEA; 16384]; // NOP filled
        rom[0x3FFC] = 0x00; // Reset vector low → $0600
        rom[0x3FFD] = 0x06; // Reset vector high
        memory.add_device(0xC000, Box::new(RomDevice::new(rom)))
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        let cpu = CPU::new(memory);

        Ok(Emulator6502 {
            cpu,
            uart,
            program_start: 0x0600,
            program_end: 0x0600,
        })
    }

    /// Inject received character from terminal into UART
    pub fn receive_char(&mut self, byte: u8) {
        self.uart.borrow_mut().receive_byte(byte);
    }

    // ... rest of existing methods remain unchanged
}
```

**Note**: You may need to implement `Clone` for `Uart6551` or adjust the device storage pattern.

---

#### Step 1.2: Build WASM Module

**Command**:
```bash
wasm-pack build --target web --out-dir demo/lib6502_wasm
```

**Expected Output**:
```
demo/lib6502_wasm/
├── lib6502.js
├── lib6502_bg.wasm
├── lib6502.d.ts
└── package.json
```

**Verify**:
```bash
ls demo/lib6502_wasm/
```

---

### Phase 2: Frontend JavaScript

#### Step 2.1: Add xterm.js CDN Links

**File**: `demo/index.html`

**Action**: Add xterm.js stylesheet and scripts to `<head>` and before `</body>`

**Code**:
```html
<head>
  <!-- Existing links... -->
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">

  <!-- ADD: xterm.js CSS -->
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/css/xterm.css" />

  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <!-- Existing HTML... -->

  <!-- ADD: xterm.js JavaScript (before app.js) -->
  <script src="https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/lib/xterm.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/@xterm/addon-fit@0.10.0/lib/addon-fit.js"></script>

  <script type="module" src="app.js"></script>
</body>
```

---

#### Step 2.2: Add Terminal Container to HTML

**File**: `demo/index.html`

**Action**: Add terminal panel to the layout

**Code**:
```html
<main class="split-panel">
  <!-- Existing left panel... -->
  <section class="left-panel">
    <!-- ... editor panel ... -->
  </section>

  <!-- Existing right panel... -->
  <section class="right-panel">
    <!-- ... CPU state and memory panels ... -->

    <!-- ADD: Terminal Panel -->
    <div class="panel terminal-panel">
      <h2>Serial Terminal</h2>
      <div id="terminal-container"></div>
    </div>
  </section>
</main>
```

---

#### Step 2.3: Create Terminal Component

**File**: `demo/components/terminal.js` (NEW FILE)

**Code**:
```javascript
/**
 * Terminal Component
 * Wrapper for xterm.js with app integration
 */

export class Terminal {
    constructor(containerId) {
        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Terminal container not found: ${containerId}`);
        }

        // Create xterm.js instance
        this.term = new window.Terminal({
            cols: 80,
            rows: 24,
            fontSize: 14,
            fontFamily: 'JetBrains Mono, Courier New, monospace',
            cursorBlink: true,
            cursorStyle: 'block',
            theme: {
                background: '#1a1a1a',
                foreground: '#ffffff',
                cursor: '#00ff00'
            }
        });

        // Create and load FitAddon
        this.fitAddon = new window.FitAddon.FitAddon();
        this.term.loadAddon(this.fitAddon);

        // Open terminal
        this.term.open(container);
        this.fitAddon.fit();

        // Set up event listeners
        this.setupEventListeners();

        // Welcome message
        this.term.write('6502 Serial Terminal Ready\r\n');
        this.term.write('UART: $A000-$A003\r\n\r\n');
    }

    setupEventListeners() {
        // Handle user input
        this.term.onData((data) => {
            // Dispatch event for app to handle
            document.dispatchEvent(new CustomEvent('terminal-data', {
                detail: { data }
            }));
        });

        // Handle window resize
        let resizeTimeout;
        window.addEventListener('resize', () => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                this.fitAddon.fit();
            }, 100);
        });
    }

    write(text) {
        this.term.write(text);
    }

    clear() {
        this.term.clear();
    }

    fit() {
        this.fitAddon.fit();
    }
}
```

---

#### Step 2.4: Modify App Class

**File**: `demo/app.js`

**Action**: Import Terminal, create instance, handle events

**Code**:
```javascript
// ADD import at top
import { Terminal } from './components/terminal.js';

class App {
    constructor() {
        // ... existing properties ...
        this.terminal = null;  // ADD
    }

    async init() {
        try {
            await init();

            // MODIFY: Create emulator with transmit callback
            this.emulator = new Emulator6502((char) => {
                this.terminal.write(char);
            });

            // ... existing component initialization ...

            // ADD: Create terminal
            this.terminal = new Terminal('terminal-container');

            this.setupEventListeners();
            // ... rest of init ...
        } catch (error) {
            console.error('Failed to initialize demo:', error);
        }
    }

    setupEventListeners() {
        // ... existing listeners ...

        // ADD: Terminal input handler
        document.addEventListener('terminal-data', (e) => {
            this.handleTerminalInput(e.detail.data);
        });
    }

    // ADD: New method
    handleTerminalInput(data) {
        if (!data) return;

        for (let i = 0; i < data.length; i++) {
            const byte = data.charCodeAt(i);
            this.emulator.receive_char(byte);
        }
    }

    handleReset() {
        this.emulator.reset();
        // ... existing reset logic ...

        // ADD: Clear terminal on reset
        if (this.terminal) {
            this.terminal.clear();
            this.terminal.write('CPU Reset\r\n\r\n');
        }
    }
}
```

---

#### Step 2.5: Add Terminal Styling

**File**: `demo/styles.css`

**Action**: Add styles for terminal panel

**Code**:
```css
/* ADD: Terminal Panel Styles */
.terminal-panel {
    margin-top: 1rem;
}

.terminal-panel h2 {
    margin-bottom: 0.5rem;
}

#terminal-container {
    width: 100%;
    height: 400px;
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 4px;
    padding: 8px;
    overflow: hidden;
}
```

---

### Phase 3: UART Example Programs

#### Step 3.1: Add Echo Example

**File**: `demo/examples/uart-echo.asm` (NEW FILE)

**Code**:
```asm
; UART Echo Program
; Reads characters from UART and echoes them back

UART_DATA    = $A000
UART_STATUS  = $A001
UART_COMMAND = $A002

        ; Initialize
        LDA #$00
        STA UART_COMMAND    ; No echo mode

loop:
        ; Poll status register
        LDA UART_STATUS
        AND #$08            ; Check RDRF (bit 3)
        BEQ loop            ; Wait if no data

        ; Read and echo character
        LDA UART_DATA       ; Read byte
        STA UART_DATA       ; Write back
        JMP loop            ; Repeat
```

---

#### Step 3.2: Add Hello World Example

**File**: `demo/examples/uart-hello.asm` (NEW FILE)

**Code**:
```asm
; UART Hello World
; Prints "Hello, 6502!\r\n" to terminal

UART_DATA = $A000

        LDX #$00
loop:
        LDA message,X
        BEQ done            ; Stop at null terminator
        STA UART_DATA       ; Transmit character
        INX
        JMP loop

done:
        BRK                 ; End program

message:
        .byte "Hello, 6502!"
        .byte $0D, $0A      ; CR, LF
        .byte $00           ; Null terminator
```

---

#### Step 3.3: Register Examples in ExampleSelector

**File**: `demo/components/examples.js`

**Action**: Add UART examples to examples array

**Code**:
```javascript
getExamples() {
    return [
        // ... existing examples (counter, fibonacci, stack) ...

        // ADD: UART examples
        {
            id: 'uart-echo',
            name: 'UART Echo',
            description: 'Echo characters via serial terminal',
            code: `; UART Echo Program
UART_DATA    = $A000
UART_STATUS  = $A001

loop:
        LDA UART_STATUS     ; Read status
        AND #$08            ; Check RDRF
        BEQ loop            ; Wait for data
        LDA UART_DATA       ; Read char
        STA UART_DATA       ; Echo back
        JMP loop`
        },
        {
            id: 'uart-hello',
            name: 'Hello World',
            description: 'Print message to terminal',
            code: `; UART Hello World
UART_DATA = $A000

        LDX #$00
loop:
        LDA message,X
        BEQ done
        STA UART_DATA
        INX
        JMP loop
done:
        BRK

message:
        .text "Hello, 6502!"
        .byte $0D, $0A, $00`
        }
    ];
}
```

---

### Phase 4: Testing

#### Step 4.1: Build and Serve

**Commands**:
```bash
# Build WASM
wasm-pack build --target web --out-dir demo/lib6502_wasm

# Serve demo
cd demo
python3 -m http.server 8000
```

**Open**: http://localhost:8000

---

#### Step 4.2: Manual Test Cases

**Test 1: Echo**
1. Load "UART Echo" example
2. Click "Assemble"
3. Click "Run"
4. Type "Hello" in terminal
5. Verify "Hello" appears in terminal output

**Test 2: Hello World**
1. Load "Hello World" example
2. Click "Assemble"
3. Click "Run"
4. Verify "Hello, 6502!" appears in terminal

**Test 3: Rapid Input**
1. Load "UART Echo" example
2. Run the program
3. Type rapidly or paste long text
4. Verify no characters dropped (256-byte buffer)

**Test 4: Reset**
1. Run any UART program
2. Type some text
3. Click "Reset"
4. Verify terminal clears
5. Verify UART buffer cleared

---

## Common Issues

### Issue 1: "Terminal container not found"

**Cause**: HTML container missing or wrong ID

**Fix**: Verify `<div id="terminal-container"></div>` exists in index.html

---

### Issue 2: xterm.js not loaded

**Cause**: CDN script tags missing or wrong order

**Fix**: Ensure xterm.js scripts load before app.js:
```html
<script src=".../xterm.js"></script>
<script src=".../addon-fit.js"></script>
<script type="module" src="app.js"></script>
```

---

### Issue 3: Characters not echoing

**Cause**: UART device not in memory map or wrong address

**Fix**: Verify in browser console:
```javascript
app.emulator.read_memory(0xA001); // Should return status byte
```

---

### Issue 4: "Cannot read property 'write' of undefined"

**Cause**: Terminal created after emulator (callback fails)

**Fix**: Ensure terminal is created BEFORE emulator in app.init():
```javascript
this.terminal = new Terminal('terminal-container');  // First
this.emulator = new Emulator6502((char) => {         // Then
    this.terminal.write(char);
});
```

---

## Verification Checklist

- [ ] WASM builds without errors
- [ ] Browser console shows no errors on page load
- [ ] Terminal displays welcome message
- [ ] Typing in terminal dispatches `terminal-data` event
- [ ] Echo program echoes typed characters
- [ ] Hello World program prints to terminal
- [ ] Reset clears terminal
- [ ] Memory viewer shows UART at $A000-$A003
- [ ] Rapid typing doesn't crash (256-byte buffer)

---

## Next Steps

After completing this quickstart:

1. **Review `/speckit.tasks`**: Run task generation for detailed implementation steps
2. **Enhance examples**: Add interrupt-driven UART, buffered I/O examples
3. **Add terminal controls**: Clear button, font size selector
4. **Testing**: Add automated tests for UART integration
5. **Documentation**: Update main README with UART usage guide

---

## Summary

You've successfully integrated xterm.js serial terminal support by:

1. ✅ Modified WASM backend to use MappedMemory with UART at $A000
2. ✅ Created Terminal component wrapping xterm.js
3. ✅ Connected terminal input → UART receive → CPU read flow
4. ✅ Connected CPU write → UART transmit → terminal display flow
5. ✅ Added UART example programs
6. ✅ Tested bidirectional communication

The demo now supports interactive serial I/O, enabling users to learn 6502 UART programming patterns in the browser!
