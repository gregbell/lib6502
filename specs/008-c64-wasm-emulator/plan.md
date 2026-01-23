# Implementation Plan: Commodore 64 WASM Emulator

**Branch**: `008-c64-wasm-emulator` | **Date**: 2025-01-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-c64-wasm-emulator/spec.md`

## Summary

Build a fully functional Commodore 64 emulator running in the browser via WebAssembly, using the existing lib6502 CPU core as foundation. The emulator will include VIC-II graphics chip, SID audio chip, both CIA timer/I/O chips, 1541 disk drive emulation, and C64 memory banking - all implemented as memory-mapped devices following the existing Device trait pattern. The web frontend extends the existing demo infrastructure with C64-specific display canvas, audio integration, and file loading.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021), JavaScript ES6+, HTML5/CSS3
**Primary Dependencies**: wasm-bindgen, web-sys (WASM bindings), existing lib6502 core (zero external deps)
**Storage**: Browser localStorage for ROM caching; in-memory emulator state; File API for .D64/.PRG/.snapshot files
**Testing**: cargo test (unit/integration), Klaus Dormann functional test, browser manual testing
**Target Platform**: WASM in modern browsers (Chromium, Firefox, Safari/WebKit 2018+)
**Project Type**: Web application with Rust WASM core + JavaScript frontend
**Performance Goals**: 50/60 fps (PAL/NTSC), real-time audio (44.1/48kHz), <50ms input latency
**Constraints**: No external Rust dependencies in core library; ROM files must be user-provided; ~2 second boot time
**Scale/Scope**: Single-user browser app; 90%+ compatibility with top 100 C64 games; 6 major chip emulations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status | Notes |
|-----------|-------------|--------|-------|
| I. Modularity & Separation | CPU core decoupled from machine implementation; all access via trait-based bus | ✅ PASS | VIC-II, SID, CIA chips implement Device trait; CPU uses MemoryBus abstraction |
| II. WebAssembly Portability | Pure Rust, no OS dependencies, no_std compatible | ✅ PASS | Core library remains zero-dependency; WASM bindings via wasm-bindgen (existing pattern) |
| III. Cycle Accuracy | Correct cycle totals, page-crossing, branch timing | ⚠️ PARTIAL | CPU core is cycle-accurate; VIC-II is frame-accurate (not cycle-exact) per spec requirement |
| IV. Clarity & Hackability | Readable code, doc comments, easy to extend | ✅ PASS | Each chip is self-contained module with clear public API |
| V. Table-Driven Design | Opcode metadata in single table | ✅ PASS | Existing OPCODE_TABLE unchanged; new chips have register tables |

**Gate Evaluation**: ✅ PASS - All principles satisfied. VIC-II frame accuracy explicitly permitted by spec (FR-020: "frame-accurate level, not cycle-exact").

## Project Structure

### Documentation (this feature)

```text
specs/008-c64-wasm-emulator/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Existing Rust core library (extended)
src/
├── lib.rs                    # Public API exports (add C64 modules)
├── cpu.rs                    # Existing CPU (add NMI support)
├── memory.rs                 # Existing MemoryBus trait
├── devices/
│   ├── mod.rs                # Existing Device trait, MappedMemory
│   ├── ram.rs                # Existing RamDevice
│   ├── rom.rs                # Existing RomDevice
│   ├── uart.rs               # Existing Uart6551
│   └── c64/                  # NEW: C64-specific devices
│       ├── mod.rs            # C64 device exports
│       ├── vic_ii.rs         # VIC-II video chip (MOS 6569)
│       ├── sid.rs            # SID audio chip (MOS 6581)
│       ├── cia.rs            # CIA timer chip (MOS 6526)
│       ├── port_6510.rs      # 6510 I/O port ($00-$01)
│       ├── color_ram.rs      # 1KB Color RAM
│       └── memory_map.rs     # C64 bank switching logic
├── c64/                      # NEW: C64 system integration
│   ├── mod.rs                # C64System struct, clock timing
│   ├── keyboard.rs           # Keyboard matrix scanning
│   ├── joystick.rs           # Joystick port emulation
│   └── disk_1541.rs          # 1541 drive IEC protocol
└── wasm/
    ├── api.rs                # Existing WASM bindings (extend for C64)
    └── c64_api.rs            # NEW: C64-specific WASM API

