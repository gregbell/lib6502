# lib6502 Ergonomics Research

## Overview

This document captures findings from analyzing the c64-emu and c64-demo codebases to identify opportunities for improving the lib6502 core library. The goal is to make the library more ergonomic for building retro computer emulators while reducing boilerplate and friction.

## Analysis Scope

**Analyzed Codebases:**
- `c64-emu/` - Rust WASM emulator (~16,400 lines)
- `c64-demo/` - JavaScript frontend (~4,000 lines)
- `src/` - lib6502 core library

**Key Files Examined:**
- `c64-emu/src/system/c64_memory.rs` (533 lines) - Bank-switching memory bus
- `c64-emu/src/system/c64_system.rs` (1,050 lines) - System orchestration
- `c64-emu/src/devices/vic_ii.rs` (4,518 lines) - Video chip
- `c64-emu/src/devices/sid.rs` (3,003 lines) - Audio chip
- `c64-emu/src/devices/cia.rs` (736 lines) - Timer/I/O chips
- `c64-emu/src/wasm.rs` (833 lines) - WASM bindings
- `c64-demo/c64.js` (2,557 lines) - JavaScript frontend

---

## Identified Pain Points

### 1. Immutable Memory Access Missing

**Location:** `c64-emu/src/system/c64_system.rs`

**Problem:** lib6502 CPU only provides `memory_mut(&mut self)`, forcing mutable borrows even for read-only operations.

**Evidence:**
```rust
// c64_system.rs has awkward dual implementations:
pub fn roms_loaded(&self) -> bool {
    // Note: We need mutable access because lib6502 doesn't have memory() getter
    false // Can't implement properly without mutable borrow!
}

pub fn roms_loaded_mut(&mut self) -> bool {
    self.cpu.memory_mut().roms_loaded()
}
```

**Impact:**
- Forces `&mut self` where `&self` would suffice
- Prevents concurrent read access patterns
- Complicates API design in downstream crates

---

### 2. Read-Side Effects Not Supported

**Location:** `c64-emu/src/devices/vic_ii.rs`, `c64-emu/src/devices/cia.rs`

**Problem:** `Device::read(&self, offset: u16) -> u8` is immutable, but real hardware clears registers on read.

**Affected Registers:**
- VIC-II `$D01E` (sprite-sprite collision) - clears on read
- VIC-II `$D01F` (sprite-background collision) - clears on read
- CIA1 `$DC0D` (ICR) - clears interrupt flags on read
- CIA2 `$DD0D` (ICR) - clears interrupt flags on read

**Current Workaround in VIC-II:**
```rust
fn read(&self, offset: u16) -> u8 {
    0x1E | 0x1F => {
        // Note: We can't clear in immutable read, so collision
        // clearing will be handled specially by C64Memory
        self.sprite_collision_ss
    }
}
```

**Impact:**
- Requires special-case handling in memory bus
- Breaks device encapsulation
- CIA1 ICR read-clear behavior requires interior mutability (`RefCell`)

---

### 3. Manual Device Clocking

**Location:** `c64-emu/src/system/c64_system.rs:step_frame()`

**Problem:** Every device must be manually clocked in the emulation loop.

**Current Code:**
```rust
// Repeated boilerplate in step_frame() loop:
for _ in 0..cycles {
    self.cpu.memory_mut().cia1.clock();
    self.cpu.memory_mut().cia2.clock();
    self.cpu.memory_mut().sid.clock();
}
```

**Impact:**
- Easy to forget to clock a device
- Ordering dependencies not explicit
- Difficult to add/remove devices dynamically
- Clock domain differences (e.g., SID runs at different rate) require manual handling

---

### 4. Register Mirroring Boilerplate

**Location:** `c64-emu/src/devices/vic_ii.rs`, `c64-emu/src/devices/cia.rs`

**Problem:** Many 6502-era chips mirror registers across their address space.

**Examples:**
- VIC-II: 47 registers mirror every 64 bytes ($D000-$D3FF)
- CIA: 16 registers mirror every 16 bytes
- SID: 29 registers mirror across space

**Repeated Pattern:**
```rust
fn read(&self, offset: u16) -> u8 {
    let effective_offset = offset & 0x3F; // Mirror every 64 bytes
    match effective_offset as usize {
        n if n < REGISTER_COUNT => self.registers[n],
        _ => 0xFF, // Unmapped
    }
}
```

**Impact:**
- Repeated masking logic in every device
- Easy to get wrong (off-by-one, wrong mask)
- Not self-documenting

---

### 5. Device Composition Complexity

**Location:** `c64-emu/src/system/c64_memory.rs`

