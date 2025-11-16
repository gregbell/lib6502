# WASM API Contract

**Module**: `lib6502_wasm`
**Target**: WebAssembly (wasm32-unknown-unknown)
**Bindings**: wasm-bindgen 0.2
**JavaScript Import**: ES6 module

## Overview

This contract defines the JavaScript-accessible API surface for the lib6502 emulator compiled to WebAssembly. The API provides complete control over CPU execution, state inspection, and memory access.

## Module Initialization

### init()

**Signature**:
```typescript
function init(module_or_path?: InitInput): Promise<InitOutput>
```

**Description**: Initialize the WASM module. Must be called before using any other API.

**Parameters**:
- `module_or_path`: Optional WebAssembly module or path to `.wasm` file
- If omitted, loads from default path (`./lib6502_wasm_bg.wasm`)

**Returns**: Promise resolving to initialized module

**Example**:
```javascript
import init, { Emulator6502 } from './lib6502_wasm/lib6502_wasm.js';

await init();
const emu = new Emulator6502();
```

## Class: Emulator6502

The main emulator class providing CPU control and state access.

### Constructor

#### new Emulator6502()

**Signature**:
```typescript
constructor(): Emulator6502
```

**Description**: Creates a new 6502 CPU instance with 64KB flat memory, initialized to all zeros.

**Initial State**:
- All registers: 0x00
- Program counter: 0x0000
- Stack pointer: 0xFF
- All flags: false
- Memory: 65536 bytes of 0x00
- Cycle count: 0

**Example**:
```javascript
const emu = new Emulator6502();
```

---

### Execution Control

#### step()

**Signature**:
```typescript
step(): void
```

**Description**: Execute exactly one instruction at the current program counter.

**Behavior**:
- Fetches opcode at PC
- Decodes instruction
- Executes instruction logic
- Updates registers, flags, memory as appropriate
- Advances PC by instruction size
- Increments cycle counter by instruction cycles (including page-crossing penalties)

**Throws**:
- `Error` if opcode is unimplemented
- `Error` if execution fails

**Example**:
```javascript
emu.step(); // Execute one instruction
console.log('PC after step:', emu.get_pc());
```

---

#### run_for_cycles(cycles)

**Signature**:
```typescript
run_for_cycles(cycles: number): number
```

**Description**: Execute instructions until the specified cycle budget is exhausted.

**Parameters**:
- `cycles`: Maximum number of cycles to execute (u32, 0 to 4,294,967,295)

**Returns**: Actual number of cycles executed (may be less if CPU halts)

**Behavior**:
- Repeatedly calls step() until cycle budget reached or CPU halts
- Returns actual cycles consumed
- Useful for timed execution (e.g., "execute 1000 cycles per frame")

**Throws**:
- `Error` if execution error occurs during any instruction

**Example**:
```javascript
// Execute approximately 1000 cycles
const executed = emu.run_for_cycles(1000);
console.log('Executed', executed, 'cycles');
```

---

#### reset()

**Signature**:
```typescript
reset(): void
```

**Description**: Reset the CPU to initial power-on state.

**Behavior**:
- Resets all registers to 0x00
- Resets program counter to 0x0000
- Resets stack pointer to 0xFF
- Clears all status flags
- Resets cycle counter to 0
- **Does NOT** clear memory (preserves loaded program)

**Example**:
```javascript
emu.reset(); // Reset CPU but keep program in memory
```

**Note**: To fully clear memory, create a new instance or manually zero memory.

---

### Register Access

All register getters return the current value. Values are read-only from JavaScript (use WASM methods to modify).

#### get_a()

**Signature**: `get_a(): number`
**Returns**: Accumulator value (0x00-0xFF)

#### get_x()

**Signature**: `get_x(): number`
**Returns**: X index register value (0x00-0xFF)

#### get_y()

**Signature**: `get_y(): number`
**Returns**: Y index register value (0x00-0xFF)

#### get_pc()

**Signature**: `get_pc(): number`
**Returns**: Program counter (0x0000-0xFFFF)

#### get_sp()

**Signature**: `get_sp(): number`
**Returns**: Stack pointer (0x00-0xFF)

**Note**: Actual stack address is 0x0100 + SP (stack lives in page 1)

#### get_cycles()

**Signature**: `get_cycles(): bigint`
**Returns**: Total cycle count since reset (u64, represented as JavaScript BigInt)

