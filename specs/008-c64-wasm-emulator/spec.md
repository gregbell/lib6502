# Feature Specification: Commodore 64 WASM Emulator

**Feature Branch**: `008-c64-wasm-emulator` **Created**: 2025-01-22 **Status**:
Draft **Input**: User description: "spec a fully emulated c64 using lib6502 that
will run in the browser via wasm"

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Run Classic C64 Programs (Priority: P1)

A retro computing enthusiast visits the browser-based C64 emulator and loads a
classic BASIC program or machine code demo. They see the familiar C64 boot
screen, can type on their keyboard (mapped to the C64 keyboard matrix), and run
programs that render graphics and play sound just like the original hardware.

**Why this priority**: This is the core value proposition - a fully functional
C64 in the browser. Without accurate emulation of the core hardware, nothing
else matters.

**Independent Test**: Can be fully tested by loading the emulator, observing the
C64 BASIC startup screen with "64K RAM SYSTEM" message, typing a simple BASIC
program (e.g., `10 PRINT "HELLO": GOTO 10`), running it, and verifying output
appears on the emulated screen. Delivers the authentic C64 experience.

**Acceptance Scenarios**:

1. **Given** the emulator is loaded, **When** the page initializes, **Then** the
   C64 boot sequence executes showing the blue screen with "COMMODORE 64 BASIC
   V2" and "READY." prompt
2. **Given** the boot sequence completes, **When** user types on their keyboard,
   **Then** characters appear on the emulated C64 screen in the correct position
3. **Given** a BASIC program is entered, **When** user types RUN and presses
   Enter, **Then** the program executes and output appears on screen
4. **Given** the emulator is running, **When** the user presses a function key,
   **Then** the corresponding C64 function key action occurs (e.g., F5 loads, F7
   lists)

---

### User Story 2 - Load Programs from Disk Images (Priority: P1)

A user wants to play classic C64 games or run software. They drag and drop a
.D64 disk image file onto the emulator, and the emulated 1541 disk drive becomes
accessible. They can LOAD programs from the disk and run them.

**Why this priority**: Most C64 software was distributed on floppy disks.
Without disk support, users cannot run the vast library of existing C64
software, making the emulator far less useful.

**Independent Test**: Can be fully tested by loading a .D64 image containing a
simple program, typing `LOAD "*",8,1`, waiting for the program to load, typing
`RUN`, and verifying the program executes correctly. Delivers access to the
entire C64 software library.

**Acceptance Scenarios**:

1. **Given** the emulator is running, **When** user drags a .D64 file onto the
   emulator, **Then** the file is mounted as a virtual disk in drive 8
2. **Given** a .D64 is mounted, **When** user types `LOAD "$",8`, **Then** the
   disk directory loads into memory
3. **Given** the directory is loaded, **When** user types `LIST`, **Then** the
   disk directory displays showing filenames and file types
4. **Given** a program is on the mounted disk, **When** user types
   `LOAD "PROGRAM",8,1` and then `RUN`, **Then** the program loads and executes

---

### User Story 3 - Experience Authentic Graphics and Sound (Priority: P1)

A user runs a C64 demo or game and expects to see VIC-II graphics (sprites,
multicolor modes, raster effects) and hear SID chip sound (music, sound
effects). The graphics display at the correct resolution and the audio plays
through the browser.

**Why this priority**: The VIC-II and SID are what made the C64 special. Games
and demos rely heavily on these chips for their experience. Without accurate
emulation, programs won't look or sound right.

**Independent Test**: Can be fully tested by running a known demo (e.g., a
simple scroller with SID music), verifying sprites display correctly, colors
match expected output, and audio plays through browser speakers. Delivers the
authentic C64 audiovisual experience.

**Acceptance Scenarios**:

1. **Given** a program uses sprites, **When** the program runs, **Then** sprites
   display at correct positions with proper collision detection
2. **Given** a program uses multicolor bitmap mode, **When** the program runs,
   **Then** the graphics display with correct color palette and resolution
   (320x200 or 160x200 multicolor)
3. **Given** a program plays SID music, **When** the program runs, **Then**
   audio plays through browser speakers matching the expected sound
4. **Given** a program uses raster interrupts, **When** the raster reaches
   specified lines, **Then** interrupts fire at correct screen positions
   enabling effects like split-screen scrolling

---

### User Story 4 - Use Joystick Controls for Games (Priority: P2)

A user wants to play C64 games using either their keyboard (arrow keys + a fire
button) or a connected gamepad. The emulator maps these inputs to the C64's
joystick ports so games are playable.

**Why this priority**: Most C64 games require joystick input. Without this,
games are unplayable, but keyboard BASIC interaction is still possible (hence P2
not P1).

**Independent Test**: Can be fully tested by loading a game that uses joystick
input, pressing arrow keys or using a gamepad, and verifying the on-screen
character/sprite responds to directional and fire button inputs. Delivers
playable game experience.

**Acceptance Scenarios**:

1. **Given** a game expects joystick in port 2, **When** user presses arrow
   keys, **Then** the game responds to directional input
2. **Given** a game expects fire button, **When** user presses the mapped fire
   key (e.g., Ctrl or Space), **Then** the fire action triggers in the game
3. **Given** a USB gamepad is connected, **When** user moves the gamepad stick,
   **Then** the joystick direction registers in the game
4. **Given** joystick port selection, **When** user switches between port 1 and
   port 2, **Then** input routes to the selected port

---

### User Story 5 - Save and Restore Emulator State (Priority: P2)

A user playing a game wants to save their progress at any point. They click a
save state button, and the entire emulator state is captured. Later, they can
load that state to continue exactly where they left off.

**Why this priority**: Save states are a modern convenience that enhance
usability, especially for games without built-in save features. The emulator is
still useful without this, but user experience is significantly better with it.

**Independent Test**: Can be fully tested by running a program, clicking "Save
State", changing the program state (e.g., moving a character, changing screen
content), clicking "Load State", and verifying the emulator returns to the exact
saved state. Delivers modern save/load convenience.

**Acceptance Scenarios**:

1. **Given** a program is running, **When** user clicks "Save State", **Then** a
   snapshot file is generated and available for download
2. **Given** a saved state exists, **When** user loads the state file, **Then**
   the emulator restores to the exact saved state (CPU registers, RAM, VIC-II
   state, SID state)
3. **Given** multiple save states exist, **When** user selects a specific state,
   **Then** that particular state loads correctly
4. **Given** a save state from a different session, **When** user loads it in a
   new browser session, **Then** the emulator restores correctly

---

### User Story 6 - Adjust Emulation Settings (Priority: P3)

A user wants to customize their emulation experience. They access a settings
panel to adjust video scaling, enable/disable scanline effects, change joystick
mappings, or adjust audio volume.

**Why this priority**: Customization improves user experience but is not
essential for basic functionality. The emulator works fine with sensible
defaults.

**Independent Test**: Can be fully tested by opening settings, changing video
scaling from 1x to 2x, verifying the display size changes, enabling scanlines,
and verifying the visual effect applies. Delivers personalization options.

**Acceptance Scenarios**:

1. **Given** the settings panel is open, **When** user changes video scale,
   **Then** the emulator display resizes accordingly
2. **Given** scanline effect is enabled, **When** viewing the display, **Then**
   horizontal darkened lines appear simulating CRT display
3. **Given** audio volume slider is adjusted, **When** SID audio plays, **Then**
   volume matches the selected level
4. **Given** joystick mapping is changed, **When** user presses the newly mapped
   key, **Then** the correct joystick action registers

---

### Edge Cases

- What happens when user loads an incompatible or corrupted .D64 file? System
  displays error message indicating the file could not be read; emulator
  continues running with no disk mounted.
- What happens when a program writes to ROM addresses? Writes are ignored per
  authentic C64 behavior (ROM is read-only); emulation continues without error.
- What happens when emulation falls behind real-time? Audio may stutter; visual
  display shows frame drops; system prioritizes audio continuity where possible.