**Problem:** CIA1 Port B combines multiple input sources (keyboard matrix, joystick, external).

**Current Code:**
```rust
// In C64Memory::read() for $DC01:
if offset == 0x01 {
    let col_select = self.cia1.port_a.output();
    let kb_rows = self.keyboard.scan(col_select);
    let combined = self.cia1.external_b & kb_rows;
    self.cia1.port_b.read(combined)
}
```

**Impact:**
- Device logic bleeds into memory bus
- Hard to test devices in isolation
- Input source management is ad-hoc

---

### 6. Interrupt Source Management

**Location:** `c64-emu/src/system/c64_memory.rs`

**Problem:** Multiple devices can trigger IRQ/NMI - must be manually aggregated.

**Current Implementation:**
```rust
impl MemoryBus for C64Memory {
    fn irq_active(&self) -> bool {
        self.cia1.has_interrupt() || self.vic.has_interrupt()
    }

    fn nmi_active(&self) -> bool {
        self.cia2.has_interrupt()
    }
}
```

**Impact:**
- Must update memory bus when adding new interrupt sources
- No distinction between edge/level-sensitive sources
- Interrupt priority not handled

---

### 7. No Standard Video Output Interface

**Location:** `c64-emu/src/devices/vic_ii.rs`, `c64-emu/src/wasm.rs`

**Problem:** VIC-II maintains a framebuffer that must be extracted via custom methods.

**Current API:**
```rust
// VIC-II has custom methods:
impl VicII {
    pub fn get_framebuffer(&self) -> &[u8] { ... }
    pub fn get_border_color(&self) -> u8 { ... }
}

// WASM wrapper exposes:
#[wasm_bindgen]
pub fn get_framebuffer(&self) -> Vec<u8> { ... }
pub fn framebuffer_ptr(&self) -> *const u8 { ... }
```

**Impact:**
- No standard way to get video output from a device
- Each emulator reinvents framebuffer extraction
- Palette handling varies between implementations

---

### 8. No Standard Audio Output Interface

**Location:** `c64-emu/src/devices/sid.rs`, `c64-emu/src/wasm.rs`

**Problem:** SID generates audio samples with no standard extraction pattern.

**Current API:**
```rust
// SID has custom methods:
impl Sid {
    pub fn get_samples(&mut self) -> Vec<f32> { ... }
    pub fn set_sample_rate(&mut self, rate: u32) { ... }
}
```

**Impact:**
- No standard audio device trait
- Sample rate configuration ad-hoc
- Buffer management varies

---

### 9. Save State Complexity

**Location:** `c64-emu/src/system/save_state.rs` (1,350 lines)

**Problem:** Serializing full emulator state requires extensive boilerplate.

**What Must Be Serialized:**
- CPU state (registers, flags, cycle count)
- All device states (VIC-II, SID, CIA, etc.)
- Memory contents (RAM, banked regions)
- System timing state

**Impact:**
- 1,350+ lines just for save/load
- Version compatibility handling
- Easy to miss fields when adding features

---

### 10. WASM Framebuffer Conversion

**Location:** `c64-demo/c64.js:renderFrame()`

**Problem:** JavaScript must convert indexed colors to RGBA every frame.

**Current JavaScript:**
```javascript
renderFrame() {
    const fb = this.emulator.get_framebuffer(); // 64KB indexed
    const imageData = new ImageData(320, 200);
    for (let i = 0; i < 64000; i++) {
        const color = PALETTE[fb[i]];
        imageData.data[i*4+0] = (color >> 16) & 0xFF; // R
        imageData.data[i*4+1] = (color >> 8) & 0xFF;  // G
        imageData.data[i*4+2] = color & 0xFF;         // B
        imageData.data[i*4+3] = 255;                  // A
    }
    ctx.putImageData(imageData, 0, 0);
}
```

**Impact:**
- 256KB write per frame in JavaScript
- Could be done faster in WASM
- Repeated across emulator implementations

---

## Proposed Improvements

### Priority 1: High Impact, Low Effort

#### 1.1 Add Immutable Memory Access

**Change:** Add `memory(&self) -> &M` to CPU struct.

**Implementation:**
```rust
impl<M: MemoryBus> CPU<M> {
    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }
}
```

**Benefits:**
- Eliminates `_mut` workaround methods
- Enables concurrent read patterns
- Zero runtime cost

---

#### 1.2 Register Mirroring Wrapper

**Change:** Add generic mirroring device wrapper.

