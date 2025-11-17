# Browser Testing Plan - Memory Mapping & UART

## Overview

This document outlines the manual testing checklist for validating the memory mapping module and UART device in browser environments using WebAssembly.

## Prerequisites

- Modern browser (Chrome 85+, Firefox 78+, Safari 14+)
- Local web server (Python's `http.server`, Node's `http-server`, etc.)
- WASM build environment (`wasm-pack`)
- Terminal library (xterm.js recommended)

## Build Setup

### 1. Install Dependencies

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install npm packages for web app
npm install xterm xterm-addon-fit
```

### 2. Build WASM Module

```bash
# Build for web target
wasm-pack build --target web

# Output will be in pkg/ directory
```

### 3. Serve Locally

```bash
# Using Python
python3 -m http.server 8000

# Or using Node
npx http-server -p 8000
```

## Test Cases

### TC-01: Basic UART Transmit

**Objective**: Verify UART transmits characters to browser terminal

**Setup**:
1. Load echo program into RAM at 0x0200
2. Configure UART at 0x8000
3. Connect transmit callback to xterm.js

**Steps**:
1. Run emulator for 100 cycles
2. Write byte 0x41 ('A') to UART data register (0x8000)
3. Observe terminal display

**Expected**:
- Character 'A' appears in terminal
- Transmit callback fires immediately
- Status register TDRE flag remains set

**Pass/Fail**: ___

---

### TC-02: Basic UART Receive

**Objective**: Verify UART receives characters from browser terminal

**Setup**:
1. Load echo program into RAM
2. Configure UART with receive buffer
3. Connect terminal onData to receive_byte()

**Steps**:
1. Type 'B' in terminal
2. Check UART status register (0x8001)
3. Read UART data register (0x8000)

**Expected**:
- RDRF flag (bit 3) set in status register
- Data register returns 0x42 ('B')
- Receive buffer contains character

**Pass/Fail**: ___

---

### TC-03: Bidirectional Echo

**Objective**: Verify bidirectional character flow

**Setup**:
1. Load echo program (reads from UART, writes back)
2. Configure both transmit and receive

**Steps**:
1. Type "Hello" in terminal
2. Run emulator continuously
3. Observe terminal output

**Expected**:
- All characters echo back: "HHeelllloo"
- No characters lost or duplicated
- Cycle timing feels responsive

**Pass/Fail**: ___

---

### TC-04: UART Buffer Overflow

**Objective**: Verify receive buffer handles overflow gracefully

**Setup**:
1. Configure UART with default 256-byte buffer
2. Pause emulator (don't read from UART)

**Steps**:
1. Type 260 characters rapidly
2. Check status register
3. Resume emulator and read data

**Expected**:
- Overrun flag (bit 2) set after 256 bytes
- Buffer contains first 256 characters
- Subsequent characters dropped without crash

**Pass/Fail**: ___

---

### TC-05: Echo Mode

**Objective**: Verify echo mode automatically retransmits

**Setup**:
1. Configure UART at 0x8000
2. Set command register (0x8002) bit 3 to enable echo

**Steps**:
1. Call receive_byte('C') from JavaScript
2. Observe terminal (no CPU program needed)

**Expected**:
- Character 'C' appears in terminal immediately
- Transmit callback fires automatically
- CPU doesn't need to read/write

**Pass/Fail**: ___

---

### TC-06: Status Register Flags

**Objective**: Verify all status register bits update correctly

**Setup**:
1. Configure UART
2. Monitor status register (0x8001)

**Steps**:
1. Read status after init: should be 0x10 (TDRE only)
2. Receive byte: should be 0x18 (TDRE + RDRF)
3. Read data: RDRF clears, should be 0x10
4. Overflow buffer: should include 0x04 (Overrun)

**Expected**:
- TDRE (bit 4) always set
- RDRF (bit 3) reflects buffer state
- Overrun (bit 2) set on overflow
- Bits update immediately

**Pass/Fail**: ___

---

### TC-07: Command Register

**Objective**: Verify command register read/write

**Setup**:
1. Configure UART at 0x8000

**Steps**:
1. Write 0xAA to command register (0x8002)
2. Read back from 0x8002
3. Verify echo mode bit affects behavior

**Expected**:
- Command register stores written value
- Echo mode bit (3) affects receive behavior
- Other bits stored for program use

**Pass/Fail**: ___

---

### TC-08: Control Register

**Objective**: Verify control register read/write

**Setup**:
1. Configure UART at 0x8000

**Steps**:
1. Write 0x55 to control register (0x8003)
2. Read back from 0x8003

**Expected**:
- Control register stores written value
- Reads return last written value
- No side effects on operation

**Pass/Fail**: ___

---

### TC-09: Memory Map Integration

**Objective**: Verify UART integrates with memory bus

**Setup**:
1. Configure MappedMemory with:
   - RAM at 0x0000-0x7FFF
   - UART at 0x8000-0x8003
   - ROM at 0xC000-0xFFFF

**Steps**:
1. Write to RAM (e.g., 0x1000): verify RAM updated
2. Write to UART (0x8000): verify transmit fires
3. Write to ROM (0xC000): verify write ignored
4. Read from each region

**Expected**:
- Each device responds correctly
- No device overlap errors
- Unmapped addresses return 0xFF

**Pass/Fail**: ___

---

### TC-10: CPU Program Integration

**Objective**: Verify CPU can drive UART through memory writes

**Setup**:
1. Load program:
   ```
   LDA #$41      ; Load 'A'
   STA $8000     ; Write to UART
   ```

**Steps**:
1. Execute LDA instruction
2. Execute STA instruction
3. Observe terminal

**Expected**:
- Character 'A' appears in terminal
- CPU A register = 0x41
- Cycle count increases correctly

**Pass/Fail**: ___

---

### TC-11: Rapid Input Stress Test

**Objective**: Verify system handles rapid input

**Setup**:
1. Configure UART with echo program running

**Steps**:
1. Paste 1000 characters into terminal
2. Observe output and buffer state

**Expected**:
- Characters buffered up to 256-byte limit
- Overrun flag set if overflow
- No browser freeze or crash
- Characters processed as buffer drains

**Pass/Fail**: ___

---

### TC-12: Long-Running Session

**Objective**: Verify no memory leaks or degradation

**Setup**:
1. Start emulator with echo program
2. Run continuously for 5 minutes

**Steps**:
1. Type intermittently during session
2. Monitor browser memory usage
3. Check console for errors

**Expected**:
- Memory usage stable
- No console errors
- Performance consistent
- No frame drops

**Pass/Fail**: ___

---

## Browser-Specific Tests

### Chrome/Edge

**Special Checks**:
- DevTools Performance tab shows stable frame rate
- Memory profiler shows no leaks
- WASM module loads correctly

**Pass/Fail**: ___

---

### Firefox

**Special Checks**:
- Console shows no WASM warnings
- Performance tools show stable execution
- Module loads without CORS errors

**Pass/Fail**: ___

---

### Safari

**Special Checks**:
- WASM loads on first try
- No BigInt polyfill needed
- Terminal input/output responsive

**Pass/Fail**: ___

---

## UART Buffer Behavior in Browser Context

### Receive Buffer

**Characteristics**:
- **Capacity**: 256 bytes (configurable via `rx_buffer_capacity`)
- **Type**: VecDeque for efficient FIFO operations
- **Overflow**: Sets overrun flag, drops new bytes when full
- **Persistence**: Buffer persists until explicitly read or cleared

**Browser Integration**:
- JavaScript `onData` events call `receive_byte()`
- Each character pushed to back of buffer
- CPU reads pop from front of buffer
- Buffer handles burst input (paste, rapid typing)

**Timing Considerations**:
- Browser input events fire asynchronously
- Emulator runs in requestAnimationFrame loop (~60 FPS)
- Buffer decouples input timing from CPU execution
- Large pastes may take multiple frames to process

### Transmit Callback

**Characteristics**:
- Fires immediately on write to data register
- No transmit buffer (TDRE always ready)
- Callback must be lightweight (runs in hot path)

**Browser Integration**:
- Callback writes to xterm.js terminal
- Runs in WASM context via wasm-bindgen
- Must avoid heavy DOM operations
- Terminal handles buffering/rendering

### Status Register

**Browser Implications**:
- RDRF flag indicates buffer has data
- Programs should poll before reading
- Overrun flag alerts to dropped input
- Status reads are zero-cost (just bit checks)

### Echo Mode

**Browser Use Case**:
- Useful for simple terminals
- CPU doesn't need to run for echo
- Reduces cycle budget requirements
- Good for testing before CPU integration

---

## Performance Benchmarks

### Target Metrics

- **Cycle Rate**: ~1 MHz (1000-2000 cycles/frame at 60 FPS)
- **Input Latency**: < 50ms from keypress to buffer
- **Output Latency**: < 16ms from transmit to terminal
- **Frame Rate**: Stable 60 FPS during I/O

### Measurement

```javascript
// Cycle rate benchmark
const startTime = performance.now();
const startCycles = emulator.cycles();
setTimeout(() => {
    const elapsed = performance.now() - startTime;
    const cycles = emulator.cycles() - startCycles;
    console.log(`Cycle rate: ${(cycles / elapsed * 1000).toFixed(0)} Hz`);
}, 1000);
```

---

## Known Limitations

### Architecture

- **No Mutable Device Access**: Cannot get mutable reference to UART through MemoryBus
  - Workaround: Store UART separately in WASM wrapper
  - Future: Add `get_device_mut()` to MappedMemory

- **Callback Lifetime**: JavaScript function must outlive WASM module
  - Use closure to capture callback reference
  - Store in emulator struct to prevent drop

### Browser

- **CORS**: Must serve with proper MIME types
- **HTTPS**: Some browsers require secure context for WASM
- **Mobile**: Virtual keyboard may need special handling

---

## Success Criteria

All test cases TC-01 through TC-12 must pass on:
- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)

No console errors or warnings during any test.

Performance benchmarks within target ranges.

---

## Sign-Off

**Tester**: ___________________
**Date**: ___________________
**Browser Versions Tested**:
- Chrome: ___
- Firefox: ___
- Safari: ___

**Overall Result**: PASS / FAIL

**Notes**:
