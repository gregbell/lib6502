# Data Model: WASM Web Demo

**Date**: 2025-11-16
**Feature**: Interactive 6502 Assembly Web Demo

## Overview

This document defines the data structures and state management for the web demo. The demo is stateless (no persistence), with all state held in-memory in the browser.

## Core Entities

### 1. CPU State

**Description**: Complete snapshot of the 6502 CPU at any point in time

**Structure**:
```typescript
interface CPUState {
    // Registers (8-bit except PC)
    a: number;           // Accumulator (0x00-0xFF)
    x: number;           // X Index Register (0x00-0xFF)
    y: number;           // Y Index Register (0x00-0xFF)
    sp: number;          // Stack Pointer (0x00-0xFF)
    pc: number;          // Program Counter (0x0000-0xFFFF)

    // Status Flags (boolean)
    flag_n: boolean;     // Negative
    flag_v: boolean;     // Overflow
    flag_d: boolean;     // Decimal mode
    flag_i: boolean;     // Interrupt disable
    flag_z: boolean;     // Zero
    flag_c: boolean;     // Carry

    // Execution State
    cycles: number;      // Total cycles executed (u64)
    halted: boolean;     // Whether CPU has halted (BRK or error)
}
```

**Source of Truth**: WASM emulator instance

**Access Pattern**: Read-only from UI (fetched via WASM getters)

**Validation Rules**:
- All register values must be within valid ranges (enforced by Rust)
- Flags are strictly boolean
- Cycle count is monotonically increasing (resets only on CPU reset)

### 2. Memory State

**Description**: 64KB address space of the 6502

**Structure**:
```typescript
interface MemoryState {
    data: Uint8Array;    // 65536 bytes (0x0000-0xFFFF)
    dirty: Set<number>;  // Addresses modified since last render
}
```

**Source of Truth**: WASM emulator memory (MemoryBus implementation)

**Access Patterns**:
- **Read**: Page-based (256 bytes) for efficient transfer from WASM
- **Write**: Individual bytes during program load
- **Display**: Virtual scrolling shows 16 bytes/row (4096 rows total)

**Validation Rules**:
- Address must be 0x0000-0xFFFF (wraps on overflow per 6502 spec)
- All byte values 0x00-0xFF (enforced by u8 type)

**State Transitions**:
- Initial state: All zeros (0x00)
- Modified by: Program load, instruction execution (STA, etc.), stack operations
- Reset behavior: Returns to all zeros

### 3. Assembly Program

**Description**: User-written 6502 assembly source code

**Structure**:
```typescript
interface AssemblyProgram {
    source: string;          // Raw assembly text
    machineCode?: Uint8Array; // Assembled bytes (if assembly succeeded)
    loadAddress: number;     // Where to load in memory (typically 0x0600)
    entryPoint: number;      // Initial PC value (typically = loadAddress)
    errors?: AssemblyError[]; // Assembly errors if any
}

interface AssemblyError {
    line: number;           // Line number (1-indexed)
    message: string;        // Error description
    column?: number;        // Column position (optional)
}
```

**Source of Truth**: Browser DOM (textarea content)

**Access Pattern**: Read on demand when user clicks Run/Step

**Validation Rules**:
- Source text must be valid UTF-8
- Load address must be 0x0000-0xFFFF
- Entry point must be within valid address range
- Machine code length must not exceed 65536 bytes

**State Transitions**:
1. **Empty** → User types code → **Dirty source**
2. **Dirty source** → User clicks Run → **Assembling**
3. **Assembling** → Assembly succeeds → **Ready to execute**
4. **Assembling** → Assembly fails → **Error state** (show errors)

### 4. Example Program

**Description**: Pre-written assembly programs for quick start

**Structure**:
```typescript
interface ExampleProgram {
    id: string;          // Unique identifier (e.g., "counter", "fibonacci")
    name: string;        // Display name
    description: string; // Brief explanation
    source: string;      // Assembly source code
    expectedOutput?: {   // Optional: what to expect after execution
        cycles?: number;
        registers?: Partial<CPUState>;
        memory?: { addr: number; value: number }[];
    };
}
```

**Source of Truth**: Static data embedded in JS or loaded from `.asm` files

**Access Pattern**: Read-only, loaded into editor on button click

**Examples**:
```javascript
const EXAMPLES = [
    {
        id: 'counter',
        name: 'Simple Counter',
        description: 'Counts from 0 to 255 in accumulator',
        source: `; Simple counter demo
        LDA #$00    ; Start at 0
loop:   CLC         ; Clear carry
        ADC #$01    ; Add 1
        JMP loop    ; Repeat forever`,
        expectedOutput: {
            cycles: 10000, // After 10k cycles
            registers: { a: 255 } // Wraps at 255
        }
    },
    // ... more examples
];
```

### 5. Execution State

**Description**: Current state of the demo application

**Structure**:
```typescript
interface ExecutionState {
    mode: 'idle' | 'running' | 'stepping' | 'error';
    isPaused: boolean;
    currentProgram: AssemblyProgram | null;
    lastError: string | null;
    executionSpeed: number; // Cycles per animation frame (for Run mode)
}
```

**Source of Truth**: JavaScript application state

**State Transitions**:
```
idle
  ↓ [Load program]
idle (with program)
  ↓ [Click Step]
stepping → execute 1 instruction → idle
  ↓ [Click Run]
running → execute N cycles/frame → running
  ↓ [Click Stop]
idle
  ↓ [Error occurs]
error → display error → idle (on Reset)
```

