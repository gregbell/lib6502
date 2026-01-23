/**
 * C64 Emulator - Main Application
 * Commodore 64 emulator running in the browser via WebAssembly
 */

// C64Emulator class reference (populated after WASM load)
let C64Emulator;

// C64 Color Palette (VICE emulator colors matching style.css)
const C64_PALETTE = [
    [0x00, 0x00, 0x00], // 0 - Black
    [0xFF, 0xFF, 0xFF], // 1 - White
    [0x68, 0x37, 0x2B], // 2 - Red
    [0x70, 0xA4, 0xB2], // 3 - Cyan
    [0x6F, 0x3D, 0x86], // 4 - Purple
    [0x58, 0x8D, 0x43], // 5 - Green
    [0x35, 0x28, 0x79], // 6 - Blue
    [0xB8, 0xC7, 0x6F], // 7 - Yellow
    [0x6F, 0x4F, 0x25], // 8 - Orange
    [0x43, 0x39, 0x00], // 9 - Brown
    [0x9A, 0x67, 0x59], // 10 - Light Red
    [0x44, 0x44, 0x44], // 11 - Dark Grey
    [0x6C, 0x6C, 0x6C], // 12 - Grey
    [0x9A, 0xD2, 0x84], // 13 - Light Green
    [0x6C, 0x5E, 0xB5], // 14 - Light Blue
    [0x95, 0x95, 0x95], // 15 - Light Grey
];

// ROM sizes for validation
const ROM_SIZES = {
    kernal: 8192,
    basic: 8192,
    charrom: 4096,
};

// LocalStorage keys for ROM caching
const ROM_STORAGE_KEYS = {
    kernal: 'c64_rom_kernal',
    basic: 'c64_rom_basic',
    charrom: 'c64_rom_charrom',
};

/**
 * Main C64 Emulator Application
 */
class C64App {
    constructor() {
        // WASM module reference
        this.wasm = null;
        this.wasmMemory = null;

        // Emulator instance
        this.emulator = null;

        // Emulation state
        this.running = false;
        this.paused = false;
        this.region = 'pal'; // 'pal' or 'ntsc'
        this.scale = 2;

        // Loaded ROMs
        this.roms = {
            kernal: null,
            basic: null,
            charrom: null,
        };

        // Display
        this.canvas = null;
        this.ctx = null;
        this.imageData = null;

        // Timing
        this.frameCount = 0;
        this.lastFpsUpdate = 0;
        this.fps = 0;
        this.animationFrameId = null;

        // Audio state (for future audio component)
        this.audioEnabled = false;
        this.volume = 0.5;
    }

    /**
     * Initialize the application
     */
    async init() {
        console.log('C64 Emulator initializing...');

        try {
            // Load WASM module
            await this.loadWasm();

            // Initialize display
            this.initDisplay();

            // Set up UI event handlers
            this.setupEventHandlers();

            // Check for cached ROMs
            await this.loadCachedRoms();

            // If all ROMs are loaded, show emulator section
            if (this.hasAllRoms()) {
                this.showEmulatorSection();
            }

            console.log('C64 Emulator initialized successfully');
        } catch (error) {
            console.error('Failed to initialize C64 Emulator:', error);
            this.showError(`Initialization failed: ${error.message}`);
        }
    }

    /**
     * Load the WebAssembly module
     */
    async loadWasm() {
        try {
            // Dynamic import of the WASM module
            // The path assumes wasm-pack output is in c64-emu/pkg/
            const wasmModule = await import('../c64-emu/pkg/c64_emu.js');

            // Initialize the WASM module
            await wasmModule.default();

            // Store references
            this.wasm = wasmModule;
            C64Emulator = wasmModule.C64Emulator;

            // Get access to WASM memory for framebuffer access
            this.wasmMemory = wasmModule.wasm_memory ? wasmModule.wasm_memory() : null;

            console.log('WASM module loaded successfully');
        } catch (error) {
            console.error('Failed to load WASM module:', error);
            throw new Error(`WASM loading failed: ${error.message}. Make sure to run 'wasm-pack build --target web --features wasm' in the c64-emu directory.`);
        }
    }

