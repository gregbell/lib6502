# Quickstart: Building and Running the WASM Demo

**Audience**: Developers who want to build and test the web demo locally
**Prerequisites**: Rust 1.75+, basic command line knowledge
**Time**: ~10 minutes

## Setup (One-Time)

### 1. Install Rust and WASM Toolchain

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack
```

### 2. Verify Installation

```bash
# Check Rust version
rustc --version  # Should be 1.75 or newer

# Check WASM target
rustup target list --installed | grep wasm32

# Check wasm-pack
wasm-pack --version
```

##Build the Demo

### 3. Build WASM Module

From the repository root:

```bash
# Build optimized WASM for web deployment
wasm-pack build --target web --out-dir demo/lib6502_wasm

# This generates:
# - demo/lib6502_wasm/lib6502_wasm_bg.wasm (the binary)
# - demo/lib6502_wasm/lib6502_wasm.js (JS glue code)
# - demo/lib6502_wasm/lib6502_wasm.d.ts (TypeScript definitions)
```

**Build Options**:
```bash
# Development build (faster, larger, includes debug symbols)
wasm-pack build --dev --target web --out-dir demo/lib6502_wasm

# Release build (slower, smaller, optimized)
wasm-pack build --release --target web --out-dir demo/lib6502_wasm

# Profile build (default, good balance)
wasm-pack build --target web --out-dir demo/lib6502_wasm
```

**Typical Build Times**:
- Development: ~5 seconds
- Release: ~30 seconds (includes wasm-opt)

**Expected Output Sizes**:
- Development: ~400KB
- Release: ~150KB (with optimizations)

### 4. Serve the Demo Locally

The demo must be served via HTTP (not `file://`) because WASM requires ES6 modules.

**Option A: Python HTTP Server** (simplest):
```bash
python3 -m http.server 8000 -d demo/

# Open http://localhost:8000 in your browser
```

**Option B: Node.js http-server**:
```bash
# Install (one-time)
npm install -g http-server

# Serve
http-server demo/ -p 8000

# Open http://localhost:8000
```

**Option C: Rust miniserve**:
```bash
# Install (one-time)
cargo install miniserve

# Serve
miniserve demo/ -p 8000

# Open http://localhost:8000
```

### 5. Test the Demo

1. Open browser to `http://localhost:8000`
2. You should see the 6502 demo interface
3. Try the example programs (dropdown menu)
4. Click "Run" or "Step" to execute
5. Observe registers and memory updates

**Troubleshooting**:

| Problem | Solution |
|---------|----------|
| "Module not found" | Verify `demo/lib6502_wasm/` directory exists and contains `.wasm` file |
| CORS error | Use HTTP server, not `file://` protocol |
| Blank page | Check browser console for JavaScript errors |
| WASM won't load | Ensure browser supports WASM (Chrome 57+, Firefox 52+, Safari 11+) |

## Development Workflow

### Make Code Changes

**For Rust/WASM changes**:
```bash
# 1. Edit Rust source (src/wasm/*.rs)
# 2. Rebuild WASM
wasm-pack build --target web --out-dir demo/lib6502_wasm

# 3. Refresh browser (Ctrl+Shift+R to clear cache)
```

**For Frontend changes** (HTML/CSS/JS):
```bash
# 1. Edit demo/*.html, demo/*.css, demo/*.js
# 2. Refresh browser (changes take effect immediately)
```

### Hot Reload Setup (Optional)

For faster iteration, use a dev server with live reload:

```bash
# Install browser-sync
npm install -g browser-sync

# Serve with auto-reload
browser-sync start --server demo --files "demo/**/*.html" "demo/**/*.css" "demo/**/*.js" "demo/lib6502_wasm/**/*.wasm"
```

**Note**: Still need to rebuild WASM manually after Rust changes.

### Run Tests

```bash
# Run Rust tests (including WASM module)
cargo test

# Run browser tests (requires wasm-pack test setup)
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

## Common Tasks

### Clear Build Cache

```bash
# Remove WASM build artifacts
rm -rf demo/lib6502_wasm

# Clean Cargo build cache
cargo clean

