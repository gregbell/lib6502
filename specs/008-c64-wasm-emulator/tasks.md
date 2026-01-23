# Tasks: Commodore 64 WASM Emulator

**Input**: Design documents from `/specs/008-c64-wasm-emulator/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/wasm-api.md

**Tests**: Not explicitly requested in feature specification. Tests are omitted but can be added later.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Project Infrastructure)

**Purpose**: Project initialization, dependencies, and basic structure

**NOTE**: Implemented as separate `c64-emu` sub-crate instead of modules under `src/` to keep C64 code isolated from the core lib6502 library per user requirement.

- [x] T001 Create C64 device module structure ~~in src/devices/c64/mod.rs~~ → c64-emu/src/devices/mod.rs
- [x] T002 [P] Create C64 system module structure ~~in src/c64/mod.rs~~ → c64-emu/src/system/mod.rs
- [x] T003 [P] Create C64 WASM API module ~~in src/wasm/c64_api.rs~~ → c64-emu/src/wasm.rs
- [ ] T004 [P] Create c64-demo directory structure with index.html, style.css, c64.js
- [x] T005 Add wasm-bindgen and web-sys dependencies to Cargo.toml for WASM target → c64-emu/Cargo.toml
- [x] T006 Export C64 modules ~~from src/lib.rs~~ → c64-emu/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### CPU Extensions

- [x] T007 Add NMI support to CPU (nmi_pending flag, nmi_active method, NMI vector $FFFA) in src/cpu.rs
- [x] T008 Implement 6510 I/O port device (Port6510 struct, $00-$01 mapping, bank bits) → c64-emu/src/devices/port_6510.rs

### Memory Architecture

- [x] T009 Implement C64Memory struct with 64KB RAM, ROM slots, and bank switching → c64-emu/src/system/c64_memory.rs
- [x] T010 Implement ROM loading (validate sizes: BASIC=8KB, KERNAL=8KB, CHARROM=4KB) → c64-emu/src/system/c64_memory.rs
- [x] T011 Implement bank switching logic (8 configurations based on $01 bits 0-2) → c64-emu/src/system/c64_memory.rs
- [x] T012 Implement MemoryBus trait for C64Memory with proper ROM/RAM/I/O routing → c64-emu/src/system/c64_memory.rs

### VIC-II Core

- [x] T013 Create VicII struct with 47 registers, framebuffer, scanline state → c64-emu/src/devices/vic_ii.rs
- [x] T014 Implement Device trait for VicII (read/write registers, register mirroring) → c64-emu/src/devices/vic_ii.rs
- [x] T015 Implement VIC-II bank selection via CIA2 port A → c64-emu/src/system/c64_memory.rs (via vic_bank())

### SID Core

- [x] T016 Create Sid6581 struct with 3 voices, filter, sample buffer → c64-emu/src/devices/sid.rs
- [x] T017 Implement Device trait for Sid6581 (29 registers, read-only registers) → c64-emu/src/devices/sid.rs

### CIA Core

- [x] T018 Create Cia6526 struct with ports, timers, TOD, interrupt handling → c64-emu/src/devices/cia.rs
- [x] T019 Implement Device trait for Cia6526 (16 registers with mirroring) → c64-emu/src/devices/cia.rs
- [x] T020 Implement CIA timer countdown and underflow logic → c64-emu/src/devices/cia.rs
- [x] T021 Implement CIA interrupt generation (IRQ for CIA1, NMI for CIA2) → c64-emu/src/devices/cia.rs

### Color RAM

- [x] T022 Implement 1KB color RAM device ($D800-$DBFF, 4-bit per cell) → c64-emu/src/devices/color_ram.rs

### C64 System

- [x] T023 Create C64System struct orchestrating CPU, memory, and timing → c64-emu/src/system/c64_system.rs
- [x] T024 Implement Region enum (PAL/NTSC) with clock speeds and scanline counts → c64-emu/src/system/c64_system.rs
- [x] T025 Implement step_frame() executing one full frame worth of cycles → c64-emu/src/system/c64_system.rs
- [x] T026 Implement reset() and hard_reset() for system initialization → c64-emu/src/system/c64_system.rs

### WASM Core API

- [x] T027 Implement C64Emulator constructor and load_roms() with validation → c64-emu/src/wasm.rs
- [x] T028 Implement step_frame(), reset(), set_region() → c64-emu/src/wasm.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Run Classic C64 Programs (Priority: P1) 🎯 MVP

**Goal**: Boot to BASIC prompt, type programs, see output on screen

**Independent Test**: Load emulator, see "COMMODORE 64 BASIC V2" and "READY." prompt, type `10 PRINT "HELLO": GOTO 10`, run, verify output displays

### VIC-II Rendering

- [x] T029 [US1] Implement standard text mode rendering (40x25 chars, 8x8 font) in c64-emu/src/devices/vic_ii.rs
- [x] T030 [US1] Implement character ROM lookup for text display in c64-emu/src/devices/vic_ii.rs
- [x] T031 [US1] Implement scanline-based frame rendering (312 PAL / 263 NTSC lines) in c64-emu/src/system/c64_system.rs
- [x] T032 [US1] Implement border and background color rendering in c64-emu/src/devices/vic_ii.rs
- [x] T033 [US1] Implement raster counter and raster interrupt generation in c64-emu/src/devices/vic_ii.rs (already done in Phase 2)

### Keyboard Input

- [x] T034 [US1] Implement C64 keyboard matrix (8x8 matrix state) in c64-emu/src/system/keyboard.rs
- [x] T035 [US1] Implement CIA1 keyboard scanning (port A/B matrix) in c64-emu/src/system/c64_memory.rs
- [x] T036 [US1] Implement PC-to-C64 key mapping table in c64-emu/src/system/keyboard.rs
- [x] T037 [US1] Implement RESTORE key (NMI trigger) in c64-emu/src/system/c64_system.rs

### WASM Display API

- [x] T038 [US1] Implement get_framebuffer_ptr() returning pointer to VIC-II buffer in c64-emu/src/wasm.rs
- [x] T039 [US1] Implement get_border_color() and get_current_raster() in c64-emu/src/wasm.rs
- [x] T040 [US1] Implement key_down(), key_up() with matrix positions in c64-emu/src/wasm.rs
- [x] T041 [US1] Implement key_down_pc(), key_up_pc() with PC keycode mapping in c64-emu/src/wasm.rs
- [x] T042 [US1] Implement restore_key() for NMI trigger in c64-emu/src/wasm.rs

### Web Frontend Display

- [x] T043 [P] [US1] Create HTML structure with canvas element (320x200) in c64-demo/index.html
- [x] T044 [P] [US1] Create C64-themed CSS styling (blue background, border) in c64-demo/style.css
- [x] T045 [US1] Implement WASM module loading and initialization in c64-demo/c64.js
- [x] T046 [US1] Implement ROM upload UI and localStorage caching in c64-demo/c64.js (implemented in main app)
- [x] T047 [US1] Implement framebuffer-to-canvas rendering with C64 palette in c64-demo/c64.js (implemented in main app)
- [x] T048 [US1] Implement requestAnimationFrame render loop (50/60 FPS) in c64-demo/c64.js
- [x] T049 [US1] Implement keyboard event handling and mapping in c64-demo/c64.js (implemented in main app)
- [x] T050 [US1] Implement reset and pause/resume controls in c64-demo/c64.js (implemented in main app)

**Checkpoint**: User Story 1 complete - C64 boots to BASIC, keyboard works, screen displays

---

## Phase 4: User Story 2 - Load Programs from Disk Images (Priority: P1)

**Goal**: Mount .D64 disk images, LOAD programs, run them

**Independent Test**: Mount .D64, type `LOAD "*",8,1`, wait for load, type `RUN`, verify program executes

### 1541 Disk Drive

- [x] T051 [US2] Create Drive1541 struct with D64Image, channels, status in c64-emu/src/system/disk_1541.rs
- [x] T052 [US2] Implement D64 file format parsing (track/sector layout, 683 sectors) in c64-emu/src/system/disk_1541.rs
- [x] T053 [US2] Implement directory reading (track 18, sector 1 chain) in c64-emu/src/system/disk_1541.rs
- [x] T054 [US2] Implement file lookup and sector chain following in c64-emu/src/system/disk_1541.rs
- [x] T055 [US2] Implement channel open/read/close operations in c64-emu/src/system/disk_1541.rs
- [x] T056 [US2] Implement drive status channel (channel 15) in c64-emu/src/system/disk_1541.rs

### IEC Protocol

- [x] T057 [US2] Implement high-level IEC command handling (LISTEN, TALK, OPEN, CLOSE) in c64-emu/src/system/iec_bus.rs
- [x] T058 [US2] Implement CIA2 IEC bus signal routing (bits 3-7 of port A) - Not needed for high-level emulation
- [x] T059 [US2] Integrate drive with C64System for bus communication in c64-emu/src/system/c64_system.rs

### WASM Disk API

- [x] T060 [US2] Implement mount_d64() with size validation in c64-emu/src/wasm.rs
- [x] T061 [US2] Implement unmount_d64() and has_mounted_disk() in c64-emu/src/wasm.rs
- [x] T062 [US2] Implement load_prg() for direct memory loading in c64-emu/src/wasm.rs (already implemented earlier)
- [x] T063 [US2] Implement inject_basic_run() for auto-run after load in c64-emu/src/wasm.rs

### Web Frontend File Loading

- [x] T064 [US2] Implement drag-and-drop zone for .D64 and .PRG files in c64-demo/c64.js
- [x] T065 [US2] Implement file picker UI for loading files in c64-demo/c64.js
- [x] T066 [US2] Add disk status indicator to UI in c64-demo/c64.js (HTML/CSS already existed)

**Checkpoint**: User Story 2 complete - Disk images mount, programs load and run

---

## Phase 5: User Story 3 - Authentic Graphics and Sound (Priority: P1)

**Goal**: VIC-II graphics modes, sprites, SID audio playback

**Independent Test**: Run demo with sprites and SID music, verify sprites display, colors correct, audio plays through browser

### VIC-II Advanced Modes

- [x] T067 [US3] Implement multicolor text mode (MCM bit, 160x200 effective) in c64-emu/src/devices/vic_ii.rs
- [x] T068 [US3] Implement standard bitmap mode (BMM bit, 320x200) in c64-emu/src/devices/vic_ii.rs
- [x] T069 [US3] Implement multicolor bitmap mode (BMM+MCM, 160x200) in c64-emu/src/devices/vic_ii.rs
- [x] T070 [US3] Implement ECM (Extended Color Mode) text in c64-emu/src/devices/vic_ii.rs

### VIC-II Sprites

- [x] T071 [US3] Implement sprite data fetching (pointer at Screen+$3F8) in c64-emu/src/devices/vic_ii.rs
- [x] T072 [US3] Implement sprite rendering (8 sprites, 24x21 pixels) in c64-emu/src/devices/vic_ii.rs
- [x] T073 [US3] Implement sprite multicolor mode (12x21 pixels, 4 colors) in c64-emu/src/devices/vic_ii.rs
- [x] T074 [US3] Implement sprite X/Y expansion (double size) in c64-emu/src/devices/vic_ii.rs
- [x] T075 [US3] Implement sprite priority (sprite-to-sprite, sprite-to-background) in c64-emu/src/devices/vic_ii.rs
- [x] T076 [US3] Implement sprite collision detection (registers $1E, $1F) in c64-emu/src/devices/vic_ii.rs

### SID Audio Generation

- [x] T077 [US3] Implement 24-bit phase accumulator per voice in c64-emu/src/devices/sid.rs
- [x] T078 [US3] Implement waveform generation (triangle, sawtooth, pulse, noise) in c64-emu/src/devices/sid.rs
- [x] T079 [US3] Implement pulse width modulation (12-bit duty cycle) in c64-emu/src/devices/sid.rs (completed as part of T078)
- [x] T080 [US3] Implement noise LFSR (23-bit feedback shift register) in c64-emu/src/devices/sid.rs
- [x] T081 [US3] Implement ADSR envelope generator with rate tables in c64-emu/src/devices/sid.rs
- [x] T082 [US3] Implement exponential decay approximation in c64-emu/src/devices/sid.rs
- [x] T083 [US3] Implement simplified biquad filter (LP/BP/HP/Notch modes) in c64-emu/src/devices/sid.rs
- [x] T084 [US3] Implement voice routing through filter in c64-emu/src/devices/sid.rs
- [x] T085 [US3] Implement sample output generation (~23 clocks per 44.1kHz sample) in c64-emu/src/devices/sid.rs

### WASM Audio API

- [x] T086 [US3] Implement get_audio_samples() returning Float32Array in c64-emu/src/wasm.rs
- [x] T087 [US3] Implement set_sample_rate() for resampling configuration in c64-emu/src/wasm.rs
- [x] T088 [US3] Implement set_audio_enabled() for mute control in c64-emu/src/wasm.rs

### Web Frontend Audio

- [x] T089 [US3] Implement AudioWorklet processor for SID playback in c64-demo/components/sid-audio-processor.js
- [x] T090 [US3] Implement audio context initialization (user gesture required) in c64-demo/c64.js
- [x] T091 [US3] Add mute/volume control to UI in c64-demo/c64.js (HTML controls already existed)

**Checkpoint**: User Story 3 complete - All graphics modes work, sprites render, audio plays ✓ COMPLETE

---

## Phase 6: User Story 4 - Joystick Controls for Games (Priority: P2)

**Goal**: Arrow keys and gamepad map to C64 joystick ports for game input

**Independent Test**: Load game, press arrow keys, verify on-screen response to directional and fire inputs

### Joystick Emulation

- [x] T092 [US4] Create JoystickState struct with direction and fire state in c64-emu/src/system/joystick.rs
- [x] T093 [US4] Implement CIA1 joystick reading (port A=joy2, port B=joy1) - Already in c64-emu/src/devices/cia.rs
- [x] T094 [US4] Handle keyboard/joystick port sharing (active low signals) in c64-emu/src/system/c64_memory.rs

### WASM Joystick API

- [x] T095 [US4] Implement set_joystick(port, state) with bitmask and port swap support in c64-emu/src/wasm.rs

### Web Frontend Input

- [x] T096 [US4] Implement arrow key to joystick mapping in c64-demo/c64.js
- [x] T097 [US4] Implement Gamepad API integration in c64-demo/c64.js
- [x] T098 [US4] Add joystick port swap control to UI in c64-demo/index.html and c64-demo/c64.js

**Checkpoint**: User Story 4 complete - Games playable with keyboard and gamepad ✓ COMPLETE

---

## Phase 7: User Story 5 - Save and Restore Emulator State (Priority: P2)

**Goal**: Save complete emulator state, restore later to continue exactly where left off

**Independent Test**: Run program, save state, change state, load state, verify exact restoration

### State Serialization

- [x] T099 [US5] Create SaveState struct with version, CPU, RAM, chip states in c64-emu/src/system/savestate.rs
- [x] T100 [US5] Implement CPU state serialization (a, x, y, sp, pc, flags, cycles) in c64-emu/src/system/savestate.rs
- [x] T101 [US5] Implement VIC-II state serialization (registers, raster, collision flags) in c64-emu/src/system/savestate.rs
- [x] T102 [US5] Implement SID state serialization (registers, voice states) in c64-emu/src/system/savestate.rs
- [x] T103 [US5] Implement CIA state serialization (ports, timers, TOD) in c64-emu/src/system/savestate.rs
- [x] T104 [US5] Implement full state serialization (RAM, all chips, color RAM) in c64-emu/src/system/savestate.rs
- [x] T105 [US5] Implement state deserialization with version validation in c64-emu/src/system/savestate.rs

### WASM State API

- [x] T106 [US5] Implement save_state() returning Uint8Array in c64-emu/src/wasm.rs
- [x] T107 [US5] Implement load_state() with error handling in c64-emu/src/wasm.rs
- [x] T108 [US5] Implement get_state_size() for UI display in c64-emu/src/wasm.rs

### Web Frontend State

- [ ] T109 [US5] Implement save state button with file download in c64-demo/components/controls.js
- [ ] T110 [US5] Implement load state button with file picker in c64-demo/components/controls.js
- [ ] T111 [US5] Implement multiple save slots with localStorage in c64-demo/components/controls.js

**Checkpoint**: User Story 5 complete - Save/load states work across browser sessions

---

## Phase 8: User Story 6 - Emulation Settings (Priority: P3)

**Goal**: User can customize video scaling, scanlines, audio volume, joystick mapping

**Independent Test**: Open settings, change scale to 2x, verify display size changes, enable scanlines, verify effect

### Settings Panel

- [ ] T112 [US6] Create settings panel HTML structure in c64-demo/index.html
- [ ] T113 [US6] Implement video scaling options (1x, 2x, 3x, fit) in c64-demo/components/screen.js
- [ ] T114 [US6] Implement scanline CRT effect (CSS or canvas overlay) in c64-demo/components/screen.js
- [ ] T115 [US6] Implement audio volume slider in c64-demo/components/audio.js
- [ ] T116 [US6] Implement joystick key remapping UI in c64-demo/components/joystick.js
- [ ] T117 [US6] Implement PAL/NTSC region toggle in c64-demo/components/controls.js
- [ ] T118 [US6] Persist settings to localStorage in c64-demo/c64.js

**Checkpoint**: User Story 6 complete - All customization options functional

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Edge Case Handling

- [ ] T119 Handle invalid/corrupted .D64 files with error message in src/c64/disk_1541.rs
- [ ] T120 Handle invalid ROM uploads with specific error messages in src/wasm/c64_api.rs
- [ ] T121 Implement browser tab focus pause/resume in c64-demo/c64.js
- [ ] T122 Implement emulation speed throttling for 50/60 FPS target in c64-demo/c64.js

### Debug Features

- [ ] T123 [P] Implement read_memory() and write_memory() for debugging in src/wasm/c64_api.rs
- [ ] T124 [P] Implement get_cpu_state() returning register values in src/wasm/c64_api.rs
- [ ] T125 [P] Implement get_vic_registers() and get_sid_registers() in src/wasm/c64_api.rs
- [ ] T126 [P] Implement get_cia1_registers() and get_cia2_registers() in src/wasm/c64_api.rs
- [ ] T127 [P] Implement get_bank_config() for memory banking status in src/wasm/c64_api.rs

### Disk Write Support

- [ ] T128 Implement disk write operations (save to D64) in src/c64/disk_1541.rs
- [ ] T129 Implement modified D64 download in c64-demo/components/file-loader.js

### Final Integration

- [ ] T130 Validate quickstart.md examples work with implementation
- [ ] T131 Test cross-browser compatibility (Chrome, Firefox, Safari)
- [ ] T132 Configure GitHub Pages deployment

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - US1, US2, US3 are all P1 and should be done in order (display → disk → audio/graphics)
  - US4, US5 are P2 and can proceed after US1-3
  - US6 is P3 and can proceed after core functionality works
- **Polish (Phase 9)**: Depends on user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No story dependencies
- **User Story 2 (P1)**: Can start after Foundational - Benefits from US1 for testing
- **User Story 3 (P1)**: Can start after Foundational - Benefits from US1/US2 for testing
- **User Story 4 (P2)**: Can start after Foundational - Benefits from games loaded via US2
- **User Story 5 (P2)**: Can start after Foundational - Should have working emulation (US1-3) first
- **User Story 6 (P3)**: Depends on display (US1) and audio (US3) for settings to affect

### Within Each User Story

- Models/structs before methods
- Core Rust implementation before WASM bindings
- WASM bindings before JavaScript frontend
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- Within User Stories, tasks marked [P] can run in parallel (different files)
- Frontend components marked [P] can be developed independently
- Debug API methods (T123-T127) can all run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch frontend tasks in parallel (different files):
Task: "Create HTML structure with canvas element in c64-demo/index.html"
Task: "Create C64-themed CSS styling in c64-demo/style.css"

# After VIC-II rendering complete, launch WASM and frontend display together:
Task: "Implement get_framebuffer_ptr() in src/wasm/c64_api.rs"
Task: "Implement framebuffer-to-canvas rendering in c64-demo/components/screen.js"
```

