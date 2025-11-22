# Quick Start Guide: C64 Emulator Demo

**Feature**: Commodore 64 Emulator Web Demo
**Branch**: `007-c64-emulator-demo`
**Date**: 2025-11-20

This guide walks you through building and running the C64 emulator demo in your browser.

---

## Prerequisites

### Required Software

1. **Rust 1.75+** with `wasm32-unknown-unknown` target
   ```bash
   # Install Rust (if needed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Add WASM target
   rustup target add wasm32-unknown-unknown
   ```

2. **wasm-pack** (WASM build tool)
   ```bash
   cargo install wasm-pack
   ```

3. **Local web server** (any of these):
   - Python: `python3 -m http.server`
   - Node.js: `npx http-server`
   - Rust: `cargo install basic-http-server`

### Verify Installation

```bash
rustc --version  # Should show 1.75.0 or higher
wasm-pack --version  # Should show 0.12.0 or higher
python3 --version  # Or your preferred server
```

---

## Step 1: Clone and Build

### Clone Repository

```bash
git clone https://github.com/your-username/wt-6502--c64.git
cd wt-6502--c64
git checkout 007-c64-emulator-demo
```

### Build WASM Module

```bash
# Build for web target with WASM feature enabled
wasm-pack build --target web --features wasm --out-dir demo/c64/pkg

# This creates:
# demo/c64/pkg/lib6502.js         - JavaScript bindings
# demo/c64/pkg/lib6502_bg.wasm   - Compiled WASM module
# demo/c64/pkg/lib6502.d.ts      - TypeScript definitions
```

**Build time**: ~30-60 seconds for release build
**Output size**: ~500KB WASM file (uncompressed)

---

## Step 2: Obtain C64 ROM Files

### Option A: MEGA65 OpenROMs (Recommended - Legal & Free)

1. Download OpenROMs from official repository:
   ```bash
   cd demo/c64
   mkdir -p roms

   # Download MEGA65 OpenROMs (example URLs - verify latest)
   curl -L -o roms/basic.bin https://github.com/MEGA65/open-roms/raw/master/bin/openroms-c64-basic.bin
   curl -L -o roms/kernal.bin https://github.com/MEGA65/open-roms/raw/master/bin/openroms-c64-kernal.bin
   curl -L -o roms/chargen.bin https://github.com/MEGA65/open-roms/raw/master/bin/openroms-c64-chargen.bin
   ```

2. Verify ROM sizes:
   ```bash
   ls -lh roms/
   # Expected:
   # basic.bin   - 8KB (8192 bytes)
   # kernal.bin  - 8KB (8192 bytes)
   # chargen.bin - 4KB (4096 bytes)
   ```

### Option B: Provide Your Own ROMs

If you have legitimate C64 ROM files from another source:

```bash
cp /path/to/your/c64-basic.rom demo/c64/roms/basic.bin
cp /path/to/your/c64-kernal.rom demo/c64/roms/kernal.bin
cp /path/to/your/c64-chargen.rom demo/c64/roms/chargen.bin
```

**Important**: Ensure files are exactly 8KB, 8KB, and 4KB respectively.

---

## Step 3: Generate Character Atlas

The character atlas is a pre-rendered PNG of all 256 PETSCII characters for efficient rendering.

### Option A: Use Provided Script (Recommended)

```bash
cd demo/c64
node generate-atlas.js

# This reads roms/chargen.bin and outputs:
# assets/chargen-atlas.png (128×128 or 256×256 pixels)
```

### Option B: Manual Generation (If Script Not Available)

If the generator script doesn't exist yet, you can:
1. Load the demo—it will render characters directly from ROM (slower but functional)
2. Or wait for implementation to include the atlas generation tool

---

## Step 4: Start Local Web Server

```bash
# From demo/c64/ directory
cd demo/c64

# Choose your preferred server:

# Python (most common)
python3 -m http.server 8000

# Node.js
npx http-server -p 8000

# Rust basic-http-server
basic-http-server -a 127.0.0.1:8000 .

# You should see:
# Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...
```

**Important**: Must serve from `demo/c64/` directory so paths resolve correctly.

---

## Step 5: Open in Browser

1. Open your browser to: **http://localhost:8000**

2. **Expected behavior**:
   - Page loads HTML/CSS/JavaScript
   - Fetches WASM module (~500KB, may take 1-2 seconds)
   - Loads ROM files (20KB total, fast)
   - Initializes emulator
   - Screen displays boot sequence:
     ```
         **** COMMODORE 64 BASIC V2 ****

      64K RAM SYSTEM  38911 BASIC BYTES FREE

     READY.
     █
     ```

3. **Boot time**: 2-3 seconds total (including network fetches)

---

## Step 6: Test the Emulator

### Try BASIC Commands

**Simple Math**:
```basic
PRINT 2+2
```
Expected output: `4`

**Hello World**:
```basic
PRINT "HELLO, WORLD!"
```
Expected output: `HELLO, WORLD!`

**Program Entry**:
```basic
10 PRINT "HELLO"
20 GOTO 10
RUN
```
Expected: Scrolling "HELLO" messages. Press `RUN/STOP` (ESC key) to stop.

### Test Keyboard Mapping

| Modern Key | C64 Function | Test |
|-----------|--------------|------|
| **Enter** | RETURN | Type command + Enter |
| **Backspace** | DEL | Delete characters |
| **ESC** | RUN/STOP | Stop running program |
| **F1, F3, F5, F7** | Function keys | Display help/shortcuts |
| **Shift** | Shift | Type uppercase letters |
| **Tab** | CTRL | Control key (rarely used) |