**Example**:
```javascript
const state = {
    a: emu.get_a(),
    x: emu.get_x(),
    y: emu.get_y(),
    pc: emu.get_pc(),
    sp: emu.get_sp(),
    cycles: emu.get_cycles(),
};
console.log('CPU State:', state);
```

---

### Status Flag Access

All flag getters return boolean values.

#### get_flag_n()

**Signature**: `get_flag_n(): boolean`
**Returns**: Negative flag (N)
**Meaning**: Set if result of last operation has bit 7 set (signed negative)

#### get_flag_v()

**Signature**: `get_flag_v(): boolean`
**Returns**: Overflow flag (V)
**Meaning**: Set if signed arithmetic overflow occurred

#### get_flag_d()

**Signature**: `get_flag_d(): boolean`
**Returns**: Decimal mode flag (D)
**Meaning**: If true, ADC/SBC use BCD arithmetic

#### get_flag_i()

**Signature**: `get_flag_i(): boolean`
**Returns**: Interrupt disable flag (I)
**Meaning**: If true, maskable interrupts (IRQ) are disabled

#### get_flag_z()

**Signature**: `get_flag_z(): boolean`
**Returns**: Zero flag (Z)
**Meaning**: Set if result of last operation was zero

#### get_flag_c()

**Signature**: `get_flag_c(): boolean`
**Returns**: Carry flag (C)
**Meaning**: Set if last operation produced carry/borrow

**Example**:
```javascript
const flags = {
    N: emu.get_flag_n(),
    V: emu.get_flag_v(),
    D: emu.get_flag_d(),
    I: emu.get_flag_i(),
    Z: emu.get_flag_z(),
    C: emu.get_flag_c(),
};
console.log('Flags:', flags);
```

---

### Assembly & Disassembly

#### assemble(source, start_addr)

**Signature**:
```typescript
assemble(source: string, start_addr: number): AssemblyResult
```

**Description**: Assemble 6502 assembly source code into machine code.

**Parameters**:
- `source`: Assembly source code as string (supports labels, comments, standard mnemonics)
- `start_addr`: Base address for assembly (typically 0x0600)

**Returns**: `AssemblyResult` object (see below)

**Assembly Result Structure**:
```typescript
interface AssemblyResult {
    success: boolean;
    machine_code?: Uint8Array;  // Present if success=true
    errors?: AssemblyError[];    // Present if success=false
    warnings?: string[];         // Optional warnings even on success
}

interface AssemblyError {
    line: number;        // Line number (1-indexed)
    column?: number;     // Column number if available
    message: string;     // Error description
    error_type: string;  // "syntax" | "unknown_mnemonic" | "invalid_operand" | etc.
}
```

**Behavior**:
- Parses assembly source line by line
- Resolves labels and forward references
- Validates mnemonics and addressing modes
- Returns machine code on success or errors on failure
- Does NOT load code into memory (use `load_program()` separately)

**Example**:
```javascript
const source = `
    LDA #$42    ; Load immediate
    STA $1000   ; Store to memory
loop:
    JMP loop    ; Infinite loop
`;

const result = emu.assemble(source, 0x0600);

if (result.success) {
    console.log('Assembly succeeded!');
    console.log('Machine code:', result.machine_code);

    // Load and execute
    emu.load_program(result.machine_code, 0x0600);
    emu.step();
} else {
    console.error('Assembly failed:');
    result.errors.forEach(err => {
        console.error(`Line ${err.line}: ${err.message}`);
    });
}
```

**Supported Assembly Syntax**:
- All documented 6502 instructions (see opcodes.rs)
- Labels (e.g., `loop:`, `start:`)
- Comments (`;` to end of line)
- Hexadecimal constants (`$42`, `$1000`)
- Decimal constants (`42`, `4096`)
- Addressing modes:
  - Immediate: `#$42`
  - Zero Page: `$10`
  - Zero Page,X: `$10,X`
  - Zero Page,Y: `$10,Y`
  - Absolute: `$1000`
  - Absolute,X: `$1000,X`
  - Absolute,Y: `$1000,Y`
  - Indirect: `($1000)`
  - Indexed Indirect: `($10,X)`
  - Indirect Indexed: `($10),Y`
  - Relative: (calculated for branches)