**Implementation:**
```rust
/// Wraps a device and applies address mirroring
pub struct MirroredDevice<D: Device> {
    inner: D,
    mask: u16,
}

impl<D: Device> MirroredDevice<D> {
    pub fn new(inner: D, mirror_size: u16) -> Self {
        Self {
            inner,
            mask: mirror_size - 1,
        }
    }
}

impl<D: Device> Device for MirroredDevice<D> {
    fn read(&self, offset: u16) -> u8 {
        self.inner.read(offset & self.mask)
    }

    fn write(&mut self, offset: u16, value: u8) {
        self.inner.write(offset & self.mask, value)
    }

    fn size(&self) -> u16 {
        self.inner.size()
    }
}
```

**Usage:**
```rust
let vic = MirroredDevice::new(VicII::new(), 64); // Mirror every 64 bytes
```

---

### Priority 2: Medium Impact, Medium Effort

#### 2.1 Read-Side Effect Support

**Option A: Mutable Read Method**
```rust
pub trait Device {
    fn read(&self, offset: u16) -> u8;

    /// Read with potential side effects (e.g., clearing flags)
    fn read_mut(&mut self, offset: u16) -> u8 {
        self.read(offset) // Default: no side effects
    }

    fn write(&mut self, offset: u16, value: u8);
    // ...
}
```

**Option B: Post-Read Callback**
```rust
pub trait Device {
    fn read(&self, offset: u16) -> u8;

    /// Called after read completes, allows clearing flags
    fn post_read(&mut self, offset: u16) {
        // Default: no-op
    }
}
```

**Option C: Interior Mutability (Current Workaround)**
```rust
pub struct VicII {
    collision_ss: Cell<u8>, // Clears on read
}

fn read(&self, offset: u16) -> u8 {
    match offset {
        0x1E => {
            let val = self.collision_ss.get();
            self.collision_ss.set(0);
            val
        }
        // ...
    }
}
```

**Recommendation:** Option A provides clearest intent and matches hardware behavior.

---

#### 2.2 Clockable Device Trait

**Change:** Add clocking trait and registry.

**Implementation:**
```rust
/// Trait for devices that need periodic clocking
pub trait Clockable {
    /// Advance device state by one clock cycle
    fn clock(&mut self);

    /// Clock divisor relative to CPU (default: 1:1)
    fn clock_divisor(&self) -> u32 { 1 }
}

/// Extended MappedMemory with automatic clocking
impl MappedMemory {
    /// Clock all registered Clockable devices
    pub fn clock_devices(&mut self, cpu_cycles: u64) {
        for device in &mut self.clockable_devices {
            let device_cycles = cpu_cycles / device.clock_divisor() as u64;
            for _ in 0..device_cycles {
                device.clock();
            }
        }
    }

    /// Register a clockable device
    pub fn add_clockable_device<D: Device + Clockable + 'static>(
        &mut self,
        base_addr: u16,
        device: D,
    ) -> Result<(), MemoryMapError> {
        // ...
    }
}
```

---

#### 2.3 Interrupt Controller

**Change:** Centralize interrupt source management.

**Implementation:**
```rust
/// Manages multiple interrupt sources
pub struct InterruptController {
    irq_sources: Vec<Weak<RefCell<dyn Device>>>,
    nmi_sources: Vec<Weak<RefCell<dyn Device>>>,
}

impl InterruptController {
    pub fn new() -> Self { ... }

    pub fn add_irq_source(&mut self, device: Weak<RefCell<dyn Device>>) {
        self.irq_sources.push(device);
    }

    pub fn add_nmi_source(&mut self, device: Weak<RefCell<dyn Device>>) {
        self.nmi_sources.push(device);
    }

    /// Returns true if any IRQ source is active (logical OR)
    pub fn irq_active(&self) -> bool {
        self.irq_sources.iter().any(|src| {
            src.upgrade()
                .map(|d| d.borrow().has_interrupt())
                .unwrap_or(false)
        })
    }

    /// Returns true if any NMI source is active
    pub fn nmi_active(&self) -> bool {
        self.nmi_sources.iter().any(|src| {
            src.upgrade()
                .map(|d| d.borrow().has_interrupt())
                .unwrap_or(false)
        })
    }
}
```

---

#### 2.4 Display Device Trait

**Change:** Standardize video output interface.