### Verify Display Colors

- **Border**: Light blue (#6C5EB5)
- **Background**: Blue (#6C9FB5)
- **Text**: Light blue (#6C9FB5)
- **Cursor**: Blinking block █

---

## Troubleshooting

### Issue: "Failed to fetch WASM module"

**Cause**: CORS policy or wrong directory
**Solution**:
```bash
# Ensure you're in demo/c64/ when starting server
cd demo/c64
python3 -m http.server 8000
```

### Issue: "ROM size mismatch"

**Cause**: ROM files are wrong size or corrupted
**Solution**:
```bash
# Check file sizes
ls -l roms/
# basic.bin should be exactly 8192 bytes
# kernal.bin should be exactly 8192 bytes
# chargen.bin should be exactly 4096 bytes

# Re-download if sizes are wrong
```

### Issue: Screen is blank/black

**Possible causes**:
1. VIC-II not initialized (check browser console for errors)
2. ROM files missing or failed to load
3. JavaScript error before rendering starts

**Solution**:
```bash
# Open browser DevTools (F12)
# Check Console tab for errors
# Look for messages like:
#   - "Error loading ROMs"
#   - "VIC-II initialization failed"
#   - "WASM initialization error"
```

### Issue: Keyboard not responding

**Cause**: Canvas element doesn't have focus
**Solution**: Click on the emulator display area

### Issue: Very slow performance

**Possible causes**:
1. Debug build instead of release
2. Old browser version
3. Browser extensions interfering

**Solution**:
```bash
# Rebuild with release optimizations
wasm-pack build --target web --features wasm --release --out-dir demo/c64/pkg

# Try different browser (Chrome/Firefox recommended)
# Disable browser extensions temporarily
```

### Issue: Characters look blurry/anti-aliased

**Cause**: CSS or canvas scaling with interpolation
**Solution**: Check that JavaScript sets:
```javascript
ctx.imageSmoothingEnabled = false;
```

---

## Development Workflow

### Modify Rust Code

```bash
# 1. Edit source files in src/devices/
vim src/devices/vic2.rs

# 2. Rebuild WASM
wasm-pack build --target web --features wasm --out-dir demo/c64/pkg

# 3. Refresh browser (Ctrl+Shift+R for hard refresh)
```

### Modify JavaScript/HTML

```bash
# 1. Edit files in demo/c64/
vim demo/c64/app.js

# 2. Refresh browser (Ctrl+R or Cmd+R)
# No rebuild needed - JavaScript is interpreted
```

### Debug Tips

**Browser Console**:
```javascript
// Access emulator instance (if exposed globally)
window.emulator.get_pc();  // Check program counter
window.emulator.get_a();   // Check accumulator
window.emulator.step();    // Execute one instruction
```

**Rust Logging** (add to Cargo.toml dev-dependencies):
```toml
[dependencies]
console_error_panic_hook = { version = "0.1", optional = true }

[features]
wasm = ["wasm-bindgen", "js-sys", "console_error_panic_hook"]
```

```rust
// In lib.rs WASM code
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
```

---

## Next Steps

### Explore Code

- **VIC-II Device**: `src/devices/vic2.rs` - Video chip emulation
- **CIA Device**: `src/devices/cia.rs` - Keyboard & timers
- **WASM Interface**: `src/wasm.rs` - Rust-to-JavaScript bridge
- **Display Rendering**: `demo/c64/components/display.js`
- **Keyboard Handling**: `demo/c64/components/keyboard.js`

### Try Advanced Programs

**Classic Programs** (from C64 software libraries):
- BASIC games: Hunt the Wumpus, Lunar Lander
- Graphics demos: Scrolling text, character animations
- Utilities: Memory peek/poke, hex editor

**BASIC Examples**:
```basic
10 FOR I=1 TO 10
20 PRINT "NUMBER "; I
30 NEXT I
```

```basic
10 X=0
20 PRINT CHR$(147);  : REM Clear screen
30 X=X+1
40 PRINT "COUNT: "; X
50 GOTO 30
```

### Contribute

Found a bug? Want to add features?
- Check open issues: https://github.com/your-username/wt-6502--c64/issues
- Read CONTRIBUTING.md for guidelines
- Submit pull requests for improvements

---

## Reference Links

**Documentation**:
- [AGENTS.md](../../AGENTS.md) - Project architecture
- [specs/007-c64-emulator-demo/spec.md](./spec.md) - Feature specification
- [specs/007-c64-emulator-demo/plan.md](./plan.md) - Implementation plan

**C64 Resources**:
- [C64 Wiki](https://www.c64-wiki.com/) - Hardware reference
- [C64 BASIC Manual](https://www.c64-wiki.com/wiki/BASIC) - BASIC programming
- [VIC-II Documentation](http://www.zimmers.net/cbmpics/cbm/c64/vic-ii.txt)
- [CIA 6526 Datasheet](http://archive.6502.org/datasheets/mos_6526_cia.pdf)

**WASM Development**:
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)

---

## Quick Reference Commands

```bash
# Build WASM
wasm-pack build --target web --features wasm --out-dir demo/c64/pkg

# Start server
cd demo/c64 && python3 -m http.server 8000

# Run tests
cargo test

# Check ROM sizes
ls -lh demo/c64/roms/

# Clean build
cargo clean
rm -rf demo/c64/pkg
```

---

**Quick Start Status**: ✅ Complete
**Happy hacking!** If you encounter issues, check the GitHub issues page or open a new issue with your browser console output.
