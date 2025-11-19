# Data Model: xterm.js Serial Terminal Integration

**Feature**: 005-xterm-serial-connection
**Date**: 2025-11-18
**Purpose**: Define data structures, state management, and entity relationships for terminal-UART integration

## Overview

This feature does not involve traditional database entities. Instead, the data model describes:
- In-memory state structures for terminal and UART communication
- Data flow between JavaScript and WASM
- Event payload structures
- UART register state machine

## Core Entities

### 1. Terminal Component State

**Entity**: `Terminal` (JavaScript class)

**Purpose**: Manage xterm.js instance and handle terminal I/O

**Attributes**:

| Attribute | Type | Description | Mutability |
|-----------|------|-------------|------------|
| `term` | `Terminal` (xterm.js) | Terminal instance | Immutable after init |
| `fitAddon` | `FitAddon` | Resize addon instance | Immutable after init |
| `containerId` | `string` | DOM element ID for terminal | Immutable |

**State** (internal to xterm.js):
- Cursor position (row, col)
- Display buffer (lines × columns matrix)
- Scrollback history (configurable, default 1000 lines)

**Methods**:
- `constructor(containerId: string)`: Initialize terminal
- `write(text: string)`: Output to terminal display
- `clear()`: Clear terminal screen
- `fit()`: Resize terminal to container

**Lifecycle**:
1. Created during app initialization
2. Lives for entire page session
3. Cleared on CPU reset (not destroyed)

---

### 2. UART Device State

**Entity**: `Uart6551` (Rust struct, exposed via WASM)

**Purpose**: Emulate W65C51 ACIA serial device with memory-mapped registers

**Attributes** (from src/devices/uart.rs):

| Attribute | Type | Size | Description | Access |
|-----------|------|------|-------------|--------|
| `data_register` | `u8` | 1 byte | Last transmitted/received byte | R/W at $A000 |
| `status_register` | `RefCell<u8>` | 1 byte | Status flags (TDRE, RDRF, OVRN) | R at $A001 |
| `command_register` | `u8` | 1 byte | Control bits (echo mode, etc.) | R/W at $A002 |
| `control_register` | `u8` | 1 byte | Configuration register | R/W at $A003 |
| `rx_buffer` | `RefCell<VecDeque<u8>>` | 256 bytes max | FIFO receive queue | Internal |
| `on_transmit` | `Option<Box<dyn Fn(u8)>>` | Callback | JavaScript transmit callback | Internal |

**Status Register Flags** ($A001):

| Bit | Name | Meaning | Set When | Cleared When |
|-----|------|---------|----------|--------------|
| 4 | TDRE | Transmit Data Register Empty | Always (immediate transmit) | Never |
| 3 | RDRF | Receive Data Register Full | rx_buffer not empty | rx_buffer empty after read |
| 2 | OVRN | Overrun Error | rx_buffer overflow (>256 bytes) | Next successful read from $A000 |
| 1-0 | - | Reserved | Always 0 | Always 0 |

**State Transitions**:

```
Receive Flow:
[Terminal Input] → receive_byte() → rx_buffer.push()
                                   → RDRF flag = 1

[CPU Reads $A000] → rx_buffer.pop() → data_register
                                    → RDRF flag = 0 (if buffer empty)

Transmit Flow:
[CPU Writes $A000] → data_register = value
                   → on_transmit callback → JavaScript
                   → TDRE remains 1
```

---

### 3. Emulator Instance State

**Entity**: `Emulator6502` (Rust struct, WASM-bindgen)

**Purpose**: Bridge between JavaScript and CPU/UART

**Attributes** (modified from src/wasm/api.rs):

| Attribute | Type | Description | Lifetime |
|-----------|------|-------------|----------|
| `cpu` | `CPU<MappedMemory>` | CPU core with memory-mapped devices | Session |
| `uart` | `Rc<RefCell<Uart6551>>` | Shared reference to UART device | Session |
| `program_start` | `u16` | Last loaded program start address | Per-load |
| `program_end` | `u16` | Last loaded program end address | Per-load |