    /**
     * Initialize the display canvas
     */
    initDisplay() {
        this.canvas = document.getElementById('c64-screen');
        if (!this.canvas) {
            throw new Error('Canvas element not found');
        }

        this.ctx = this.canvas.getContext('2d');
        this.imageData = this.ctx.createImageData(320, 200);

        // Fill with C64 blue initially
        const blueColor = C64_PALETTE[6];
        for (let i = 0; i < 320 * 200; i++) {
            this.imageData.data[i * 4 + 0] = blueColor[0];
            this.imageData.data[i * 4 + 1] = blueColor[1];
            this.imageData.data[i * 4 + 2] = blueColor[2];
            this.imageData.data[i * 4 + 3] = 255;
        }
        this.ctx.putImageData(this.imageData, 0, 0);
    }

    /**
     * Set up UI event handlers
     */
    setupEventHandlers() {
        // ROM file inputs
        const romInputs = document.querySelectorAll('[data-rom-type]');
        romInputs.forEach(input => {
            input.addEventListener('change', (e) => this.handleRomFileSelect(e));
        });

        // Start button
        const startBtn = document.getElementById('start-emulator-btn');
        if (startBtn) {
            startBtn.addEventListener('click', () => this.startEmulator());
        }

        // Control buttons
        const resetBtn = document.getElementById('reset-btn');
        if (resetBtn) {
            resetBtn.addEventListener('click', () => this.reset());
        }

        const hardResetBtn = document.getElementById('hard-reset-btn');
        if (hardResetBtn) {
            hardResetBtn.addEventListener('click', () => this.hardReset());
        }

        const pauseBtn = document.getElementById('pause-btn');
        if (pauseBtn) {
            pauseBtn.addEventListener('click', () => this.togglePause());
        }

        // Mute button
        const muteBtn = document.getElementById('mute-btn');
        if (muteBtn) {
            muteBtn.addEventListener('click', () => this.toggleMute());
        }

        // Volume slider
        const volumeSlider = document.getElementById('volume-slider');
        if (volumeSlider) {
            volumeSlider.addEventListener('input', (e) => {
                this.volume = e.target.value / 100;
            });
        }

        // Region select
        const regionSelect = document.getElementById('region-select');
        if (regionSelect) {
            regionSelect.addEventListener('change', (e) => {
                this.setRegion(e.target.value);
            });
        }

        // Scale select
        const scaleSelect = document.getElementById('scale-select');
        if (scaleSelect) {
            scaleSelect.addEventListener('change', (e) => {
                this.setScale(e.target.value);
            });
        }

        // Keyboard events
        document.addEventListener('keydown', (e) => this.handleKeyDown(e));
        document.addEventListener('keyup', (e) => this.handleKeyUp(e));

        // Tab visibility for auto-pause
        document.addEventListener('visibilitychange', () => {
            if (document.hidden && this.running && !this.paused) {
                // Release all keys when tab loses focus
                if (this.emulator) {
                    this.emulator.release_all_keys();
                }
            }
        });

        // File drag and drop (placeholder for future file-loader component)
        const dropZone = document.getElementById('drop-zone');
        if (dropZone) {
            dropZone.addEventListener('click', () => {
                document.getElementById('file-input').click();
            });

            dropZone.addEventListener('dragover', (e) => {
                e.preventDefault();
                dropZone.classList.add('drag-over');
            });

            dropZone.addEventListener('dragleave', () => {
                dropZone.classList.remove('drag-over');
            });

            dropZone.addEventListener('drop', (e) => {
                e.preventDefault();
                dropZone.classList.remove('drag-over');
                // File handling will be in file-loader component
                console.log('File dropped:', e.dataTransfer.files);
            });
        }

        const fileInput = document.getElementById('file-input');
        if (fileInput) {
            fileInput.addEventListener('change', (e) => {
                // File handling will be in file-loader component
                console.log('File selected:', e.target.files);
            });
        }

        const fileBrowseBtn = document.getElementById('file-browse-btn');
        if (fileBrowseBtn) {
            fileBrowseBtn.addEventListener('click', () => {
                document.getElementById('file-input').click();
            });
        }
    }

