# Implementation Plan: Commodore 64 Emulator Web Demo

**Branch**: `007-c64-emulator-demo` | **Date**: 2025-11-20 | **Spec**:
[spec.md](./spec.md) **Input**: Feature specification from
`/specs/007-c64-emulator-demo/spec.md`

## Summary

Build a browser-based Commodore 64 emulator demonstrating lib6502's capability
to run authentic C64 BASIC ROM code. The demo will render a 40×25 PETSCII
character display using VIC-II emulation, handle keyboard input, and boot to the
BASIC ready prompt. This validates the core's modularity principle by
implementing C64-specific components (VIC-II, CIA, C64 memory map) on top of the
generic 6502 CPU core without modifying core logic.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021) for emulator logic,
HTML5/CSS3/JavaScript ES6+ for frontend **Primary Dependencies**: wasm-bindgen
0.2, js-sys 0.3 (already in Cargo.toml), wasm-pack for builds **Storage**: N/A
(ROM binaries loaded via JavaScript fetch API, no persistence) **Testing**:
cargo test (unit tests), manual browser testing (visual/interaction validation)
**Target Platform**: WebAssembly (browser-native) targeting Chrome 85+, Firefox
78+, Safari 14+ **Project Type**: Web (Rust WASM backend + HTML/JS/CSS frontend)
**Performance Goals**:

- Boot sequence completes within 3 seconds
- Character echo appears within 100ms of keystroke
- Maintain stable 60Hz display refresh without tearing
- Handle typing at 10 characters/second without dropped keys

**Constraints**:

- Must work entirely client-side (no server communication)
- Desktop browsers only initially (mobile touch keyboard deferred)
- Text mode only (no bitmap graphics or sprites)
- No sound (SID chip excluded per spec)
- NTSC timing (60Hz) only (PAL 50Hz deferred)

**Scale/Scope**:

- Single demo page (demo/c64/index.html)
- ~1000 lines Rust (VIC-II, CIA, C64 memory map devices)
- ~500 lines JavaScript (display rendering, keyboard handling)
- 3 ROM binaries: BASIC (8KB), KERNAL (8KB), CHARGEN (4KB)

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

### ✅ Principle I: Modularity & Separation of Concerns

**Status**: PASS **Evidence**: C64-specific components (VIC-II, CIA, character
ROM) will implement the `Device` trait and plug into `MappedMemory`. CPU core
remains generic and unchanged. Memory map defined entirely by device
registration—no C64 logic bleeds into CPU implementation.

**Validation**: Existing codebase already demonstrates this pattern with UART
(src/devices/uart.rs), RAM (src/devices/ram.rs), and ROM (src/devices/rom.rs).
VIC-II and CIA will follow identical architecture.

### ✅ Principle II: WebAssembly Portability

**Status**: PASS **Evidence**: Project already has WASM target configured
(Cargo.toml line 29: `wasm = ["wasm-bindgen", "js-sys"]`). Existing demo
(demo/index.html) successfully runs lib6502 in browser. C64 demo extends
existing WASM patterns with no new OS dependencies.

**Validation**: C64 devices use only deterministic computation (memory
reads/writes, register updates, character lookups). Display rendering happens in
JavaScript canvas—no WASM-incompatible APIs required.

### ✅ Principle III: Cycle Accuracy

**Status**: PASS (with documentation caveat) **Evidence**: CPU core already
implements cycle-accurate execution (see AGENTS.md line 72-74). C64 demo will
run CPU with fixed cycle budget per frame (e.g., 20000 cycles/frame @ 60Hz ≈ 1.2
MHz, matching PAL C64 timing). VIC-II and CIA devices don't add cycle costs—they
respond to memory-mapped reads/writes synchronously.

**Caveat**: Initial implementation prioritizes functional correctness over
micro-timing (e.g., VIC-II raster interrupts deferred). This aligns with
constitution line 73: "prefer instruction-level accuracy over micro-op models
unless timing demands it."

### ✅ Principle IV: Clarity & Hackability

**Status**: PASS **Evidence**: Implementation will include extensive doc
comments explaining C64-specific behavior (e.g., VIC-II register $D011 controls
screen mode, CIA $DC00 scans keyboard matrix). Code structure mirrors hardware
(one module per chip). Examples in `examples/c64_demo.rs` will demonstrate
standalone usage.