- What happens when user resizes the browser window? The emulator display scales
  proportionally while maintaining aspect ratio; scanline effects (if enabled)
  scale appropriately.
- What happens when a program performs an illegal 6502 opcode? Emulator behavior
  matches documented C64 behavior for illegal opcodes (many were used
  intentionally by games/demos).
- What happens when the browser tab loses focus? Emulation pauses to conserve
  resources; resumes when tab regains focus.
- What happens when user tries to load a .PRG file directly? The file loads into
  memory at the address specified in the first two bytes; user can type RUN to
  execute if it's a BASIC program.
- What happens when user uploads invalid ROM files? System validates file sizes
  (BASIC=8KB, KERNAL=8KB, CHARROM=4KB) and displays specific error message
  indicating which ROM is invalid and expected size; emulator does not start
  until valid ROMs are provided.

## Requirements _(mandatory)_

### Functional Requirements

#### CPU Emulation

- **FR-001**: System MUST emulate the MOS 6510 CPU (6502 with I/O port at
  $00-$01) using the existing lib6502 CPU core
- **FR-002**: System MUST implement the 6510's built-in I/O port for memory bank
  switching and datasette control signals
- **FR-003**: System MUST execute at the authentic PAL clock speed of 985,248 Hz
  (or NTSC at 1,022,727 Hz based on user preference)
- **FR-004**: System MUST support NMI (Non-Maskable Interrupt) for RESTORE key
  functionality
- **FR-005**: System MUST support IRQ interrupts from CIA chips and VIC-II

#### Memory Architecture

- **FR-010**: System MUST implement the C64's full 64KB address space with
  proper bank switching
- **FR-011**: System MUST provide 64KB of RAM accessible through bank switching
- **FR-012**: System MUST include the C64 BASIC ROM (8KB at $A000-$BFFF)
- **FR-013**: System MUST include the C64 KERNAL ROM (8KB at $E000-$FFFF)
- **FR-014**: System MUST include the Character ROM (4KB, memory-mapped at
  $D000-$DFFF when accessed by VIC-II)
- **FR-015**: System MUST implement the memory banking controlled by $01
  processor port and VIC-II bank selection

#### VIC-II Video Chip

- **FR-020**: System MUST emulate the VIC-II (MOS 6569 for PAL) video chip at frame-accurate level (not cycle-exact); sufficient for games but some advanced demos may not render correctly
- **FR-021**: System MUST support all standard display modes: standard text,
  multicolor text, standard bitmap, multicolor bitmap, and ECM (Extended Color
  Mode)
- **FR-022**: System MUST render the full PAL display (504x312 pixels visible
  area, 320x200 active area) or NTSC equivalent
- **FR-023**: System MUST support all 8 hardware sprites with proper collision
  detection
- **FR-024**: System MUST support sprite priority (sprite-to-sprite and
  sprite-to-background)
- **FR-025**: System MUST implement raster interrupts at the correct scanline
  positions
- **FR-026**: System MUST emulate border color changes including the classic
  split-border effects
- **FR-027**: System MUST update the display at 50 Hz (PAL) or 60 Hz (NTSC)
  frame rate

#### SID Sound Chip

- **FR-030**: System MUST emulate the SID MOS 6581 sound chip (not 8580) with 3
  oscillator voices; 8580 variant support deferred to future enhancement
- **FR-031**: System MUST support all SID waveforms: triangle, sawtooth, pulse
  (with variable duty cycle), and noise
- **FR-032**: System MUST implement ADSR envelope generators for each voice
- **FR-033**: System MUST implement the multimode filter (low-pass, high-pass,
  band-pass)
- **FR-034**: System MUST output audio through the Web Audio API at a sample
  rate compatible with browser audio (typically 44.1kHz or 48kHz)
- **FR-035**: System MUST implement volume control and master volume register

#### CIA Chips (Complex Interface Adapter)