    /**
     * Handle ROM file selection
     */
    async handleRomFileSelect(event) {
        const input = event.target;
        const romType = input.dataset.romType;
        const expectedSize = parseInt(input.dataset.expectedSize, 10);
        const statusEl = document.getElementById(`${romType}-status`);

        if (!input.files || !input.files[0]) {
            return;
        }

        const file = input.files[0];

        try {
            const data = await this.readFileAsArrayBuffer(file);
            const bytes = new Uint8Array(data);

            // Validate size
            if (bytes.length !== expectedSize) {
                throw new Error(`Invalid ${romType.toUpperCase()} ROM size: expected ${expectedSize} bytes, got ${bytes.length}`);
            }

            // Store ROM
            this.roms[romType] = bytes;

            // Cache to localStorage
            this.cacheRom(romType, bytes);

            // Update status
            if (statusEl) {
                statusEl.textContent = `✓ ${file.name} (${bytes.length} bytes)`;
                statusEl.className = 'rom-status success';
            }

            // Check if all ROMs are loaded
            this.updateStartButton();

            console.log(`${romType.toUpperCase()} ROM loaded: ${file.name}`);
        } catch (error) {
            console.error(`Failed to load ${romType} ROM:`, error);
            if (statusEl) {
                statusEl.textContent = `✗ ${error.message}`;
                statusEl.className = 'rom-status error';
            }
        }
    }