**Validation**: Follows existing documentation patterns in src/devices/uart.rs
(15KB of doc comments explaining 6551 ACIA behavior).

### ✅ Principle V: Table-Driven Design

**Status**: PASS (not directly applicable but followed in spirit) **Evidence**:
C64-specific data (PETSCII character bitmaps, keyboard matrix mapping, VIC-II
color palette) will use const arrays and lookup tables rather than hardcoded
logic. Character rendering reads from ROM data table; keyboard scanning uses
matrix table.

**Validation**: Aligns with existing OPCODE_TABLE approach (src/opcodes.rs).

### Summary

**All constitutional principles satisfied.** No violations require
justification. Architecture naturally extends existing device-based design.
Proceed to Phase 0 research.

## Project Structure

### Documentation (this feature)

```text
specs/007-c64-emulator-demo/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output - NEEDS GENERATION
├── data-model.md        # Phase 1 output - NEEDS GENERATION
├── quickstart.md        # Phase 1 output - NEEDS GENERATION
├── contracts/           # Phase 1 output - NEEDS GENERATION
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
# Rust WASM Backend
src/
├── devices/
│   ├── mod.rs           # Existing device infrastructure
│   ├── ram.rs           # Existing RAM device
│   ├── rom.rs           # Existing ROM device
│   ├── vic2.rs          # NEW - VIC-II video chip emulation
│   ├── cia.rs           # NEW - CIA timer/keyboard chip
│   └── c64_memory.rs    # NEW - Helper for C64 memory map setup
├── lib.rs               # Existing WASM bindings (extend for C64)
└── wasm.rs              # NEW - C64-specific WASM interface

# Browser Frontend
demo/c64/
├── index.html           # NEW - C64 demo page structure
├── app.js               # NEW - Main application logic
├── styles.css           # NEW - C64 visual styling
├── components/
│   ├── display.js       # NEW - Canvas rendering for PETSCII display
│   ├── keyboard.js      # NEW - Keyboard input handling
│   └── controls.js      # NEW - UI controls (reset, load, etc.)
└── roms/
    ├── basic.bin        # C64 BASIC ROM (8KB)
    ├── kernal.bin       # C64 KERNAL ROM (8KB)
    └── chargen.bin      # C64 character ROM (4KB)

# Examples & Tests
examples/
└── c64_demo.rs          # NEW - Standalone C64 emulator example

tests/
└── c64_integration_test.rs  # NEW - C64 boot and BASIC tests
```

**Structure Decision**: Web application structure chosen because feature
requires both Rust WASM backend (emulator devices) and JavaScript frontend
(display rendering, keyboard capture). Follows existing demo/ directory pattern.
Single project architecture maintained—C64 devices live in src/devices/
alongside existing UART/RAM/ROM.

## Complexity Tracking

> **No violations—table not required.**

## Phase 0: Research & Decision Documentation

**Status**: NEEDS GENERATION **Output**: `research.md`

### Research Tasks

1. **C64 ROM Acquisition & Licensing**
   - Identify legal sources for BASIC/KERNAL/CHARGEN ROM binaries
   - Document licensing requirements (public domain? VICE project license?)
   - Alternatives: Use open-source C64 ROM replacements (JiffyDOS alternatives?)

2. **VIC-II Register Implementation Scope**
   - Research minimal VIC-II register subset for text mode
   - Identify critical registers:
     $D011 (screen control), $D018 (memory pointers), $D020/$D021
     (border/background colors)
   - Document deferred features: sprites, bitmap mode, raster interrupts

3. **CIA Timing Implementation**
   - Research CIA timer operation for keyboard scanning
   - Determine if timers need cycle-accurate countdown or polling suffices
   - Identify KERNAL timer dependencies for cursor blink

4. **PETSCII Character Rendering**
   - Evaluate rendering approaches: Canvas 2D API vs WebGL
   - Research character scaling strategies (8×8 pixel chars → display
     resolution)
   - Investigate font rendering (bitmap from CHARGEN ROM vs pre-rendered PNG
     atlas)