**Implementation:**
```rust
/// Trait for devices that produce video output
pub trait DisplayDevice: Device {
    /// Framebuffer width in pixels
    fn width(&self) -> u32;

    /// Framebuffer height in pixels
    fn height(&self) -> u32;

    /// Get indexed color framebuffer (one byte per pixel)
    fn framebuffer(&self) -> &[u8];

    /// Get palette as RGBA values (index -> 0xRRGGBBAA)
    fn palette(&self) -> &[u32];

    /// Get current border color index (if applicable)
    fn border_color(&self) -> Option<u8> { None }
}

/// WASM helper for framebuffer conversion
#[cfg(feature = "wasm")]
pub fn framebuffer_to_rgba(
    indexed: &[u8],
    palette: &[u32],
    output: &mut [u8],
) {
    for (i, &color_index) in indexed.iter().enumerate() {
        let rgba = palette[color_index as usize];
        output[i * 4 + 0] = ((rgba >> 24) & 0xFF) as u8;
        output[i * 4 + 1] = ((rgba >> 16) & 0xFF) as u8;
        output[i * 4 + 2] = ((rgba >> 8) & 0xFF) as u8;
        output[i * 4 + 3] = (rgba & 0xFF) as u8;
    }
}
```

---

#### 2.5 Audio Device Trait

**Change:** Standardize audio output interface.

**Implementation:**
```rust
/// Trait for devices that produce audio output
pub trait AudioDevice: Device {
    /// Set output sample rate (e.g., 44100)
    fn set_sample_rate(&mut self, rate: u32);

    /// Get current sample rate
    fn sample_rate(&self) -> u32;

    /// Get buffered audio samples (mono f32, -1.0 to 1.0)
    fn get_samples(&mut self) -> Vec<f32>;

    /// Number of channels (1 = mono, 2 = stereo)
    fn channels(&self) -> u8 { 1 }

    /// Enable/disable audio generation (for muting)
    fn set_enabled(&mut self, enabled: bool);

    /// Check if audio generation is enabled
    fn is_enabled(&self) -> bool;
}
```

---

### Priority 3: High Impact, High Effort

#### 3.1 Save State Framework

**Change:** Add serialization traits and derive macros.

**Implementation:**
```rust
/// Trait for types that can be saved/restored
pub trait Stateful {
    /// Serialize state to bytes
    fn save_state(&self) -> Vec<u8>;

    /// Restore state from bytes
    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError>;

    /// Version identifier for compatibility checking
    fn state_version(&self) -> u32;
}

/// Errors during state operations
#[derive(Debug)]
pub enum StateError {
    VersionMismatch { expected: u32, found: u32 },
    InvalidData(String),
    InsufficientData { expected: usize, found: usize },
}

/// CPU implements Stateful
impl<M: MemoryBus + Stateful> Stateful for CPU<M> {
    fn save_state(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(&self.a.to_le_bytes());
        data.extend(&self.x.to_le_bytes());
        data.extend(&self.y.to_le_bytes());
        data.extend(&self.pc.to_le_bytes());
        data.extend(&self.sp.to_le_bytes());
        // ... flags, cycles, memory state
        data.extend(self.memory.save_state());
        data
    }

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        // ...
    }

    fn state_version(&self) -> u32 { 1 }
}
```

**Future Enhancement:** Derive macro for automatic implementation.

---

#### 3.2 Device Composition Framework

**Change:** Formalize how devices can combine inputs.

**Implementation:**
```rust
/// An input source that can be combined with others
pub trait InputSource {
    fn read(&self) -> u8;
}

/// Combines multiple input sources with AND logic (active-low)
pub struct CombinedInput {
    sources: Vec<Box<dyn InputSource>>,
}

impl CombinedInput {
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    pub fn add_source(&mut self, source: Box<dyn InputSource>) {
        self.sources.push(source);
    }

    pub fn read(&self) -> u8 {
        self.sources.iter()
            .map(|s| s.read())
            .fold(0xFF, |acc, v| acc & v) // AND all sources
    }
}

// Example: Keyboard as InputSource
impl InputSource for KeyboardMatrix {
    fn read(&self) -> u8 {
        self.scan(self.column_select)
    }
}
```

---

#### 3.3 Bank Switching Support

**Change:** Generic banked memory device.

**Implementation:**
```rust
/// Memory region that can switch between multiple banks
pub struct BankedMemory {
    banks: Vec<Box<dyn Device>>,
    active_bank: usize,
    base_size: u16,
}

impl BankedMemory {
    pub fn new(base_size: u16) -> Self {
        Self {
            banks: Vec::new(),
            active_bank: 0,
            base_size,
        }
    }

    pub fn add_bank(&mut self, device: Box<dyn Device>) -> usize {
        let index = self.banks.len();
        self.banks.push(device);
        index
    }

    pub fn set_active_bank(&mut self, index: usize) {
        if index < self.banks.len() {
            self.active_bank = index;
        }
    }

    pub fn active_bank(&self) -> usize {
        self.active_bank
    }
}

impl Device for BankedMemory {
    fn read(&self, offset: u16) -> u8 {
        self.banks.get(self.active_bank)
            .map(|b| b.read(offset))
            .unwrap_or(0xFF)
    }

    fn write(&mut self, offset: u16, value: u8) {
        if let Some(bank) = self.banks.get_mut(self.active_bank) {
            bank.write(offset, value);
        }
    }

    fn size(&self) -> u16 {
        self.base_size
    }
}
```

