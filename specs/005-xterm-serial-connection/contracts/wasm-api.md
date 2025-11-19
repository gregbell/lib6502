# WASM API Contract: Terminal UART Integration

**Feature**: 005-xterm-serial-connection
**Date**: 2025-11-18
**Purpose**: Define the JavaScript-callable WASM API for UART terminal integration

## Overview

This contract defines the public API exposed by the Rust WASM module for terminal and UART functionality. All methods are callable from JavaScript via wasm-bindgen.

## Modified Class: `Emulator6502`

### Constructor

```typescript
/**
 * Create new emulator instance with UART terminal support
 * @param on_transmit - Callback function invoked when UART transmits a character
 * @returns New emulator instance
 * @throws JsValue error if initialization fails
 */
constructor(on_transmit: (char: string) => void): Emulator6502
```

**Behavior**:
1. Initialize `MappedMemory` (replacing `FlatMemory`)
2. Add `RamDevice` at $0000 (32KB)
3. Add `Uart6551` at $A000 (4 bytes) with transmit callback
4. Add `RomDevice` at $C000 (16KB) with reset vector → $0600
5. Create CPU instance with mapped memory
6. Store UART reference for receive access

**Memory Map After Construction**:
```
$0000-$7FFF: RAM
$A000-$A003: UART
$C000-$FFFF: ROM
```

**Example**:
```javascript
const emulator = new Emulator6502((char) => {
  terminal.write(char);
});
```

---

### New Method: `receive_char`

```typescript
/**
 * Inject received character from terminal into UART receive buffer
 * @param byte - ASCII byte value (0-255)
 * @throws None - silently handles buffer overflow by setting OVRN flag
 */
receive_char(byte: number): void
```

**Behavior**:
1. Call `uart.receive_byte(byte)`
2. Push byte to 256-byte FIFO buffer
3. Set RDRF flag (bit 3) if successful
4. Set OVRN flag (bit 2) if buffer full (byte dropped)

**Thread Safety**: Not applicable (single-threaded WASM)

**Example**:
```javascript
// User types 'A' in terminal
emulator.receive_char(0x41);
```

---

### Existing Methods (No Changes)

The following methods remain unchanged from the current implementation:

- `step(): void` - Execute single instruction
- `run_for_cycles(cycles: number): number` - Execute multiple cycles
- `reset(): void` - Reset CPU to initial state
- Getters: `a`, `x`, `y`, `pc`, `sp`, `cycles`
- Flag getters: `flag_n`, `flag_v`, `flag_d`, `flag_i`, `flag_z`, `flag_c`
- `set_pc(addr: number): void` - Set program counter
- `read_memory(addr: number): number` - Read single byte
- `write_memory(addr: number, value: number): void` - Write single byte
- `get_memory_page(page: number): Uint8Array` - Read 256-byte page
- `load_program(program: Uint8Array, start_addr: number): void` - Load program
- `assemble(source: string, start_addr: number): AssemblyResult` - Assemble code
- `assemble_and_load(source: string, start_addr: number): AssemblyResult` - Assemble and load
- `disassemble(start_addr: number, num_instructions: number): DisassemblyLine[]` - Disassemble
- Getters: `program_start`, `program_end`

---

## Callback Contract: `on_transmit`

### Function Signature

```typescript
/**
 * Callback invoked when UART transmits a character
 * @param char - Single character string (UTF-8 encoded)
 */
type TransmitCallback = (char: string) => void
```

**Invocation Timing**:
- Synchronously when 6502 writes to $A000
- Called from within `step()` or `run_for_cycles()` execution

**Thread Context**: JavaScript main thread (WASM is single-threaded)

**Error Handling**: Callback errors are not caught by WASM (will propagate to JavaScript)

**Character Encoding**:
- Input: Raw byte (0x00-0xFF)
- Output: UTF-8 string (1 character)
- Non-printable bytes: Converted to `"?"` (0x3F)

**Example**:
```javascript
const emulator = new Emulator6502((char) => {
  console.log('UART transmitted:', char);
  terminal.write(char);
});

// Later, 6502 program executes: STA $A000 (value = 0x48 'H')
// → Callback invokes: callback("H")
```

---

## Error Handling

### JsValue Errors

All errors are returned as `JsValue` (JavaScript Error objects).

**Error Scenarios**:

| Scenario | Error Message |
|----------|---------------|
| Memory device overlap | "Device overlap at address $XXXX" |
| Invalid memory access | "ExecutionError: ..." (from CPU) |
| Assembly error | "Assembly failed: [message] (line N)" |

**Example**:
```javascript
try {
  const emulator = new Emulator6502(callback);
} catch (error) {
  console.error('Failed to initialize:', error);
}
```

---

## Type Definitions (TypeScript)

```typescript
/**
 * Main emulator class exposed to JavaScript
 */
declare class Emulator6502 {
  // Constructor
  constructor(on_transmit: (char: string) => void);

  // New methods
  receive_char(byte: number): void;

  // Execution
  step(): void;
  run_for_cycles(cycles: number): number;
  reset(): void;

  // Register access
  readonly a: number;
  readonly x: number;
  readonly y: number;
  readonly pc: number;
  readonly sp: number;
  readonly cycles: number;

  // Flags
  readonly flag_n: boolean;
  readonly flag_v: boolean;
  readonly flag_d: boolean;
  readonly flag_i: boolean;
  readonly flag_z: boolean;
  readonly flag_c: boolean;

  // Program control
  set_pc(addr: number): void;
  readonly program_start: number;
  readonly program_end: number;

  // Memory access
  read_memory(addr: number): number;
  write_memory(addr: number, value: number): void;
  get_memory_page(page: number): Uint8Array;

  // Assembly
  load_program(program: Uint8Array, start_addr: number): void;
  assemble(source: string, start_addr: number): AssemblyResult;
  assemble_and_load(source: string, start_addr: number): AssemblyResult;
  disassemble(start_addr: number, num_instructions: number): DisassemblyLine[];
}

/**
 * Assembly result structure
 */
interface AssemblyResult {
  readonly success: boolean;
  readonly machine_code: Uint8Array;
  readonly start_addr: number;
  readonly end_addr: number;
  readonly error_message?: string;
  readonly error_line?: number;
}

/**
 * Disassembly line structure
 */
interface DisassemblyLine {
  readonly address: number;
  readonly bytes: Uint8Array;
  readonly mnemonic: string;
  readonly operand: string;
}
```