5. **Keyboard Matrix Mapping**
   - Document C64 keyboard matrix (8×8 grid, CIA port scanning)
   - Create modern keyboard → C64 key mapping table
   - Identify unmappable keys (C64 has £ key, modern keyboards vary)

6. **Browser Canvas Performance**
   - Benchmark Canvas 2D rendering for 40×25 character grid at 60 FPS
   - Evaluate dirty region optimization (redraw only changed characters)
   - Research double-buffering patterns for flicker-free rendering

### Decisions to Document in research.md

- ROM binary source and legal justification
- VIC-II register subset (full list with rationale for inclusions/exclusions)
- CIA implementation level (cycle-accurate vs functional)
- Rendering architecture (Canvas 2D confirmed or WebGL considered)
- Keyboard mapping strategy (table-driven with special case handling)

## Phase 1: Design Artifacts

**Status**: NEEDS GENERATION **Output**: `data-model.md`, `contracts/`,
`quickstart.md`

### Data Model (data-model.md)

Entities to document:

- **Vic2Device**: VIC-II registers ($D000-$D3FF), screen memory pointer, color
  memory, border/background colors
- **CiaDevice**: CIA registers ($DC00-$DCFF), timer state, keyboard matrix rows
- **C64Memory**: Full memory map layout (RAM regions, ROM regions, I/O area,
  bank switching logic)
- **PetsciiDisplay**: JavaScript-side display state (40×25 character grid, color
  grid, cursor position)
- **KeyboardMatrix**: 8×8 boolean matrix representing current key state

### Contracts (contracts/)

Since this is an emulator demo (not a traditional API), "contracts" = interfaces
between components:

1. **rust-to-javascript.md**: WASM interface contract
   - `Emulator::new()` → Returns emulator instance
   - `Emulator::reset()` → Performs C64 reset
   - `Emulator::run_frame()` → Executes one frame worth of cycles
   - `Emulator::get_screen_memory()` → Returns [u8; 1000] for display
   - `Emulator::get_color_memory()` → Returns [u8; 1000] for colors
   - `Emulator::key_down(row, col)` → Signals key press
   - `Emulator::key_up(row, col)` → Signals key release

2. **vic2-device.md**: VIC-II Device trait implementation
   - Register map: Offset 0x00-0x3FF → Register meanings
   - Read behavior: Color registers, screen memory pointer
   - Write behavior: Border color, background color, character ROM bank
   - Memory access: How VIC-II reads screen/character memory

3. **cia-device.md**: CIA Device trait implementation
   - Register map: Offset 0x00-0x0F → Register meanings
   - Timer behavior: Countdown, interrupt generation (deferred)
   - Keyboard scanning: Port A/B reads return matrix row state

### Quickstart (quickstart.md)

Step-by-step guide for running the C64 demo:

1. Prerequisites (Rust, wasm-pack, local web server)
2. Build WASM: `wasm-pack build --target web --features wasm`
3. Obtain ROMs (link to legal sources or instructions)
4. Place ROMs in demo/c64/roms/
5. Serve locally: `python3 -m http.server -d demo/c64 8000`
6. Open browser: `http://localhost:8000`
7. Expected behavior: Blue screen, "READY." prompt
8. Try commands: `PRINT 2+2`, `10 PRINT "HELLO"`, `RUN`

## Phase 1: Agent Context Update

**Status**: NEEDS EXECUTION **Command**:
`.specify/scripts/bash/update-agent-context.sh claude`

Will add to AGENTS.md:

- Rust 1.75+ + wasm-bindgen + js-sys + HTML5 Canvas (C64 demo)
- N/A (in-memory emulator state, no persistence) (C64 demo)

## Post-Design Constitution Re-Check

**Status**: ✅ COMPLETE (Phase 1 design validated)

### ✅ Principle I: Modularity & Separation of Concerns

**Validation**: PASS
**Evidence from Design**:
- `Vic2Device` and `CiaDevice` both implement `Device` trait (contracts/vic2-device.md, contracts/cia-device.md)
- No modifications to CPU core required—all C64 logic encapsulated in devices
- Memory map defined via `MappedMemory::add_device()` calls (data-model.md section 1.3)
- VIC-II/CIA register handling completely isolated from CPU instruction execution