- **FR-040**: System MUST emulate both CIA chips (MOS 6526) at $DC00-$DCFF
  (CIA1) and $DD00-$DDFF (CIA2)
- **FR-041**: System MUST implement CIA1 keyboard matrix scanning
- **FR-042**: System MUST implement CIA1 joystick port reading (port A and port
  B)
- **FR-043**: System MUST implement CIA timer interrupts (Timer A and Timer B on
  both CIAs)
- **FR-044**: System MUST implement CIA2 serial bus control for IEC (disk drive
  communication)
- **FR-045**: System MUST implement time-of-day clocks (TOD) on both CIAs

#### Keyboard Input

- **FR-050**: System MUST map PC keyboard keys to the C64 keyboard matrix
- **FR-051**: System MUST support special keys: RESTORE (NMI), RUN/STOP, and
  Commodore key
- **FR-052**: System MUST handle keyboard rollover for multiple simultaneous key
  presses
- **FR-053**: System MUST provide visual or configurable keyboard mapping
  reference

#### Joystick Input

- **FR-060**: System MUST map arrow keys and a fire button to joystick port 2 by
  default
- **FR-061**: System MUST support alternative keyboard layouts for joystick
  input
- **FR-062**: System MUST support browser Gamepad API for USB/Bluetooth
  controllers
- **FR-063**: System MUST allow user to swap joystick port mapping (port 1 vs
  port 2)

#### Disk Drive Emulation

- **FR-070**: System MUST emulate the 1541 disk drive at device 8 using high-level IEC protocol emulation (not full 6502 drive CPU); copy-protected software requiring low-level timing may not work
- **FR-071**: System MUST support loading .D64 disk image files
- **FR-072**: System MUST implement the IEC serial bus protocol for
  communication
- **FR-073**: System MUST support reading disk directory and loading programs
- **FR-074**: System MUST support writing to disk images (save functionality)

#### File Loading

- **FR-080**: System MUST support loading .PRG program files directly into
  memory
- **FR-081**: System MUST support drag-and-drop file loading for .D64 and .PRG
  files
- **FR-082**: System MUST provide a file picker UI for loading files

#### State Management

- **FR-090**: System MUST support saving complete emulator state to a
  downloadable file
- **FR-091**: System MUST support loading previously saved state files
- **FR-092**: Saved state MUST include: CPU state, all 64KB RAM, VIC-II
  registers, SID state, CIA states, and mounted disk image reference

#### User Interface

- **FR-100**: System MUST display the emulated C64 screen as an HTML canvas
  element
- **FR-101**: System MUST provide control buttons: Reset, Pause/Resume, Mute
- **FR-102**: System MUST provide file loading interface (drag-drop zone and
  file picker)
- **FR-103**: System MUST provide settings panel for: video scaling, scanline
  effects, audio volume, region selection (PAL/NTSC), joystick configuration
- **FR-104**: System MUST be deployable to GitHub Pages as static files
- **FR-105**: System MUST prompt user to upload ROM files (BASIC, KERNAL, Character ROM) on first use and store them in browser localStorage for subsequent sessions
- **FR-106**: System MUST validate uploaded ROM file sizes (BASIC=8KB, KERNAL=8KB, Character ROM=4KB) and display specific error messages for invalid files

### Key Entities

- **C64 System**: The complete emulated machine containing CPU, memory, and all
  chips working together
- **Memory Map**: The 64KB address space with bank-switching logic determining
  what components respond at each address
- **VIC-II State**: Video chip registers, sprite data, color RAM, and display
  mode configuration
- **SID State**: Sound chip registers, oscillator states, filter configuration,
  and envelope generators
- **CIA State**: Timer values, interrupt masks, port registers, and time-of-day
  clocks
- **Disk Image**: Virtual floppy disk containing programs and data in .D64
  format
- **Save State**: Serialized snapshot of entire emulator state for save/load
  functionality

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: The C64 BASIC boot screen displays within 2 seconds of page load,
  showing "64K RAM SYSTEM" and "READY." prompt
- **SC-002**: Users can type BASIC programs and execute them with correct output
  100% of the time for valid programs