**Methods** (new/modified):
- `new(on_transmit: js_sys::Function) -> Result<Self, JsValue>`: Initialize with UART callback
- `receive_char(byte: u8)`: Inject character into UART rx_buffer
- `reset()`: Reset CPU (preserves UART state)

---

### 4. App Instance State

**Entity**: `App` (JavaScript class in app.js)

**Purpose**: Coordinate emulator, terminal, and UI components

**Attributes** (modified from demo/app.js):

| Attribute | Type | Description | Added/Modified |
|-----------|------|-------------|----------------|
| `emulator` | `Emulator6502` | WASM emulator instance | Modified (new constructor) |
| `terminal` | `Terminal` | Terminal component | NEW |
| `editor` | `CodeEditor` | Assembly code editor | Existing |
| `registerDisplay` | `RegisterDisplay` | CPU registers display | Existing |
| `flagsDisplay` | `FlagsDisplay` | CPU flags display | Existing |
| `memoryViewer` | `MemoryViewer` | Memory viewer | Existing |
| `controlPanel` | `ControlPanel` | Execution controls | Existing |
| `errorDisplay` | `ErrorDisplay` | Error messages | Existing |
| `exampleSelector` | `ExampleSelector` | Example programs | Modified (add UART examples) |

**Event Handlers** (new):
- `terminal-data`: User typed in terminal → inject into UART
- `terminal-clear`: Clear terminal button clicked → clear display

---

## Data Flow Diagrams

### User Types → CPU Reads

```
┌─────────────┐
│   User      │
│  Keyboard   │
└──────┬──────┘
       │ keypress
       ▼
┌─────────────────┐
│  xterm.js       │
│  term.onData()  │
└──────┬──────────┘
       │ CustomEvent('terminal-data')
       ▼
┌──────────────────┐
│  App.js          │
│  event listener  │
└──────┬───────────┘
       │ emulator.receive_char(byte)
       ▼
┌──────────────────────┐
│  WASM Emulator       │
│  receive_char()      │
└──────┬───────────────┘
       │ uart.receive_byte()
       ▼
┌──────────────────────┐
│  Uart6551            │
│  rx_buffer.push()    │
│  RDRF flag ← 1       │
└──────┬───────────────┘
       │ CPU polling $A001
       ▼
┌──────────────────────┐
│  6502 Program        │
│  LDA $A001 (status)  │
│  AND #$08 (RDRF?)    │
│  BEQ poll_loop       │
│  LDA $A000 (read)    │
└──────────────────────┘
```

### CPU Writes → Terminal Display

```
┌──────────────────────┐
│  6502 Program        │
│  STA $A000 (write)   │
└──────┬───────────────┘
       │ memory.write($A000, byte)
       ▼
┌──────────────────────┐
│  Uart6551            │
│  write_data_register │
└──────┬───────────────┘
       │ on_transmit callback
       ▼
┌──────────────────────┐
│  WASM → JS Boundary  │
│  js_sys::Function    │
└──────┬───────────────┘
       │ transmit callback(char)
       ▼
┌──────────────────────┐
│  App.js              │
│  terminal.write()    │
└──────┬───────────────┘
       │ term.write(char)
       ▼
┌──────────────────────┐
│  xterm.js            │
│  Display Buffer      │
└──────────────────────┘
```

## Event Payload Structures

### CustomEvent: `terminal-data`

**Direction**: JavaScript → WASM

**Payload**:
```javascript
{
  detail: {
    data: string  // Characters typed (may be multiple chars from paste)
  }
}
```

**Processing**:
```javascript
document.addEventListener('terminal-data', (e) => {
  const data = e.detail.data;
  for (let i = 0; i < data.length; i++) {
    const byte = data.charCodeAt(i);
    emulator.receive_char(byte);
  }
});
```

### WASM Callback: `on_transmit`

**Direction**: WASM → JavaScript

**Signature**:
```rust
Fn(u8) -> ()  // Takes byte, returns nothing
```