**Error Types**:
- `syntax`: Malformed instruction or operand
- `unknown_mnemonic`: Invalid instruction name
- `invalid_operand`: Operand doesn't match addressing mode
- `undefined_label`: Reference to non-existent label
- `duplicate_label`: Label defined multiple times

---

#### assemble_and_load(source, start_addr)

**Signature**:
```typescript
assemble_and_load(source: string, start_addr: number): AssemblyResult
```

**Description**: Convenience method that assembles code and automatically loads it into memory on success.

**Parameters**:
- `source`: Assembly source code
- `start_addr`: Load address and PC entry point

**Returns**: Same `AssemblyResult` as `assemble()`

**Behavior**:
- Calls `assemble(source, start_addr)`
- If assembly succeeds:
  - Calls `load_program(machine_code, start_addr)`
  - Sets PC to `start_addr`
  - Returns successful result
- If assembly fails:
  - Does NOT modify memory or PC
  - Returns error result

**Example**:
```javascript
const result = emu.assemble_and_load(`
    LDA #$42
    STA $1000
`, 0x0600);

if (result.success) {
    // Code is assembled, loaded, and ready to run
    emu.step();
    console.log('A:', emu.get_a()); // 0x42
} else {
    // Show errors to user
    displayErrors(result.errors);
}
```

**Rationale**: This is the most common workflow (assemble → load → run), so providing a convenience method reduces boilerplate in the UI code.

---

#### disassemble(start_addr, num_instructions)

**Signature**:
```typescript
disassemble(start_addr: number, num_instructions: number): DisassemblyResult[]
```

**Description**: Disassemble machine code back to assembly mnemonics.

**Parameters**:
- `start_addr`: Starting memory address
- `num_instructions`: Number of instructions to disassemble

**Returns**: Array of disassembled instructions

**Disassembly Result Structure**:
```typescript
interface DisassemblyResult {
    address: number;      // Instruction address
    bytes: number[];      // Opcode and operand bytes
    mnemonic: string;     // Instruction mnemonic (e.g., "LDA")
    operand?: string;     // Operand text (e.g., "#$42", "$1000,X")
    text: string;         // Full assembly text (e.g., "LDA #$42")
}
```

**Example**:
```javascript
// Load some code
const code = new Uint8Array([0xA9, 0x42, 0x8D, 0x00, 0x10]);
emu.load_program(code, 0x0600);

// Disassemble it
const disasm = emu.disassemble(0x0600, 2);

console.log(disasm);
// [
//   {
//     address: 0x0600,
//     bytes: [0xA9, 0x42],
//     mnemonic: "LDA",
//     operand: "#$42",
//     text: "LDA #$42"
//   },
//   {
//     address: 0x0602,
//     bytes: [0x8D, 0x00, 0x10],
//     mnemonic: "STA",
//     operand: "$1000",
//     text: "STA $1000"
//   }
// ]
```

**Use Cases**:
- Displaying current instruction at PC in debugger
- Memory viewer enhancement (show disassembly alongside hex)
- Verification that assembly produced expected machine code

---

### Memory Access

#### read_memory(addr)

**Signature**:
```typescript
read_memory(addr: number): number
```

**Description**: Read a single byte from memory.

**Parameters**:
- `addr`: Memory address (0x0000-0xFFFF, wraps if > 0xFFFF)

**Returns**: Byte value at address (0x00-0xFF)

**Example**:
```javascript
const value = emu.read_memory(0x0600);
console.log('Value at $0600:', value.toString(16));
```

---

#### write_memory(addr, value)

**Signature**:
```typescript
write_memory(addr: number, value: number): void
```

**Description**: Write a single byte to memory.

**Parameters**:
- `addr`: Memory address (0x0000-0xFFFF)
- `value`: Byte value to write (0x00-0xFF, truncated to u8)

**Example**:
```javascript
emu.write_memory(0x1000, 0x42);
```

---

#### get_memory_page(page)

**Signature**:
```typescript
get_memory_page(page: number): Uint8Array
```

**Description**: Read an entire 256-byte page of memory efficiently.

**Parameters**:
- `page`: Page number (0-255), where page N spans addresses N*256 to N*256+255

**Returns**: Uint8Array of 256 bytes

**Rationale**: More efficient than 256 individual read_memory() calls for bulk transfer.

**Example**:
```javascript
// Read page 6 (addresses 0x0600-0x06FF)
const page = emu.get_memory_page(6);
console.log('First byte of page 6:', page[0].toString(16));
```