**Validation Rules**:
- Cannot step/run when in error state
- Cannot run without a loaded program
- Execution speed must be > 0

## Data Flow

### Program Load Flow

```
User types code in editor
  ↓
[Click Run/Step]
  ↓
Extract source text from DOM
  ↓
Call assembler (via WASM)
  ↓
Assembly succeeds?
  ├─ Yes → Load machine code into emulator memory
  │         Set PC to entry point
  │         Update UI state to 'ready'
  └─ No  → Display assembly errors
           Update UI state to 'error'
```

### Execution Flow

```
User clicks Step:
  emulator.step() → executes 1 instruction
  ↓
  Update CPU state display (fetch all registers)
  Update memory viewer (fetch changed pages)
  Update cycle counter

User clicks Run:
  while (running && !halted):
    emulator.run_for_cycles(1000) → batch execution
    ↓
    requestAnimationFrame:
      Update CPU state display
      Update memory viewer
      Check for halt/error
    ↓
    Repeat until Stop clicked or error
```

### Memory Update Flow

```
Instruction executes (e.g., STA $1000)
  ↓
WASM updates memory internally
  ↓
UI timer (60fps):
  Fetch affected memory page(s)
  Compare with cached memory
  Mark dirty bytes
  ↓
  Memory viewer re-renders only visible rows
  Highlight dirty bytes (flash animation)
  ↓
  After 1s: clear dirty flags
```

## UI State Management

### Component State

```javascript
class DemoApp {
    constructor() {
        // WASM instance
        this.emulator = null;

        // UI components
        this.editor = new CodeEditor('#asm-editor');
        this.registers = new RegisterDisplay('#registers');
        this.flags = new FlagsDisplay('#flags');
        this.memory = new MemoryViewer('#memory');
        this.controls = new ControlPanel('#controls');

        // Application state
        this.state = {
            mode: 'idle',
            currentProgram: null,
            memoryCache: new Uint8Array(65536),
        };
    }

    async init() {
        await init(); // Initialize WASM
        this.emulator = new Emulator6502();
        this.setupEventListeners();
        this.startUpdateLoop();
    }

    startUpdateLoop() {
        const update = () => {
            if (this.state.mode === 'running') {
                // Execute batch of instructions
                try {
                    this.emulator.run_for_cycles(1000);
                } catch (e) {
                    this.handleError(e);
                    return;
                }
            }

            // Update UI displays
            this.updateCPUDisplay();
            this.updateMemoryDisplay();

            requestAnimationFrame(update);
        };
        requestAnimationFrame(update);
    }

    updateCPUDisplay() {
        const state = {
            a: this.emulator.get_a(),
            x: this.emulator.get_x(),
            y: this.emulator.get_y(),
            pc: this.emulator.get_pc(),
            sp: this.emulator.get_sp(),
            flag_n: this.emulator.get_flag_n(),
            // ... other flags
            cycles: this.emulator.get_cycles(),
        };

        this.registers.update(state);
        this.flags.update(state);
    }

    updateMemoryDisplay() {
        // Fetch visible memory pages only
        const visibleRange = this.memory.getVisibleRange();
        for (let page of visibleRange.pages) {
            const pageData = this.emulator.get_memory_page(page);
            this.memory.updatePage(page, pageData);
        }
    }
}
```

## Performance Considerations

### Memory Access Optimization

- **Problem**: Fetching all 64KB every frame (60fps) = 3.8 MB/s transfer
- **Solution**: Page-based access (256 bytes/page), fetch only visible pages
- **Result**: ~25 pages visible × 256 bytes × 60fps = 384 KB/s (10x reduction)

### Dirty Byte Tracking

- **Client-side diffing**: Compare new page data with cached data
- **Only highlight changed bytes**: Avoid full re-render
- **Timeout clearing**: Auto-clear highlights after 1s (user feedback)

### Execution Batching

- **Problem**: Individual step() calls have overhead (JS↔WASM boundary crossing)
- **Solution**: Batch execution with run_for_cycles(N)
- **Result**: 1 WASM call per frame instead of N calls

## Error Handling

### Assembly Errors

```typescript
interface AssemblyError {
    type: 'syntax' | 'unknown_mnemonic' | 'invalid_operand';
    line: number;
    message: string;
}
```

**Display**: Highlight error line in editor, show error message below editor

### Runtime Errors

```typescript
interface RuntimeError {
    type: 'unimplemented_opcode' | 'halt';
    pc: number;        // Program counter when error occurred
    opcode?: number;   // Opcode that caused error
    message: string;
}
```

**Display**: Show error modal, highlight PC in memory viewer, disable controls until reset

## Data Persistence

**None** - This demo is fully ephemeral. All state is lost on page refresh.

**Rationale**: Aligns with "simple, minimal" design goal. Persistence would require:
- LocalStorage implementation
- Serialization/deserialization logic
- UI for save/load management
- Privacy considerations

**Future Enhancement**: Could add localStorage in a future iteration without changing core data model.

## Summary

| Entity | Size | Source of Truth | Update Frequency |
|--------|------|----------------|------------------|
| CPU State | ~32 bytes | WASM | 60fps (running mode) or on-demand (stepping) |
| Memory | 64KB | WASM | On-demand (page-based, visible pages only) |
| Assembly Source | Variable | DOM (textarea) | On user input |
| Examples | ~10KB total | Static JS | Once on page load |
| UI State | <1KB | JavaScript | On user interaction |

**Total Runtime Memory**: <100KB (excluding WASM module itself)