**JavaScript Binding**:
```javascript
const emulator = new Emulator6502((charStr) => {
  terminal.write(charStr);
});
```

**Character Conversion**:
```rust
// Rust side (in WASM binding)
let char_str = String::from_utf8(vec![byte]).unwrap_or("?".to_string());
callback.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
```

## Validation Rules

### Terminal Input

| Rule | Validation | Action |
|------|------------|--------|
| Printable ASCII (0x20-0x7E) | Pass through | Send to UART |
| Control characters (0x00-0x1F) | Special handling | Enter (0x0D), Backspace (0x7F) |
| Extended ASCII (0x80-0xFF) | Convert to `?` (0x3F) | Prevent UART confusion |
| Empty string | Ignore | No action |

### UART Buffer

| Rule | Condition | Behavior |
|------|-----------|----------|
| Buffer full (256 bytes) | New receive_byte() | Set OVRN flag, drop new byte |
| Read from empty buffer | $A000 read when RDRF=0 | Return last_rx_byte or 0x00 |
| TDRE flag | Any time | Always 1 (immediate transmit) |

## Memory Map

```
┌─────────────────────────────┐
│ $0000-$7FFF  RAM (32KB)     │  RamDevice
├─────────────────────────────┤
│ $8000-$9FFF  [Future I/O]   │  Unmapped
├─────────────────────────────┤
│ $A000        UART Data      │  Uart6551
│ $A001        UART Status    │  Uart6551
│ $A002        UART Command   │  Uart6551
│ $A003        UART Control   │  Uart6551
├─────────────────────────────┤
│ $A004-$BFFF  [Future I/O]   │  Unmapped
├─────────────────────────────┤
│ $C000-$FFFF  ROM (16KB)     │  RomDevice (or RAM for demo)
│              $FFFC-$FFFD    │  Reset vector → $0600
└─────────────────────────────┘
```

## State Persistence

**No Persistence Required**:
- Terminal history: Cleared on page refresh (intentional)
- UART buffer: Cleared on CPU reset
- Program code: Stored in editor component (not terminal-related)

**Reset Behavior**:
```
CPU Reset:
- Clear UART rx_buffer
- Reset status flags
- Clear terminal display (via app.js handler)
- Preserve terminal component instance
```

## Relationships

```
App (1) ─┬─ (1) Terminal
         ├─ (1) Emulator6502 ─── (1) Uart6551
         ├─ (1) CodeEditor
         ├─ (1) RegisterDisplay
         ├─ (1) FlagsDisplay
         ├─ (1) MemoryViewer
         ├─ (1) ControlPanel
         ├─ (1) ErrorDisplay
         └─ (1) ExampleSelector

Emulator6502 (1) ── (1) CPU<MappedMemory>
                  └─ (1) Uart6551 (shared Rc<RefCell>)

MappedMemory (1) ─┬─ (1) RamDevice (0x0000)
                  ├─ (1) Uart6551 (0xA000)
                  └─ (1) RomDevice (0xC000)
```

## Type Definitions (TypeScript-style for documentation)

```typescript
// Terminal Component
interface TerminalConfig {
  cols: number;          // 80
  rows: number;          // 24
  fontSize: number;      // 12-14
  fontFamily: string;    // 'monospace'
}

// Event Payloads
interface TerminalDataEvent {
  detail: {
    data: string;  // Characters typed
  }
}

// UART Status Flags (bitfield)
interface UartStatus {
  TDRE: boolean;  // Bit 4
  RDRF: boolean;  // Bit 3
  OVRN: boolean;  // Bit 2
}
```

## Summary

The data model for this feature is primarily in-memory state with no persistence:
- **Terminal**: xterm.js display buffer and cursor state
- **UART**: 256-byte FIFO receive buffer and 4 hardware registers
- **Emulator**: CPU state and memory map
- **App**: Component references and event coordination

All state transitions are event-driven (JavaScript CustomEvents) or callback-driven (WASM → JS for transmit). The architecture maintains clean boundaries between components while enabling bidirectional character flow.