---

#### load_program(program, start_addr)

**Signature**:
```typescript
load_program(program: Uint8Array, start_addr: number): void
```

**Description**: Load assembled machine code into memory and set PC to entry point.

**Parameters**:
- `program`: Assembled machine code bytes
- `start_addr`: Memory address to load program (typically 0x0600)

**Behavior**:
- Writes each byte of `program` sequentially starting at `start_addr`
- Wraps addresses if program extends beyond 0xFFFF
- Sets program counter to `start_addr`
- Does NOT reset registers or flags (use reset() if needed)

**Example**:
```javascript
// LDA #$42 ; STA $1000
const program = new Uint8Array([0xA9, 0x42, 0x8D, 0x00, 0x10]);
emu.load_program(program, 0x0600);

console.log('PC:', emu.get_pc()); // 0x0600
emu.step(); // Execute LDA #$42
console.log('A:', emu.get_a());   // 0x42
```

---

## Error Handling

All methods that can fail throw JavaScript `Error` objects with descriptive messages.

### Error Types

#### Unimplemented Opcode

**Thrown by**: `step()`, `run_for_cycles()`
**Message Format**: `"Unimplemented opcode: 0xXX at PC 0xYYYY"`
**Cause**: Attempted to execute an opcode not yet implemented

**Example**:
```javascript
try {
    emu.step();
} catch (e) {
    console.error('Execution error:', e.message);
    // "Unimplemented opcode: 0x02 at PC 0x0600"
}
```

#### Invalid State

**Thrown by**: Various methods
**Message Format**: Descriptive text
**Cause**: Internal emulator state inconsistency (should not occur in normal use)

---

## Performance Characteristics

### Method Call Overhead

Approximate overhead for JS↔WASM boundary crossing:

| Method | Overhead | Use Case |
|--------|----------|----------|
| `step()` | ~0.1μs | Interactive stepping |
| `run_for_cycles(1000)` | ~10μs | Batch execution (recommended) |
| `get_a()` (single register) | ~0.05μs | UI updates (acceptable) |
| `get_memory_page()` | ~2μs | Memory viewer (much faster than 256× read_memory()) |
| `read_memory()` | ~0.05μs | Spot checks |

### Recommendations

1. **Batch execution**: Use `run_for_cycles(N)` for smooth animation (e.g., N=1000)
2. **Page-based memory**: Use `get_memory_page()` for memory viewer, not individual reads
3. **Minimize calls**: Fetch all registers in one burst, not scattered throughout frame
4. **RequestAnimationFrame**: Update UI at 60fps, not tied to instruction execution

**Anti-pattern**:
```javascript
// DON'T: Call step() in tight loop
for (let i = 0; i < 1000; i++) {
    emu.step(); // 1000 JS↔WASM boundary crossings
}
```

**Recommended pattern**:
```javascript
// DO: Batch execution in WASM
emu.run_for_cycles(1000); // Single boundary crossing
```

---

## Type Definitions (TypeScript)

```typescript
declare module 'lib6502_wasm' {
    export default function init(module_or_path?: InitInput): Promise<InitOutput>;

    export class Emulator6502 {
        constructor();

        // Execution
        step(): void;
        run_for_cycles(cycles: number): number;
        reset(): void;

        // Registers
        get_a(): number;
        get_x(): number;
        get_y(): number;
        get_pc(): number;
        get_sp(): number;
        get_cycles(): bigint;

        // Flags
        get_flag_n(): boolean;
        get_flag_v(): boolean;
        get_flag_d(): boolean;
        get_flag_i(): boolean;
        get_flag_z(): boolean;
        get_flag_c(): boolean;

        // Assembly & Disassembly
        assemble(source: string, start_addr: number): AssemblyResult;
        assemble_and_load(source: string, start_addr: number): AssemblyResult;
        disassemble(start_addr: number, num_instructions: number): DisassemblyResult[];

        // Memory
        read_memory(addr: number): number;
        write_memory(addr: number, value: number): void;
        get_memory_page(page: number): Uint8Array;
        load_program(program: Uint8Array, start_addr: number): void;
    }

    // Assembly result types
    export interface AssemblyResult {
        success: boolean;
        machine_code?: Uint8Array;
        errors?: AssemblyError[];
        warnings?: string[];
    }

    export interface AssemblyError {
        line: number;
        column?: number;
        message: string;
        error_type: string;
    }

    export interface DisassemblyResult {
        address: number;
        bytes: number[];
        mnemonic: string;
        operand?: string;
        text: string;
    }
}
```