# Existing web demo (extended for C64)
demo/                         # Existing interactive demo
c64-demo/                     # NEW: C64 emulator frontend
├── index.html                # C64 emulator page
├── style.css                 # C64-themed styling
├── c64.js                    # Main C64 application
├── components/
│   ├── screen.js             # VIC-II canvas renderer
│   ├── audio.js              # SID Web Audio integration
│   ├── keyboard.js           # Keyboard matrix mapping
│   ├── joystick.js           # Gamepad API + keyboard joystick
│   ├── file-loader.js        # .D64, .PRG, ROM file handling
│   ├── rom-manager.js        # ROM validation & localStorage
│   └── controls.js           # Reset, pause, settings panel
└── tests/                    # Browser integration tests

# Test suite
tests/
├── c64_vic_ii_tests.rs       # VIC-II chip tests
├── c64_sid_tests.rs          # SID chip tests
├── c64_cia_tests.rs          # CIA chip tests
├── c64_memory_tests.rs       # Bank switching tests
├── c64_integration_tests.rs  # Full system tests
└── functional_klaus.rs       # Existing Klaus test (unchanged)
```

**Structure Decision**: Hybrid single-project Rust library with web frontend. C64-specific devices added under `src/devices/c64/` following existing device patterns. C64 system integration in `src/c64/`. New `c64-demo/` directory mirrors existing `demo/` structure but tailored for C64.

## Complexity Tracking

> No violations - all complexity justified by C64 hardware requirements.

| Component | Complexity | Justification |
|-----------|------------|---------------|
| VIC-II (47 registers) | Medium | Required for graphics; well-documented chip |
| SID (29 registers) | Medium | Required for audio; filter emulation is complex but bounded |
| CIA × 2 (16 registers each) | Low | Timer/I/O chips; straightforward register model |
| Memory banking | Medium | C64 fundamental; clean state machine design |
| 1541 drive | Medium | High-level IEC protocol only; no drive CPU emulation |

---

## Constitution Check (Post-Design)

*Re-evaluation after Phase 1 design completion.*

| Principle | Post-Design Assessment | Status |
|-----------|------------------------|--------|
| I. Modularity & Separation | VIC-II, SID, CIA, Port6510 each implement Device trait independently. C64Memory orchestrates banking without modifying CPU. | ✅ PASS |
| II. WebAssembly Portability | All chip implementations use pure Rust with no external dependencies. WASM API uses existing wasm-bindgen pattern. No OS-specific code. | ✅ PASS |
| III. Cycle Accuracy | CPU maintains full cycle accuracy. VIC-II uses scanline-based rendering (63 cycles/line PAL). Explicitly frame-accurate per FR-020. | ✅ PASS |
| IV. Clarity & Hackability | Each device has clear register map in data-model.md. Quickstart provides copy-paste examples. Device files are self-contained. | ✅ PASS |
| V. Table-Driven Design | VIC-II, SID, CIA all use register arrays. D64 uses track/sector tables. No scattered decode logic. | ✅ PASS |

**Final Gate Status**: ✅ ALL PRINCIPLES SATISFIED

---

## Generated Artifacts

| Artifact | Path | Description |
|----------|------|-------------|
| Research | `specs/008-c64-wasm-emulator/research.md` | Technical decisions and reference material |
| Data Model | `specs/008-c64-wasm-emulator/data-model.md` | Entity definitions, register maps, state schemas |
| WASM API | `specs/008-c64-wasm-emulator/contracts/wasm-api.md` | JavaScript/TypeScript API contract |
| Quickstart | `specs/008-c64-wasm-emulator/quickstart.md` | Developer onboarding guide |

---

## Next Steps

Run `/speckit.tasks` to generate `tasks.md` with implementation tasks based on this plan.