---

## Implementation Strategy

### MVP First (User Stories 1-3)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (boot to BASIC, keyboard, display)
4. **STOP and VALIDATE**: Test US1 independently - can type and run BASIC programs
5. Complete Phase 4: User Story 2 (disk loading)
6. **STOP and VALIDATE**: Test US2 - can load and run .D64 programs
7. Complete Phase 5: User Story 3 (graphics modes, sprites, audio)
8. **STOP and VALIDATE**: Test US3 - demos/games look and sound correct
9. Deploy/demo MVP

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Test → Deploy (basic emulator works!)
3. Add User Story 2 → Test → Deploy (disk support!)
4. Add User Story 3 → Test → Deploy (full A/V MVP!)
5. Add User Story 4 → Test → Deploy (games playable!)
6. Add User Story 5 → Test → Deploy (save states!)
7. Add User Story 6 → Test → Deploy (customizable!)
8. Polish → Final release

---

## Summary

| Phase | Tasks | Completed | Description |
|-------|-------|-----------|-------------|
| Phase 1: Setup | T001-T006 | 5/6 | Project structure (sub-crate) |
| Phase 2: Foundational | T007-T028 | 22/22 | Core infrastructure |
| Phase 3: US1 | T029-T050 | 22/22 | Boot, keyboard, display ✓ COMPLETE |
| Phase 4: US2 | T051-T066 | 16/16 | Disk image loading ✓ COMPLETE |
| Phase 5: US3 | T067-T091 | 25/25 | Graphics & audio ✓ COMPLETE |
| Phase 6: US4 | T092-T098 | 7/7 | Joystick controls ✓ COMPLETE |
| Phase 7: US5 | T099-T111 | 10/13 | Save/load states (Rust complete, web UI pending) |
| Phase 8: US6 | T112-T118 | 0/7 | Settings |
| Phase 9: Polish | T119-T132 | 0/14 | Edge cases, debug, deploy |
| **Total** | **132 tasks** | **107/132** | |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- VIC-II is frame-accurate (not cycle-exact) per spec FR-020
- 1541 is high-level IEC (not full 6502 drive) per spec FR-070