---

## Example: Complete Usage

### Example 1: Using the Assembler

```javascript
import init, { Emulator6502 } from './lib6502_wasm/lib6502_wasm.js';

async function main() {
    // Initialize WASM module
    await init();

    // Create emulator instance
    const emu = new Emulator6502();

    // Write assembly code as a string
    const source = `
        ; Simple demo program
        LDA #$42        ; Load 0x42 into accumulator
        STA $1000       ; Store to memory
        LDX #$00        ; Initialize counter
    loop:
        INX             ; Increment X
        CPX #$10        ; Compare with 16
        BNE loop        ; Loop if not equal
        BRK             ; Done
    `;

    // Assemble and load in one step
    const result = emu.assemble_and_load(source, 0x0600);

    if (!result.success) {
        console.error('Assembly failed:');
        result.errors.forEach(err => {
            console.error(`  Line ${err.line}: ${err.message}`);
        });
        return;
    }

    console.log('Assembly successful!');
    console.log('Machine code size:', result.machine_code.length, 'bytes');

    // Execute instruction by instruction
    console.log('\nInitial state:');
    console.log('  PC:', emu.get_pc().toString(16));
    console.log('  A:', emu.get_a().toString(16));

    emu.step(); // Execute LDA #$42
    console.log('\nAfter LDA #$42:');
    console.log('  A:', emu.get_a().toString(16)); // 42
    console.log('  Z:', emu.get_flag_z());         // false
    console.log('  N:', emu.get_flag_n());         // false

    emu.step(); // Execute STA $1000
    console.log('\nAfter STA $1000:');
    console.log('  Memory[$1000]:', emu.read_memory(0x1000).toString(16)); // 42

    // Run the loop (100 cycles should be enough)
    const cycles = emu.run_for_cycles(100);
    console.log('\nExecuted', cycles, 'cycles');
    console.log('Final X:', emu.get_x().toString(16)); // 10 (16 decimal)
}

main();
```

### Example 2: Loading Machine Code Directly

```javascript
import init, { Emulator6502 } from './lib6502_wasm/lib6502_wasm.js';

async function main() {
    await init();
    const emu = new Emulator6502();

    // Load pre-assembled machine code
    const program = new Uint8Array([
        0xA9, 0x42,       // LDA #$42
        0x8D, 0x00, 0x10, // STA $1000
        0x00              // BRK
    ]);
    emu.load_program(program, 0x0600);

    // Verify with disassembler
    const disasm = emu.disassemble(0x0600, 3);
    console.log('Disassembly:');
    disasm.forEach(instr => {
        const bytes = instr.bytes.map(b => b.toString(16).padStart(2, '0')).join(' ');
        console.log(`  ${instr.address.toString(16).padStart(4, '0')}  ${bytes.padEnd(12)}  ${instr.text}`);
    });
    // Output:
    // 0600  a9 42        LDA #$42
    // 0602  8d 00 10     STA $1000
    // 0605  00           BRK

    emu.run_for_cycles(100);
}

main();
```

### Example 3: Error Handling

```javascript
import init, { Emulator6502 } from './lib6502_wasm/lib6502_wasm.js';

async function main() {
    await init();
    const emu = new Emulator6502();

    // Attempt to assemble invalid code
    const badSource = `
        LDA #$42
        INVALID_OP      ; This will cause an error
        STA $1000
    `;

    const result = emu.assemble(badSource, 0x0600);

    if (!result.success) {
        console.log('Assembly failed as expected:');
        result.errors.forEach(err => {
            console.log(`  Line ${err.line}: [${err.error_type}] ${err.message}`);
        });
        // Output:
        // Line 3: [unknown_mnemonic] Unknown instruction: INVALID_OP
    } else {
        console.log('Unexpected success!');
    }
}

main();
```

---

## Versioning

**Current Version**: 1.0.0 (initial release)

**Compatibility Promise**:
- All documented methods are part of the stable API
- Method signatures will not change in minor/patch versions
- New methods may be added in minor versions
- Breaking changes only in major versions

**Changelog**:
- 1.0.0 (2025-11-16): Initial WASM API release