**Verification**: Device contracts specify offset-based addressing (0 to size-1), maintaining device independence from absolute memory addresses.

### ✅ Principle II: WebAssembly Portability

**Validation**: PASS
**Evidence from Design**:
- No new dependencies beyond existing `wasm-bindgen` and `js-sys` (research.md section 1)
- All C64 devices use deterministic computation (register reads/writes, matrix lookups)
- WASM interface contract (contracts/rust-to-javascript.md) defines clean boundary
- No OS dependencies in VIC-II/CIA implementations (pure memory-mapped I/O)

**Verification**: Quickstart.md confirms WASM build command works with existing toolchain.

### ✅ Principle III: Cycle Accuracy

**Validation**: PASS (with documented Phase 1 approximations)
**Evidence from Design**:
- Frame timing budget: 16667 cycles/frame @ 60Hz ≈ 1.0 MHz (data-model.md section 2.1, contracts/rust-to-javascript.md)
- CIA Timer A: Functional 60Hz interrupt sufficient for Phase 1 (research.md section 3)
- VIC-II raster counter: Incrementing counter, cycle-accurate updates deferred to Phase 2 (contracts/vic2-device.md)

**Caveat**: Phase 1 prioritizes functional correctness—cycle-accurate timer countdown and raster interrupts deferred. Aligns with constitution principle: "prefer instruction-level accuracy over micro-op models unless timing demands it."

### ✅ Principle IV: Clarity & Hackability

**Validation**: PASS
**Evidence from Design**:
- Extensive documentation in all contracts (rust-to-javascript.md, vic2-device.md, cia-device.md)
- VIC-II register semantics fully documented with bit layouts and KERNAL dependencies
- CIA keyboard matrix scanning algorithm explained with examples
- Data model includes validation rules, state transitions, performance considerations

**Verification**: Contracts specify expected default values, register behavior, and KERNAL initialization sequences. Quickstart.md provides clear setup instructions.

### ✅ Principle V: Table-Driven Design

**Validation**: PASS (in spirit—appropriate for emulator domain)
**Evidence from Design**:
- PETSCII character atlas: Pre-rendered 16×16 grid lookup (research.md section 4, data-model.md section 3.2)
- Keyboard matrix: 8×8 const array mapping modern keys → C64 positions (research.md section 5, data-model.md section 2.2)
- VIC-II color palette: Fixed 16-color const array (contracts/vic2-device.md, data-model.md section 3.1)
- CIA register map: Offset-based lookup table (contracts/cia-device.md)

**Note**: Principle V primarily addresses opcode decoding (already implemented in `OPCODE_TABLE`). C64-specific data naturally follows table-driven approach through const arrays and lookup structures.

### Summary

**All constitutional principles satisfied post-design.** Phase 1 design artifacts demonstrate:
- Clean device-based architecture (Modularity)
- Zero new dependencies (WASM Portability)
- Documented cycle budgets with justified approximations (Cycle Accuracy)
- Comprehensive documentation with examples (Clarity & Hackability)
- Const arrays for PETSCII, keyboard, colors (Table-Driven in spirit)

**No violations introduced during design phase.** Architecture remains faithful to project constitution. Ready to proceed to implementation (Phase 2: `/speckit.tasks`).

## Next Steps

**STOP HERE** - `/speckit.plan` command completes after Phase 1 planning.

To continue implementation:

1. Execute Phase 0 research tasks → Generate `research.md`
2. Execute Phase 1 design → Generate `data-model.md`, `contracts/`,
   `quickstart.md`
3. Run agent context update script
4. Use `/speckit.tasks` command to generate task breakdown in `tasks.md`
5. Begin implementation following generated task list

---

**Plan Status**: ✅ COMPLETE (Phase 0 & Phase 1)

**Deliverables Generated**:
- ✅ plan.md - Implementation plan with technical context and constitution check
- ✅ research.md - Technology decisions and research findings
- ✅ data-model.md - Entity definitions and data structures
- ✅ contracts/ - Interface contracts (rust-to-javascript, vic2-device, cia-device)
- ✅ quickstart.md - Step-by-step setup and usage guide

**Next Command**: `/speckit.tasks` to generate task breakdown for implementation.