- **SC-003**: At least 90% of commercial C64 games from the top 100 most popular
  titles run correctly (display properly, sound works, controls responsive)
- **SC-004**: SID music playback matches reference recordings within acceptable
  audio quality (no obvious timing drift, correct instrument sounds)
- **SC-005**: Emulation maintains 50/60 fps frame rate on modern hardware (2020+
  desktop/laptop) without audio stuttering
- **SC-006**: .D64 files mount and load programs within 3 seconds of user action
- **SC-007**: Save states capture and restore complete system state with 100%
  accuracy (byte-for-byte RAM match, identical display after restore)
- **SC-008**: The emulator works correctly in the 3 major browser engines
  (Chromium, Firefox, Safari/WebKit)
- **SC-009**: Keyboard input latency is imperceptible to users (under 50ms from
  keypress to on-screen response)
- **SC-010**: Joystick/gamepad input is responsive enough for action games
  (under 16ms input lag at 60fps)

## Scope

### In Scope

- Full 6510 CPU emulation (building on lib6502)
- Complete VIC-II video chip emulation (all display modes, sprites, raster
  interrupts)
- SID 6581 audio chip emulation (3 voices, filters, ADSR envelopes)
- Both CIA 6526 chips (timers, keyboard, joystick, serial bus)
- C64 memory architecture with bank switching
- BASIC V2 and KERNAL ROM integration
- 1541 disk drive emulation for .D64 files
- .PRG file loading
- Keyboard and joystick input mapping
- Save/load state functionality
- Web-based UI with HTML/CSS/JavaScript
- GitHub Pages deployment
- PAL and NTSC region support

### Out of Scope

- Datasette (tape) emulation - disk is sufficient for most software
- REU (RAM Expansion Unit) emulation - advanced hardware not essential for core
  experience
- Other disk drive models (1571, 1581) - 1541 is the standard
- Cartridge port emulation - could be added in future
- Second SID chip (for stereo mods) - single SID covers original hardware
- SID 8580 variant emulation - 6581 provides classic C64 sound
- Serial port for user-port devices
- VIC-20 or C128 compatibility modes
- Multi-player network functionality
- Full 1541 drive CPU emulation (simplified timing acceptable for most software)
- Mobile-optimized touch controls (desktop keyboard/gamepad focus)
- Cycle-exact VIC-II timing for advanced demo effects (frame-accurate is sufficient for games)

## Assumptions

- Users have modern browsers with WebAssembly and Web Audio API support
  (released 2018+)
- C64 ROM files (BASIC, KERNAL, Character) are uploaded by the user on first use; the emulator does not ship with or link to ROMs due to copyright
- Users have legitimate .D64 disk images for software they want to run
- Desktop/laptop usage is primary (keyboard and optional gamepad)
- Single-user experience (no networking or multi-player)
- The existing lib6502 CPU core provides sufficient cycle accuracy for C64
  compatibility
- Memory-mapped device architecture from lib6502 can accommodate C64 hardware
  complexity
- Browser Gamepad API is sufficient for joystick emulation
- Web Audio API provides acceptable latency for game audio

## Dependencies

- lib6502 CPU core with 6510 I/O port extension
- lib6502 memory mapping infrastructure (MappedMemory, Device trait)
- lib6502 WASM compilation infrastructure (from 003-wasm-web-demo)
- C64 ROM images (BASIC, KERNAL, Character ROM)
- Modern browser with WebAssembly and Web Audio API
- GitHub Pages for deployment

## Open Questions

None remaining.

## Clarifications

### Session 2026-01-22

- Q: How will ROM files be handled? → A: User uploads ROMs on first use
- Q: VIC-II cycle accuracy level? → A: Frame-accurate (games focus)
- Q: 1541 drive emulation depth? → A: High-level IEC protocol emulation
- Q: Which SID chip variant to emulate? → A: 6581 only (original C64)
- Q: How to handle invalid ROM uploads? → A: Validate sizes, show specific error message
