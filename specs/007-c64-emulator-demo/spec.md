# Feature Specification: Commodore 64 Emulator Web Demo

**Feature Branch**: `007-c64-emulator-demo` **Created**: 2025-11-20 **Status**:
Draft **Input**: User description: "I want to build a new demo at
demo/c64/index.html that will boot up an emulated c64 into Basic using lib6502.
This project will not implement sound yet, but I do want to implement the video
stack and other relevant hardware such as ram, rom. We can skip cartridges for
now, but ultimately we will want them."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Boot to BASIC Ready Prompt (Priority: P1)

A developer or enthusiast visits the demo page and immediately sees a
functioning Commodore 64 boot sequence, ending at the familiar BASIC ready
prompt where they can type commands.

**Why this priority**: This is the core value proposition—a working C64 in the
browser. Without this, there is no demo. It demonstrates the lib6502 core
successfully running authentic C64 ROM code and proves the emulator's viability.

**Independent Test**: Can be fully tested by loading demo/c64/index.html in a
browser and verifying the boot sequence displays "**_ COMMODORE 64 BASIC V2 _**"
followed by "READY." prompt. Delivers immediate value as a proof-of-concept C64
emulator.

**Acceptance Scenarios**:

1. **Given** a user opens demo/c64/index.html in a web browser, **When** the
   page loads, **Then** they see the C64 screen render with blue background and
   the boot sequence begins
2. **Given** the boot sequence has completed, **When** the screen displays,
   **Then** the user sees the "READY." prompt with a blinking cursor
3. **Given** the emulator has booted, **When** the user checks the screen,
   **Then** they see the correct startup message including "38911 BASIC BYTES
   FREE"

---

### User Story 2 - Execute BASIC Commands (Priority: P2)

A user types BASIC commands at the ready prompt and sees the correct output
rendered on screen, demonstrating the emulator can handle keyboard input and
execute actual programs.

**Why this priority**: This validates the complete input/output loop and proves
the emulator isn't just playing back a recording—it's actually executing code.
It provides interactive value and demonstrates practical functionality.

**Independent Test**: Can be tested by typing "PRINT 2+2" at the READY prompt
and verifying "4" appears on screen. Can also test typing program lines like "10
PRINT HELLO" and running them with RUN command. Delivers value as an interactive
coding environment.

**Acceptance Scenarios**:

1. **Given** the BASIC prompt is ready, **When** the user types "PRINT 2+2" and
   presses RETURN, **Then** the number "4" appears on the next line
2. **Given** the user is at the prompt, **When** they type a multi-line program
   (e.g., "10 PRINT 'HELLO'", "20 GOTO 10"), **Then** each line is echoed
   correctly on screen
3. **Given** a program is entered, **When** the user types "RUN" and presses
   RETURN, **Then** the program executes and output appears on screen
4. **Given** the user is typing, **When** they make a mistake and press the
   backspace key, **Then** the cursor moves back and deletes the character

---

### User Story 3 - See Authentic C64 Display (Priority: P3)

Users see an authentic-looking C64 display with correct colors, character set,
and screen dimensions, making the demo feel like a real Commodore 64.

**Why this priority**: Visual authenticity enhances the experience and builds
trust that this is a genuine emulation. While functional correctness is more
important, accurate presentation is what makes people want to share and use the
demo.

**Independent Test**: Can be tested by comparing the rendered display against
C64 screenshots—verify blue border, light blue text, 40×25 character grid,
PETSCII character appearance. Delivers value as a nostalgic and visually
accurate experience.

**Acceptance Scenarios**:

1. **Given** the emulator is running, **When** the user looks at the display,
   **Then** they see the classic C64 blue border color (#6C5EB5) and light blue
   text (#6C9FB5)
2. **Given** text is displayed on screen, **When** the user examines the
   characters, **Then** they see the authentic C64 PETSCII character set
   including the distinctive rounded letterforms
3. **Given** the screen is rendering, **When** the user checks the layout,
   **Then** they see exactly 40 characters per line and 25 lines on screen
4. **Given** the cursor is blinking, **When** the user observes it, **Then** it
   blinks at approximately the correct rate (matching C64 hardware timing)

---

### Edge Cases

- When user types beyond 40-character line limit, cursor wraps to next line
  automatically (authentic C64 behavior)
- What happens when output scrolls beyond 25 lines (should scroll the display
  upward, maintaining the border)?
- Rapid key input buffered up to 10 keystrokes (authentic C64 buffer size);
  additional input dropped
- What happens when invalid BASIC syntax is entered (should display "?SYNTAX
  ERROR" per BASIC V2 behavior)?
- Browser window resize maintains aspect ratio and scales display proportionally
- When user leaves page and returns, emulator restarts to clean boot state (no
  state persistence)
- ROM binary load failure displays error message in browser console and shows
  blank screen

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST implement C64 memory map including 64KB RAM, BASIC
  ROM, KERNAL ROM, and character ROM at correct addresses
- **FR-002**: System MUST accurately emulate the VIC-II video chip to render
  40×25 character display with correct C64 color palette
- **FR-003**: System MUST render the authentic C64 PETSCII character set
  including both uppercase/graphics and lowercase character modes
- **FR-004**: System MUST implement keyboard input mapping from modern keyboard
  to C64 PETSCII codes
- **FR-005**: System MUST execute the C64 boot sequence including memory
  initialization and BASIC startup
- **FR-006**: System MUST implement the VIC-II memory-mapped registers for
  screen control and color management
- **FR-007**: System MUST handle screen scrolling when output exceeds 25 visible
  lines
- **FR-016**: System MUST wrap cursor to next line automatically when user types
  beyond 40-character line limit
- **FR-017**: System MUST buffer keyboard input up to 10 keystrokes and drop
  additional input when buffer is full
- **FR-018**: System MUST maintain aspect ratio and scale display proportionally
  when browser window is resized
- **FR-019**: System MUST restart emulator to clean boot state when user returns
  to the page (no state persistence between sessions)
- **FR-020**: System MUST display error message in browser console and show
  blank screen when ROM binaries fail to load at startup
- **FR-008**: System MUST implement the CIA timer chips for timing and keyboard
  scanning
- **FR-009**: System MUST render display updates at appropriate refresh rate to
  maintain visual smoothness
- **FR-010**: System MUST provide the complete BASIC V2 interpreter via
  authentic Commodore BASIC ROM
- **FR-011**: System MUST implement the KERNAL routines for character
  input/output and screen management
- **FR-012**: Demo page MUST load and display in modern web browsers without
  requiring plugins or installations
- **FR-013**: System MUST handle keyboard focus management so typing goes to the
  emulator when the display is active
- **FR-014**: System MUST implement cursor blinking behavior matching C64 visual
  characteristics
- **FR-015**: System MUST preserve the modularity principle from the
  constitution—C64-specific components built on top of generic lib6502 core

### Key Entities

- **C64 Memory Map**: 64KB address space with RAM (0x0000-0x9FFF,
  0xC000-0xCFFF), BASIC ROM (0xA000-0xBFFF), I/O area (0xD000-0xDFFF), KERNAL
  ROM (0xE000-0xFFFF), character ROM accessible via bank switching
- **VIC-II Video Chip**: Memory-mapped hardware at 0xD000-0xD3FF controlling
  screen display, character mode rendering, sprite registers (not implemented
  initially), color memory, border color, background color
- **CIA Timers**: Complex Interface Adapter chips at 0xDC00-0xDCFF and
  0xDD00-0xDDFF handling keyboard scanning, timing, and I/O port management
- **Screen Memory**: 1000-byte region (typically 0x0400-0x07E7) holding
  character codes for 40×25 display
- **Color Memory**: 1000-byte region (0xD800-0xDBE7) holding color attributes
  for each screen position
- **Character ROM**: 4KB ROM containing PETSCII character bitmaps for 256
  characters in 8×8 pixel format
- **Keyboard Matrix**: 8×8 matrix representing C64 keyboard state, scanned by
  CIA and translated to PETSCII codes by KERNAL

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Users can load demo/c64/index.html and see the C64 boot sequence
  complete within 3 seconds
- **SC-002**: Users can type BASIC commands and see character echo appear on
  screen within 100 milliseconds
- **SC-003**: The display maintains stable 50Hz or 60Hz refresh rate without
  visible tearing or stuttering during normal operation
- **SC-004**: Users successfully execute at least 5 different BASIC commands
  (PRINT, LIST, RUN, NEW, LOAD) with correct output
- **SC-005**: The emulator correctly executes BASIC programs with at least 100
  lines of code without errors
- **SC-006**: Visual appearance matches authentic C64 display as verified by
  side-by-side comparison with reference screenshots
- **SC-007**: Demo page loads and becomes interactive within 2 seconds on modern
  desktop browsers
- **SC-008**: Keyboard input handling supports typing at normal human speed (up
  to 10 characters per second) without dropped keys

## Clarifications

### Session 2025-11-20

- Q: When the user types beyond the 40-character line limit, how should the
  system respond? → A: Wrap cursor to next line automatically (authentic C64
  behavior)
- Q: When the user types faster than the emulator can process, how should the
  system handle rapid key input? → A: Buffer input up to 10 keystrokes then drop
  additional input (authentic C64 buffer size)
- Q: When the browser window is resized, how should the emulator display
  respond? → A: Maintain aspect ratio and scale display proportionally
- Q: When the user leaves the page and returns, how should the emulator handle
  state? → A: Restart emulator to clean boot state
- Q: When C64 ROM binaries fail to load at startup, how should the system
  respond? → A: Display error message in browser console and show blank screen

## Assumptions

- C64 BASIC ROM and KERNAL ROM binaries are legally obtained and available for
  the project (assuming public domain or licensed)
- Initial demo targets desktop browsers only (mobile touch keyboard support
  deferred)
- Standard NTSC timing model used (60Hz frame rate); PAL timing (50Hz) deferred
  to future enhancement
- Only text mode rendering required for initial demo; bitmap graphics, sprites,
  and multicolor modes deferred
- Sound chip (SID) explicitly excluded from this feature per user input
- Cartridge support explicitly excluded from this feature per user input
- Demo operates entirely client-side with no server communication required
- WebAssembly target compilation works with current lib6502 architecture
  (validated by constitution)
- Focus on accuracy over performance optimization—cycle-perfect timing deferred
  if it complicates initial implementation
