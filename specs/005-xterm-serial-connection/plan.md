# Implementation Plan: xterm.js Serial Terminal Integration

**Branch**: `005-xterm-serial-connection` | **Date**: 2025-11-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-xterm-serial-connection/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add an interactive serial terminal to the 6502 emulator demo website using xterm.js, enabling users to write assembly programs that communicate bidirectionally with a terminal through the existing UART device implementation. The terminal will connect to the W65C51 ACIA UART at a memory-mapped address, displaying transmitted characters and injecting received characters into the UART's 256-byte buffer.

## Technical Context

**Language/Version**:
- Rust 1.75+ with wasm-bindgen for WASM compilation
- JavaScript ES6+ (vanilla, no build tools) for frontend
- HTML5/CSS3 for demo page structure

**Primary Dependencies**:
- **Backend (Rust)**: wasm-bindgen, existing lib6502 core with MappedMemory and Uart6551
- **Frontend (JavaScript)**: xterm.js (version NEEDS CLARIFICATION: recommend 5.x latest), xterm-addon-fit
- **Existing**: lib6502_wasm WASM module, existing demo components

**Storage**: N/A (in-memory emulator state only, no persistence)

**Testing**:
- Rust: cargo test (existing test infrastructure)
- JavaScript: Manual browser testing, no test framework required for initial implementation
- Integration: Browser-based functional testing with UART echo programs

**Target Platform**: Modern web browsers (Chrome 85+, Firefox 78+, Safari 14+) with WebAssembly support

**Project Type**: Web application (frontend-only demo page with WASM backend)

**Performance Goals**:
- Terminal responsiveness: <100ms echo latency during CPU execution
- CPU simulation: Maintain 1 MHz emulation speed with terminal active
- Terminal rendering: 60 FPS for smooth text display

**Constraints**:
- Must use existing UART device implementation without modification
- Must preserve current demo UI layout and functionality
- Must maintain WASM portability (no OS dependencies)
- Terminal must handle 256-byte receive buffer without overflow indicators beyond status flags

**Scale/Scope**:
- Single demo page with terminal component
- ~3-5 new UART example programs
- UART memory-mapped at single 4-byte region (NEEDS CLARIFICATION: specific address to use)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Modularity & Separation of Concerns ✅

**Status**: PASS

- UART device already implements `Device` trait with clean abstraction
- Integration via `MappedMemory` - no CPU core modifications required
- Terminal component is separate JavaScript module
- Bidirectional callbacks maintain clean boundaries (UART → terminal, terminal → UART)

### Principle II: WebAssembly Portability ✅

**Status**: PASS

- xterm.js is a browser-native library (no OS dependencies)
- WASM bindings use standard wasm-bindgen patterns
- No `std::fs`, `std::net`, or OS syscalls introduced
- Terminal integration is browser-only (doesn't break core portability)

### Principle III: Cycle Accuracy ✅

**Status**: PASS (no impact)

- No changes to CPU execution or timing logic
- UART device already implements correct status flag timing
- Terminal I/O is asynchronous to CPU execution (doesn't affect cycle counts)

### Principle IV: Clarity & Hackability ✅

**Status**: PASS

- Terminal component follows existing demo component pattern (editor.js, registers.js, etc.)
- WASM API extensions mirror existing methods (clear, documented)
- Integration example documented in existing `examples/wasm_terminal.rs`
- Assembly examples will demonstrate UART usage patterns

### Principle V: Table-Driven Design ✅

**Status**: PASS (no impact)

- No changes to opcode table or instruction decoder
- UART device uses memory-mapped registers (not new instructions)

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
# Rust WASM Backend
src/wasm/
├── api.rs               # [MODIFY] Add UART integration to Emulator6502
└── mod.rs               # WASM module exports

src/devices/
└── uart.rs              # [EXISTING] W65C51 UART device (no changes needed)

# Demo Website Frontend
demo/
├── index.html           # [MODIFY] Add terminal container element
├── app.js               # [MODIFY] Initialize terminal, connect UART callbacks
├── styles.css           # [MODIFY] Add terminal styling
├── components/
│   ├── terminal.js      # [NEW] Terminal component wrapper for xterm.js
│   ├── examples.js      # [MODIFY] Add UART example programs
│   ├── editor.js        # [EXISTING] No changes
│   ├── registers.js     # [EXISTING] No changes
│   ├── flags.js         # [EXISTING] No changes
│   ├── memory.js        # [EXISTING] No changes
│   ├── controls.js      # [EXISTING] No changes
│   └── error.js         # [EXISTING] No changes
└── examples/
    ├── uart-echo.asm    # [NEW] Character echo example
    ├── uart-hello.asm   # [NEW] Hello World via UART
    ├── uart-polling.asm # [NEW] Status register polling demo
    └── [existing .asm files - no changes]

# External Dependencies (CDN-loaded)
# - xterm.js (via CDN link in index.html)
# - xterm-addon-fit (via CDN link in index.html)
```

**Structure Decision**: Web application with Rust WASM backend and vanilla JavaScript frontend. The existing demo website structure is extended with a new terminal component and UART-specific examples. No build tools required - dependencies loaded via CDN, same as current demo architecture.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

N/A - All constitution principles are satisfied with no violations.

---

## Post-Design Constitution Re-Evaluation

*Performed after Phase 1 design completion (2025-11-18)*

### Design Review Against Principles

**Principle I: Modularity & Separation of Concerns** ✅ CONFIRMED
- Terminal component cleanly separated (demo/components/terminal.js)
- UART device integration via MappedMemory (no CPU modifications)
- Event-driven communication (CustomEvents, callbacks)
- Each component testable independently

**Principle II: WebAssembly Portability** ✅ CONFIRMED
- xterm.js loaded via CDN (browser-native, no OS deps)
- WASM bindings use standard wasm-bindgen patterns
- No new Rust dependencies requiring OS features
- Terminal is browser-only enhancement (core emulator remains portable)

**Principle III: Cycle Accuracy** ✅ CONFIRMED
- No changes to CPU timing logic
- UART device timing already correct (per src/devices/uart.rs)
- Terminal I/O is asynchronous (doesn't affect cycle counts)

**Principle IV: Clarity & Hackability** ✅ CONFIRMED
- Terminal component follows existing demo patterns (mirrors editor.js, etc.)
- Quickstart guide provides clear implementation steps
- Contracts define all APIs explicitly
- Example programs demonstrate usage patterns

**Principle V: Table-Driven Design** ✅ CONFIRMED
- No changes to opcode table or instruction decoder
- UART uses memory-mapped registers (not new opcodes)

### Architectural Integrity Verification

✅ **No CPU core changes** - All modifications in demo/ and src/wasm/api.rs
✅ **Existing Device trait used** - UART already implements proper abstraction
✅ **Memory map documented** - Clear address assignments in data-model.md
✅ **Event flow documented** - Diagrams in data-model.md show bidirectional flow
✅ **Performance constraints met** - <100ms latency achievable with current design

### Conclusion

**STATUS**: All constitution principles remain satisfied after detailed design.

No violations introduced during Phase 0 (research) or Phase 1 (design). The implementation plan preserves project architecture while adding terminal capability as a modular enhancement.
