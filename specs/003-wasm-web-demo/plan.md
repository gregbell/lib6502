# Implementation Plan: Interactive 6502 Assembly Web Demo

**Branch**: `003-wasm-web-demo` | **Date**: 2025-11-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/home/greg/src/6502/specs/003-wasm-web-demo/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create an interactive web-based 6502 assembly playground that runs lib6502 in the browser via WebAssembly. The site features a split-panel interface with a code editor on the left and CPU state visualization (registers, flags, memory) on the right. Users can write assembly code, step through execution, run complete programs, and observe real-time CPU and memory state changes. The demo will be deployed to GitHub Pages as a static site, showcasing lib6502's capabilities without requiring installation.

## Technical Context

**Language/Version**: Rust 1.75+ (for WASM compilation), HTML5/CSS3/JavaScript ES6+ (for frontend)
**Primary Dependencies**:
- Rust: `wasm-bindgen` (JS bindings), `wasm-pack` (build tooling)
- Frontend: Zero runtime dependencies (vanilla JS), Google Fonts (Sixtyfour + JetBrains Mono)
- Build: GitHub Actions for CI/CD to GitHub Pages

**Storage**: N/A (fully client-side, no persistence)
**Testing**: `cargo test` for Rust/WASM, browser testing for frontend integration
**Target Platform**: WebAssembly in modern browsers (Chrome 57+, Firefox 52+, Safari 11+)
**Project Type**: Web application (static site with WASM module)
**Performance Goals**:
- Page load: <3s on broadband
- Step execution: <100ms per instruction
- WASM module size: <500KB (optimized for web delivery)
- Memory viewer scrolling: 60fps for smooth navigation

**Constraints**:
- Zero external dependencies in lib6502 core (maintains `no_std` compatibility)
- No server-side components (purely static deployment)
- Cross-browser WASM compatibility required
- Mobile-responsive optional (desktop-first)

**Scale/Scope**:
- Single-page application (~500-1000 lines of JS)
- 3-5 UI panels (editor, registers, flags, memory, controls)
- 2-3 example programs
- Support for programs up to ~200 lines of assembly

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Modularity & Separation of Concerns

**Status**: ✅ PASS

**Analysis**: This feature builds on top of the existing CPU core abstraction without modifying core logic. The WASM bindings will expose CPU control methods (step, run, reset) and state accessors (read registers/memory) through `wasm-bindgen`, maintaining the trait-based MemoryBus architecture. The web UI is completely decoupled from emulator internals.

**Compliance**:
- ✅ No direct memory arrays in CPU (uses existing MemoryBus trait)
- ✅ WASM layer exposes high-level API, doesn't leak CPU internals
- ✅ Frontend communicates only through defined JS API surface

### II. WebAssembly Portability

**Status**: ✅ PASS

**Analysis**: This feature directly demonstrates WASM portability by compiling lib6502 to run in the browser. The implementation uses `wasm-bindgen` which is the standard Rust→WASM bridge and maintains `no_std` compatibility.

**Compliance**:
- ✅ No OS dependencies added (existing CPU core is already WASM-compatible)
- ✅ No threading or async runtime required
- ✅ Deterministic execution (client-side only, no network/time dependencies)
- ✅ Uses established WASM toolchain (`wasm-pack`, `wasm-bindgen`)

### III. Cycle Accuracy

**Status**: ✅ PASS

**Analysis**: This feature does not modify cycle counting logic. It surfaces existing cycle counter to the UI for display, enabling users to observe timing behavior. The step/run execution model preserves cycle-accurate execution.

**Compliance**:
- ✅ No changes to cycle counting mechanisms
- ✅ Exposes cycle counter for educational visibility
- ✅ Step execution allows observing instruction-level timing

### IV. Clarity & Hackability

**Status**: ✅ PASS with considerations

**Analysis**: The web demo adds a new layer (frontend + WASM bindings) but maintains clarity through:
- Simple vanilla JS (no framework complexity)
- Clear separation between WASM API and UI
- Well-documented example for how to embed lib6502 in a web app

**Considerations**:
- Must document WASM API clearly for future embedders
- Frontend code should prioritize readability over clever optimizations
- Example programs should be educational and well-commented

**Compliance**:
- ✅ Uses standard web technologies (no framework magic)
- ✅ Serves as reference implementation for WASM embedding
- ⚠️ Requires clear documentation of WASM API surface (to be addressed in Phase 1)

### V. Table-Driven Design

**Status**: ✅ PASS (not applicable)

**Analysis**: This feature does not modify the opcode table or instruction decoder. It consumes the existing table-driven implementation through the CPU API.

**Compliance**:
- N/A - No changes to opcode handling

---

### Overall Gate Status: ✅ PASS

**Summary**: This feature aligns with all constitutional principles. It demonstrates WASM portability without compromising modularity, adds no OS dependencies, preserves cycle accuracy, and maintains project hackability. The only action item is ensuring WASM API documentation is comprehensive (addressed in Phase 1 contracts generation).

**Re-evaluation Required After Phase 1**: Verify that API contracts maintain clarity and don't leak internal CPU abstractions.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Rust library with WASM bindings
src/
├── lib.rs              # Existing CPU core
├── cpu.rs              # Core CPU implementation
├── memory.rs           # MemoryBus trait
├── opcodes.rs          # Opcode table
├── addressing.rs       # Addressing modes
├── assembler.rs        # Assembler (from feature 002)
├── disassembler.rs     # Disassembler (from feature 002)
└── wasm/               # NEW: WASM bindings module
    ├── mod.rs          # WASM module exports
    ├── api.rs          # JS-facing API (step, run, reset, getters, assemble)
    └── memory.rs       # WASM-compatible memory wrapper

# Web demo (static site)
demo/                   # NEW: GitHub Pages deployment directory
├── index.html          # Main page (split-panel layout)
├── styles.css          # CSS with Oxide-inspired design, Sixtyfour + JetBrains Mono
├── app.js              # Main application logic
├── components/         # UI component modules
│   ├── editor.js       # Code editor with syntax highlighting
│   ├── registers.js    # CPU register display
│   ├── flags.js        # Status flags display
│   ├── memory.js       # Memory viewer with navigation
│   └── controls.js     # Run/Step/Reset/Stop buttons
├── examples/           # Example assembly programs
│   ├── counter.asm     # Simple counter example
│   ├── fibonacci.asm   # Fibonacci sequence
│   └── stack-demo.asm  # Stack operations demo
└── lib6502_wasm/       # WASM build output (generated)
    ├── lib6502_wasm_bg.wasm
    ├── lib6502_wasm.js
    └── lib6502_wasm.d.ts

# Build configuration
.github/
└── workflows/
    └── deploy-demo.yml # NEW: GitHub Actions workflow for demo deployment

# Tests
tests/
├── integration/        # Existing integration tests
└── wasm/               # NEW: WASM-specific tests
    └── browser_test.rs # Browser environment tests
```

**Structure Decision**: Hybrid approach with WASM module integrated into existing Rust library and separate `demo/` directory for static web assets. The existing `src/` structure remains unchanged except for adding `src/wasm/` module. Web demo is self-contained in `demo/` for easy GitHub Pages deployment (can point to `demo/` as the Pages source directory). WASM build artifacts are generated into `demo/lib6502_wasm/` during build process.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

N/A - No constitutional violations. All principles satisfied.