    /**
     * Read file as ArrayBuffer
     */
    readFileAsArrayBuffer(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result);
            reader.onerror = () => reject(new Error('Failed to read file'));
            reader.readAsArrayBuffer(file);
        });
    }

    /**
     * Cache ROM to localStorage
     */
    cacheRom(romType, data) {
        try {
            const base64 = this.arrayBufferToBase64(data);
            localStorage.setItem(ROM_STORAGE_KEYS[romType], base64);
            console.log(`${romType} ROM cached to localStorage`);
        } catch (error) {
            console.warn(`Failed to cache ${romType} ROM:`, error);
        }
    }

    /**
     * Load cached ROMs from localStorage
     */
    async loadCachedRoms() {
        for (const romType of Object.keys(ROM_STORAGE_KEYS)) {
            const cached = localStorage.getItem(ROM_STORAGE_KEYS[romType]);
            if (cached) {
                try {
                    const bytes = this.base64ToArrayBuffer(cached);
                    if (bytes.length === ROM_SIZES[romType]) {
                        this.roms[romType] = bytes;
                        const statusEl = document.getElementById(`${romType}-status`);
                        if (statusEl) {
                            statusEl.textContent = `✓ Loaded from cache (${bytes.length} bytes)`;
                            statusEl.className = 'rom-status success';
                        }
                        console.log(`${romType} ROM loaded from cache`);
                    }
                } catch (error) {
                    console.warn(`Failed to load cached ${romType} ROM:`, error);
                }
            }
        }

        this.updateStartButton();
    }

    /**
     * Check if all ROMs are loaded
     */
    hasAllRoms() {
        return this.roms.kernal && this.roms.basic && this.roms.charrom;
    }

    /**
     * Update start button enabled state
     */
    updateStartButton() {
        const startBtn = document.getElementById('start-emulator-btn');
        if (startBtn) {
            startBtn.disabled = !this.hasAllRoms();
        }
    }

    /**
     * Show the emulator section (hide ROM upload)
     */
    showEmulatorSection() {
        const romSection = document.getElementById('rom-upload-section');
        const emulatorSection = document.getElementById('emulator-section');

        if (romSection) {
            romSection.style.display = 'none';
        }
        if (emulatorSection) {
            emulatorSection.style.display = 'flex';
        }
    }

    /**
     * Start the emulator
     */
    async startEmulator() {
        if (!this.hasAllRoms()) {
            this.showError('Please load all ROM files first');
            return;
        }

        try {
            // Create emulator instance
            this.emulator = new C64Emulator();

            // Load ROMs
            if (!this.emulator.load_kernal(this.roms.kernal)) {
                throw new Error('Failed to load KERNAL ROM');
            }
            if (!this.emulator.load_basic(this.roms.basic)) {
                throw new Error('Failed to load BASIC ROM');
            }
            if (!this.emulator.load_charrom(this.roms.charrom)) {
                throw new Error('Failed to load Character ROM');
            }

            // Verify ROMs loaded
            if (!this.emulator.roms_loaded()) {
                throw new Error('ROM validation failed');
            }

            // Reset to initialize
            this.emulator.reset();

            // Show emulator section
            this.showEmulatorSection();

            // Start the emulator
            this.emulator.start();
            this.running = true;
            this.paused = false;

            // Update status
            this.updateStatus('Running');

            // Start the main loop
            this.startMainLoop();

            console.log('C64 Emulator started successfully');
        } catch (error) {
            console.error('Failed to start emulator:', error);
            this.showError(`Failed to start emulator: ${error.message}`);
        }
    }

    /**
     * Start the main emulation loop
     */
    startMainLoop() {
        const targetFps = this.region === 'pal' ? 50 : 60;
        const frameTime = 1000 / targetFps;
        let lastFrameTime = performance.now();

        const loop = (currentTime) => {
            if (!this.running) {
                return;
            }

            this.animationFrameId = requestAnimationFrame(loop);

            if (this.paused) {
                return;
            }

            // Check if enough time has passed for a frame
            const elapsed = currentTime - lastFrameTime;
            if (elapsed < frameTime) {
                return;
            }

            lastFrameTime = currentTime - (elapsed % frameTime);

            try {
                // Execute one frame
                this.emulator.step_frame();

                // Render the frame
                this.renderFrame();

                // Update FPS counter
                this.updateFps(currentTime);
            } catch (error) {
                console.error('Emulation error:', error);
                this.showError(`Emulation error: ${error.message}`);
                this.stop();
            }
        };

        this.animationFrameId = requestAnimationFrame(loop);
    }

    /**
     * Render the current frame to the canvas
     */
    renderFrame() {
        if (!this.emulator || !this.ctx) {
            return;
        }

        // Get framebuffer from emulator
        const framebuffer = this.emulator.get_framebuffer();

        // Convert indexed colors to RGBA
        for (let i = 0; i < 320 * 200; i++) {
            const colorIndex = framebuffer[i] & 0x0F;
            const rgb = C64_PALETTE[colorIndex];
            this.imageData.data[i * 4 + 0] = rgb[0];
            this.imageData.data[i * 4 + 1] = rgb[1];
            this.imageData.data[i * 4 + 2] = rgb[2];
            this.imageData.data[i * 4 + 3] = 255;
        }

        // Draw to canvas
        this.ctx.putImageData(this.imageData, 0, 0);

        // Update border color
        this.updateBorderColor();
    }

    /**
     * Update the CSS border color based on VIC-II border register
     */
    updateBorderColor() {
        if (!this.emulator) return;

        const borderColorIndex = this.emulator.get_border_color();
        const rgb = C64_PALETTE[borderColorIndex];
        const borderEl = document.getElementById('c64-border');
        if (borderEl) {
            borderEl.style.backgroundColor = `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
        }
    }

    /**
     * Update FPS counter
     */
    updateFps(currentTime) {
        this.frameCount++;

        if (currentTime - this.lastFpsUpdate >= 1000) {
            this.fps = this.frameCount;
            this.frameCount = 0;
            this.lastFpsUpdate = currentTime;

            const fpsEl = document.getElementById('fps-counter');
            if (fpsEl) {
                fpsEl.textContent = `FPS: ${this.fps}`;
            }
        }
    }

    /**
     * Handle keyboard key down
     */
    handleKeyDown(event) {
        if (!this.emulator || !this.running) {
            return;
        }

        // Don't capture input when typing in form elements
        if (event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA') {
            return;
        }

        // Handle special keys
        if (event.code === 'PageUp') {
            // RESTORE key (NMI)
            this.emulator.restore_key();
            event.preventDefault();
            return;
        }

        // Send to emulator via PC keycode mapping
        this.emulator.key_down_pc(event.code);

        // Prevent default for most keys to avoid browser shortcuts
        if (!event.ctrlKey && !event.metaKey && !event.altKey) {
            event.preventDefault();
        }
    }

    /**
     * Handle keyboard key up
     */
    handleKeyUp(event) {
        if (!this.emulator || !this.running) {
            return;
        }

        // Don't capture input when typing in form elements
        if (event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA') {
            return;
        }

        this.emulator.key_up_pc(event.code);
    }

    /**
     * Reset the emulator (warm reset)
     */
    reset() {
        if (this.emulator) {
            this.emulator.reset();
            console.log('C64 reset');
        }
    }

    /**
     * Hard reset (power cycle)
     */
    hardReset() {
        if (this.emulator) {
            // For now, recreate the emulator
            this.stop();
            this.startEmulator();
            console.log('C64 hard reset');
        }
    }

    /**
     * Toggle pause state
     */
    togglePause() {
        this.paused = !this.paused;

        const pauseBtn = document.getElementById('pause-btn');
        if (pauseBtn) {
            pauseBtn.textContent = this.paused ? 'Resume' : 'Pause';
        }

        if (this.emulator) {
            if (this.paused) {
                this.emulator.stop();
                this.updateStatus('Paused');
            } else {
                this.emulator.start();
                this.updateStatus('Running');
            }
        }

        console.log(this.paused ? 'Emulation paused' : 'Emulation resumed');
    }

    /**
     * Toggle audio mute
     */
    toggleMute() {
        this.audioEnabled = !this.audioEnabled;

        const muteBtn = document.getElementById('mute-btn');
        if (muteBtn) {
            muteBtn.textContent = this.audioEnabled ? 'Mute' : 'Unmute';
        }

        // Audio will be handled by the audio component
        console.log(this.audioEnabled ? 'Audio enabled' : 'Audio muted');
    }

    /**
     * Set video region (PAL/NTSC)
     */
    setRegion(region) {
        this.region = region;
        // Region change would require recreating emulator with new settings
        // For now, just store the preference
        console.log(`Region set to: ${region.toUpperCase()}`);
    }

    /**
     * Set display scale
     */
    setScale(scale) {
        this.scale = scale;
        const displayEl = document.getElementById('c64-display');
        if (displayEl) {
            // Remove existing scale classes
            displayEl.classList.remove('scale-1x', 'scale-2x', 'scale-3x', 'scale-fit');
            // Add new scale class
            displayEl.classList.add(`scale-${scale}x`);
        }
        console.log(`Display scale set to: ${scale}`);
    }

    /**
     * Stop the emulator
     */
    stop() {
        this.running = false;
        if (this.animationFrameId) {
            cancelAnimationFrame(this.animationFrameId);
            this.animationFrameId = null;
        }
        if (this.emulator) {
            this.emulator.stop();
        }
        this.updateStatus('Stopped');
        console.log('Emulator stopped');
    }

    /**
     * Update status display
     */
    updateStatus(status) {
        const statusEl = document.getElementById('emulator-status');
        if (statusEl) {
            statusEl.textContent = status;
            statusEl.className = '';
            if (status === 'Paused') {
                statusEl.classList.add('paused');
            } else if (status === 'Error') {
                statusEl.classList.add('error');
            }
        }
    }

    /**
     * Show error message
     */
    showError(message) {
        console.error(message);
        this.updateStatus('Error');
        // Could add a toast or modal for errors
        alert(message);
    }

    /**
     * Convert ArrayBuffer to Base64 string
     */
    arrayBufferToBase64(buffer) {
        let binary = '';
        const bytes = new Uint8Array(buffer);
        for (let i = 0; i < bytes.length; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        return btoa(binary);
    }

    /**
     * Convert Base64 string to Uint8Array
     */
    base64ToArrayBuffer(base64) {
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
        }
        return bytes;
    }
}

// Initialize the application when the page loads
const app = new C64App();
document.addEventListener('DOMContentLoaded', () => {
    app.init();
});

// Export for debugging and potential use by components
window.c64App = app;
export { C64App, C64_PALETTE, ROM_SIZES };