# Rebuild from scratch
wasm-pack build --target web --out-dir demo/lib6502_wasm
```

### Measure WASM Size

```bash
# Check file sizes
ls -lh demo/lib6502_wasm/*.wasm

# Detailed size breakdown
wasm-objdump -h demo/lib6502_wasm/lib6502_wasm_bg.wasm

# Analyze with twiggy (install: cargo install twiggy)
twiggy top demo/lib6502_wasm/lib6502_wasm_bg.wasm
```

### Optimize for Production

```bash
# Build with maximum optimizations
wasm-pack build --release --target web --out-dir demo/lib6502_wasm

# Further optimize with wasm-opt (already included in release build)
wasm-opt demo/lib6502_wasm/lib6502_wasm_bg.wasm -O3 -o demo/lib6502_wasm/lib6502_wasm_bg.wasm

# Enable LTO in Cargo.toml
# [profile.release]
# lto = true
```

## Deployment

### Deploy to GitHub Pages

**Manual deployment**:
```bash
# 1. Build release WASM
wasm-pack build --release --target web --out-dir demo/lib6502_wasm

# 2. Commit demo directory
git add demo/
git commit -m "Update WASM demo"
git push

# 3. Enable GitHub Pages in repo settings
# Settings → Pages → Source: "main" branch → /demo directory
```

**Automatic deployment** (GitHub Actions):

The workflow file (`.github/workflows/deploy-demo.yml`) automatically builds and deploys on push to main:

```bash
# Just push changes
git push origin main

# GitHub Actions will:
# 1. Build WASM
# 2. Deploy to GitHub Pages
# 3. Available at https://username.github.io/6502/
```

**View deployment status**:
- Go to repository → Actions tab
- Click latest workflow run
- View build logs and deployment status

## Debugging

### Browser DevTools

**JavaScript Console**:
```javascript
// Check WASM module loaded
console.log(typeof Emulator6502); // Should be "function"

// Create emulator
const emu = new Emulator6502();

// Inspect state
console.log('A:', emu.get_a());
console.log('PC:', emu.get_pc());
```

**Network Tab**:
- Verify `.wasm` file loads (should be ~150KB for release build)
- Check MIME type: `application/wasm`
- Verify no 404 errors

**Performance Tab**:
- Profile WASM execution
- Check frame rate during "Run" mode (should be 60fps)

### WASM-Specific Debugging

**Enable debug symbols** (development build):
```bash
wasm-pack build --dev --target web --out-dir demo/lib6502_wasm
```

**Chrome DevTools WASM debugging**:
1. Open DevTools → Sources tab
2. Navigate to `wasm://wasm/` to see disassembled WASM
3. Set breakpoints in WASM code
4. Step through execution

**Console logging from Rust**:
```rust
// Add to Cargo.toml dependencies:
// console_error_panic_hook = "0.1"
// web-sys = { version = "0.3", features = ["console"] }

use web_sys::console;

console::log_1(&"Debug message from Rust".into());
```

## Performance Tuning

### Measure Frame Rate

```javascript
// Add to app.js
let frameCount = 0;
let lastTime = performance.now();

function measureFPS() {
    frameCount++;
    const now = performance.now();
    if (now - lastTime >= 1000) {
        console.log('FPS:', frameCount);
        frameCount = 0;
        lastTime = now;
    }
    requestAnimationFrame(measureFPS);
}
measureFPS();
```

### Optimize WASM Calls

```javascript
// SLOW: Fetch registers individually every frame
function updateUI() {
    document.getElementById('reg-a').textContent = emu.get_a();
    document.getElementById('reg-x').textContent = emu.get_x();
    // ... (6 WASM calls)
}

// FASTER: Batch into single update
function updateUI() {
    const state = {
        a: emu.get_a(),
        x: emu.get_x(),
        y: emu.get_y(),
        pc: emu.get_pc(),
        sp: emu.get_sp(),
        cycles: emu.get_cycles()
    };
    // Update DOM in single batch
    updateRegisters(state);
}

// FASTEST: Only update changed values
let lastState = {};
function updateUI() {
    const state = getState();
    for (const [key, value] of Object.entries(state)) {
        if (lastState[key] !== value) {
            updateElement(key, value);
        }
    }
    lastState = state;
}
```

## Next Steps

- **Customize UI**: Edit `demo/styles.css` for visual tweaks
- **Add Examples**: Create new `.asm` files in `demo/examples/`
- **Extend API**: Add new methods to `src/wasm/api.rs`
- **Improve Performance**: Profile with browser DevTools

## Resources

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [MDN: WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)

## Getting Help

**Common Issues**:
- Check [GitHub Issues](https://github.com/username/6502/issues)
- Review [WASM API Contract](./contracts/wasm-api.md)
- Consult [Research Notes](./research.md)

**File Structure Reference**:
```
/home/greg/src/6502/
├── src/
│   ├── wasm/          # WASM bindings (Rust)
│   │   ├── mod.rs
│   │   ├── api.rs
│   │   └── memory.rs
├── demo/              # Web frontend
│   ├── index.html
│   ├── styles.css
│   ├── app.js
│   ├── components/
│   │   ├── editor.js
│   │   ├── registers.js
│   │   ├── memory.js
│   │   └── controls.js
│   ├── examples/
│   │   ├── counter.asm
│   │   └── fibonacci.asm
│   └── lib6502_wasm/ # Generated (gitignored)
└── .github/
    └── workflows/
        └── deploy-demo.yml
```