---

### Priority 4: Low Impact / Nice-to-Have

#### 4.1 Debugging Helpers

```rust
/// Breakpoint types
pub enum Breakpoint {
    Address(u16),
    Opcode(u8),
    MemoryRead(u16),
    MemoryWrite(u16),
}

/// Step result with debug info
pub enum StepResult {
    Ok,
    BreakpointHit(Breakpoint),
    WatchpointHit { addr: u16, old: u8, new: u8 },
    IllegalOpcode(u8),
}

impl<M: MemoryBus> CPU<M> {
    pub fn add_breakpoint(&mut self, bp: Breakpoint);
    pub fn remove_breakpoint(&mut self, bp: &Breakpoint);
    pub fn clear_breakpoints(&mut self);
    pub fn step_with_debug(&mut self) -> StepResult;
}
```

---

#### 4.2 Trace Logging

```rust
/// Instruction trace entry
pub struct TraceEntry {
    pub pc: u16,
    pub opcode: u8,
    pub operand: [u8; 2],
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub flags: u8,
    pub cycles: u64,
}

impl<M: MemoryBus> CPU<M> {
    pub fn enable_trace(&mut self, buffer_size: usize);
    pub fn disable_trace(&mut self);
    pub fn get_trace(&self) -> &[TraceEntry];
    pub fn clear_trace(&mut self);
}
```

---

## Summary Matrix

| Feature | Priority | Impact | Effort | Dependencies |
|---------|----------|--------|--------|--------------|
| Immutable memory access | 1 | High | Low | None |
| Register mirroring wrapper | 1 | Medium | Low | None |
| Read-side effects | 2 | High | Medium | None |
| Clockable trait | 2 | High | Medium | None |
| Interrupt controller | 2 | Medium | Medium | Shared device refs |
| Display device trait | 2 | Medium | Low | None |
| Audio device trait | 2 | Medium | Low | None |
| Save state framework | 3 | High | High | All device traits |
| Device composition | 3 | Medium | High | None |
| Bank switching | 3 | Medium | Medium | None |
| Debugging helpers | 4 | Low | Medium | None |
| Trace logging | 4 | Low | Low | None |

---

## Recommended Implementation Order

1. **Phase 1 (Quick Wins)**
   - Add `memory()` getter to CPU
   - Add `MirroredDevice` wrapper

2. **Phase 2 (Core Improvements)**
   - Add `read_mut()` to Device trait
   - Add `Clockable` trait and registry
   - Add `DisplayDevice` trait
   - Add `AudioDevice` trait

3. **Phase 3 (Advanced Features)**
   - Implement `InterruptController`
   - Add `BankedMemory` device
   - Design save state framework

4. **Phase 4 (Developer Experience)**
   - Add debugging helpers
   - Add trace logging
   - Documentation and examples

---

## Appendix: C64-Emu Code Distribution

| Component | Lines | % of Total |
|-----------|-------|------------|
| VIC-II (video) | 4,518 | 27.5% |
| SID (audio) | 3,003 | 18.3% |
| Disk Drive | 2,307 | 14.1% |
| Save State | 1,350 | 8.2% |
| C64System | 1,050 | 6.4% |
| WASM bindings | 833 | 5.1% |
| CIA (timers) | 736 | 4.5% |
| Keyboard | 689 | 4.2% |
| Memory bus | 533 | 3.2% |
| IEC Bus | 473 | 2.9% |
| Joystick | 390 | 2.4% |
| Port 6510 | 254 | 1.5% |
| Color RAM | 183 | 1.1% |
| **Total** | **~16,400** | 100% |

---

## References

- `c64-emu/src/system/c64_memory.rs` - Bank switching, device integration
- `c64-emu/src/system/c64_system.rs` - Main loop, device clocking
- `c64-emu/src/devices/*.rs` - Device implementations
- `c64-emu/src/wasm.rs` - WASM API patterns
- `c64-demo/c64.js` - Frontend integration patterns
- `src/devices/mod.rs` - Current Device trait
- `src/memory.rs` - Current MemoryBus trait