---

## Behavioral Contracts

### UART Register Behavior

**Reading $A000 (Data Register)**:
- Returns next byte from receive buffer (FIFO pop)
- Clears RDRF flag if buffer becomes empty
- Returns last_rx_byte if buffer was already empty

**Writing $A000 (Data Register)**:
- Stores byte in data_register
- Invokes `on_transmit` callback immediately
- TDRE flag remains set (always ready)

**Reading $A001 (Status Register)**:
- Returns current status flags:
  - Bit 4 (TDRE): Always 1
  - Bit 3 (RDRF): 1 if receive buffer not empty
  - Bit 2 (OVRN): 1 if overflow occurred since last read
- Read-only (writes ignored)

**Writing $A002 (Command Register)**:
- Bit 3: Echo mode (auto-retransmit received bytes)
- Other bits: User-defined (stored but not interpreted)

**Reading/Writing $A003 (Control Register)**:
- User-defined (stored but not interpreted)
- No effect on emulator behavior

### Reset Behavior

```typescript
reset(): void
```

**Actions**:
1. Create new CPU with fresh memory
2. **IMPORTANT**: UART state is preserved (buffer, status flags)
3. Copy existing memory contents to new memory
4. Reset PC to address from reset vector ($FFFC-$FFFD)

**Rationale**: Matches real hardware - UART peripheral state survives CPU reset

---

## Compatibility

### Backward Compatibility

**Breaking Changes**:
- Constructor signature changed (now requires `on_transmit` callback)
- Memory type changed from `FlatMemory` to `MappedMemory`

**Migration Path**:
```javascript
// Old code (no longer works)
const emulator = new Emulator6502();

// New code
const emulator = new Emulator6502((char) => {
  // Handle transmitted character
  console.log(char);
});
```

**Recommendation**: Keep old `new()` method for backward compatibility, add `new_with_uart()` variant

### Browser Compatibility

- Requires WebAssembly support (all modern browsers)
- ES6 features used (arrow functions, const/let)
- No polyfills required for target browsers (Chrome 85+, Firefox 78+, Safari 14+)

---

## Performance Characteristics

### Method Performance

| Method | Time Complexity | Notes |
|--------|----------------|-------|
| `receive_char(byte)` | O(1) | VecDeque push operation |
| `read_memory($A000)` | O(1) | VecDeque pop operation |
| `write_memory($A000)` | O(1) | Callback invocation |
| Callback invocation | O(1) | WASM→JS boundary crossing (~1-2μs) |

### Buffer Capacity

- **Receive Buffer**: 256 bytes maximum (VecDeque)
- **Overflow Behavior**: Drops new bytes, sets OVRN flag
- **No Transmit Buffer**: Immediate callback, no queuing

---

## Testing Contract

### Unit Tests (Rust)

Required test cases in `src/wasm/api.rs`:

1. **Constructor**: Verify memory map initialization
2. **receive_char**: Test buffer insertion and RDRF flag
3. **Transmit callback**: Verify callback invocation on $A000 write
4. **Buffer overflow**: Test OVRN flag on 257th byte
5. **Reset preservation**: Verify UART state survives reset

### Integration Tests (JavaScript/Browser)

Required test cases:

1. **Round-trip**: Type → receive → CPU echo → transmit → display
2. **Rapid typing**: Stress test 256-byte buffer capacity
3. **Status flags**: Verify RDRF toggles correctly
4. **Echo mode**: Test command register bit 3 functionality

---

## Example Usage

### Complete Integration Example

```javascript
import init, { Emulator6502 } from './lib6502_wasm/lib6502.js';
import { Terminal } from './components/terminal.js';

await init();

// Create terminal
const terminal = new Terminal('terminal-container');

// Create emulator with transmit callback
const emulator = new Emulator6502((char) => {
  terminal.write(char);
});

// Handle terminal input
document.addEventListener('terminal-data', (e) => {
  const data = e.detail.data;
  for (let i = 0; i < data.length; i++) {
    const byte = data.charCodeAt(i);
    emulator.receive_char(byte);
  }
});

// Load echo program
const echoProgram = `
loop:
  LDA $A001   ; Read status
  AND #$08    ; Check RDRF
  BEQ loop    ; Wait for data
  LDA $A000   ; Read byte
  STA $A000   ; Echo back
  JMP loop    ; Repeat
`;

const result = emulator.assemble_and_load(echoProgram, 0x0600);
if (result.success) {
  emulator.set_pc(0x0600);
  // Run emulator loop
  setInterval(() => {
    emulator.run_for_cycles(1000);
  }, 16); // ~60 FPS
}
```

---

## Summary

The WASM API contract defines:
- **Modified constructor**: Requires `on_transmit` callback
- **New method**: `receive_char()` for terminal input injection
- **Callback interface**: `(char: string) => void` for UART output
- **UART behavior**: 256-byte buffer, status flags, echo mode
- **Backward compatibility**: Breaking change (constructor signature)
- **Performance**: O(1) operations, <2μs callback overhead

This contract enables bidirectional communication between the JavaScript terminal and the 6502 emulator's UART device.
