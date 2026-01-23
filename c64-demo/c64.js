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

// LocalStorage keys for save state slots
const SAVE_SLOT_PREFIX = 'c64_savestate_slot_';
const SAVE_SLOT_COUNT = 4;

// LocalStorage key for settings
const SETTINGS_STORAGE_KEY = 'c64_settings';

// Default settings
const DEFAULT_SETTINGS = {
    scale: '2',
    scanlines: false,
    volume: 50,
    region: 'pal',
    joystickMappings: {
        port1: {
            up: 'KeyW',
            down: 'KeyS',
            left: 'KeyA',
            right: 'KeyD',
            fire: 'ControlLeft'
        },
        port2: {
            up: 'ArrowUp',
            down: 'ArrowDown',
            left: 'ArrowLeft',
            right: 'ArrowRight',
            fire: 'Space'
        }
    }
};

/**
 * Main C64 Emulator Application
 */
// Joystick bit constants (must match WASM exports)
const JOY_UP = 0x01;
const JOY_DOWN = 0x02;
const JOY_LEFT = 0x04;
const JOY_RIGHT = 0x08;
const JOY_FIRE = 0x10;

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

        // Audio state
        this.audioEnabled = false;
        this.audioInitialized = false;
        this.volume = 0.5;
        this.audioContext = null;
        this.audioWorkletNode = null;
        this.audioSampleRate = 44100;

        // Joystick state (T092-T098)
        this.joystickState = {
            port1: 0, // Physical port 1 state (bitmask)
            port2: 0, // Physical port 2 state (bitmask)
        };
        this.joystickSwapped = false;
        this.gamepadConnected = false;
        this.gamepadIndex = null;
        this.gamepadPollingId = null;

        // Settings (T112-T118)
        this.settings = { ...DEFAULT_SETTINGS };
        this.scanlines = false;
        this.settingsPanelOpen = false;
        this.listeningForKey = null; // For joystick remapping

        // Tab visibility auto-pause (T121)
        this.autoPaused = false; // True if we auto-paused due to tab hidden
    }

    /**
     * Initialize the application
     */
    async init() {
        console.log('C64 Emulator initializing...');

        try {
            // Load saved settings first
            this.loadSettings();

            // Load WASM module
            await this.loadWasm();

            // Initialize display
            this.initDisplay();

            // Set up UI event handlers
            this.setupEventHandlers();

            // Initialize joystick/gamepad support
            this.initJoystick();

            // Apply loaded settings to UI
            this.applySettingsToUI();

            // Check for cached ROMs
            await this.loadCachedRoms();

            // If all ROMs are loaded, auto-start the emulator
            if (this.hasAllRoms()) {
                await this.startEmulator();
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
                this.setVolume(e.target.value / 100);
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

        // Scanlines checkbox
        const scanlinesCheckbox = document.getElementById('scanlines-checkbox');
        if (scanlinesCheckbox) {
            scanlinesCheckbox.addEventListener('change', (e) => {
                this.setScanlines(e.target.checked);
            });
        }

        // Joystick swap button
        const joystickSwapBtn = document.getElementById('joystick-swap-btn');
        if (joystickSwapBtn) {
            joystickSwapBtn.addEventListener('click', () => this.toggleJoystickSwap());
        }

        // Keyboard events
        document.addEventListener('keydown', (e) => this.handleKeyDown(e));
        document.addEventListener('keyup', (e) => this.handleKeyUp(e));

        // Tab visibility for auto-pause (T121)
        document.addEventListener('visibilitychange', () => {
            this.handleVisibilityChange();
        });

        // File drag and drop
        const dropZone = document.getElementById('drop-zone');
        if (dropZone) {
            dropZone.addEventListener('dragover', (e) => {
                e.preventDefault();
                e.stopPropagation();
                dropZone.classList.add('drag-over');
            });

            dropZone.addEventListener('dragleave', (e) => {
                e.preventDefault();
                e.stopPropagation();
                dropZone.classList.remove('drag-over');
            });

            dropZone.addEventListener('drop', (e) => {
                e.preventDefault();
                e.stopPropagation();
                dropZone.classList.remove('drag-over');
                this.handleFileDrop(e.dataTransfer.files);
            });
        }

        const fileInput = document.getElementById('file-input');
        if (fileInput) {
            fileInput.addEventListener('change', (e) => {
                this.handleFileDrop(e.target.files);
                // Reset input so the same file can be loaded again
                e.target.value = '';
            });
        }

        const fileBrowseBtn = document.getElementById('file-browse-btn');
        if (fileBrowseBtn) {
            fileBrowseBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                document.getElementById('file-input').click();
            });
        }

        // Disk write controls (T128-T129)
        const saveDiskBtn = document.getElementById('save-disk-btn');
        if (saveDiskBtn) {
            saveDiskBtn.addEventListener('click', () => this.saveDiskToFile());
        }

        const unmountDiskBtn = document.getElementById('unmount-disk-btn');
        if (unmountDiskBtn) {
            unmountDiskBtn.addEventListener('click', () => this.unmountDiskWithConfirm());
        }

        // Save state controls (T109-T111)
        this.setupSaveStateHandlers();

        // Settings panel controls (T112-T118)
        this.setupSettingsHandlers();
    }

    /**
     * Set up save state event handlers (T109-T111)
     */
    setupSaveStateHandlers() {
        // Save state to file (T109)
        const saveStateBtn = document.getElementById('save-state-btn');
        if (saveStateBtn) {
            saveStateBtn.addEventListener('click', () => this.saveStateToFile());
        }

        // Load state from file (T110)
        const loadStateBtn = document.getElementById('load-state-btn');
        if (loadStateBtn) {
            loadStateBtn.addEventListener('click', () => {
                document.getElementById('load-state-file').click();
            });
        }

        const loadStateFile = document.getElementById('load-state-file');
        if (loadStateFile) {
            loadStateFile.addEventListener('change', (e) => {
                this.loadStateFromFile(e.target.files[0]);
                e.target.value = ''; // Reset so same file can be loaded again
            });
        }

        // Quick save slots (T111)
        document.querySelectorAll('.slot-save-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const slot = parseInt(e.target.dataset.slot, 10);
                this.saveToSlot(slot);
            });
        });

        document.querySelectorAll('.slot-load-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const slot = parseInt(e.target.dataset.slot, 10);
                this.loadFromSlot(slot);
            });
        });

        // Clear all slots
        const clearSlotsBtn = document.getElementById('clear-slots-btn');
        if (clearSlotsBtn) {
            clearSlotsBtn.addEventListener('click', () => this.clearAllSlots());
        }

        // Initialize slot UI state
        this.updateSaveSlotUI();
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

            // Validate size with specific error messages
            if (bytes.length !== expectedSize) {
                const romNames = {
                    kernal: 'KERNAL ROM',
                    basic: 'BASIC ROM',
                    charrom: 'Character ROM'
                };
                const romName = romNames[romType] || romType.toUpperCase();

                let errorMsg = `Invalid ${romName} size: expected ${expectedSize} bytes, got ${bytes.length} bytes.`;

                // Provide helpful hints based on size
                if (bytes.length === 0) {
                    errorMsg = `The file appears to be empty. Please select a valid ${romName} file.`;
                } else if (bytes.length > expectedSize * 2) {
                    errorMsg += '\n\nThe file is much larger than expected. This might be a combined ROM image or wrong file type.';
                } else if (bytes.length < expectedSize / 2) {
                    errorMsg += '\n\nThe file is much smaller than expected. It may be incomplete or corrupted.';
                } else if (romType === 'kernal' && bytes.length === 16384) {
                    errorMsg += '\n\nThis might be a combined BASIC+KERNAL ROM. Please use separate ROM files.';
                } else if (romType === 'charrom' && bytes.length === 8192) {
                    errorMsg += '\n\nThis appears to be a BASIC or KERNAL ROM (8KB). Character ROM should be 4KB.';
                }

                throw new Error(errorMsg);
            }

            // Basic content validation (check if ROM contains actual code, not all zeros/FFs)
            const validationError = this.validateRomContent(bytes, romType);
            if (validationError) {
                throw new Error(validationError);
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
     * Validate ROM content for obvious corruption
     * @param {Uint8Array} bytes - ROM data
     * @param {string} romType - Type of ROM ('kernal', 'basic', 'charrom')
     * @returns {string|null} Error message if invalid, null if OK
     */
    validateRomContent(bytes, romType) {
        // Check if ROM is all zeros
        let allZeros = true;
        let allOnes = true;
        let zeroCount = 0;
        let ffCount = 0;

        for (let i = 0; i < bytes.length; i++) {
            if (bytes[i] !== 0x00) allZeros = false;
            if (bytes[i] !== 0xFF) allOnes = false;
            if (bytes[i] === 0x00) zeroCount++;
            if (bytes[i] === 0xFF) ffCount++;
        }

        if (allZeros) {
            return 'ROM file contains only zeros. This is likely a blank or corrupted file.';
        }
        if (allOnes) {
            return 'ROM file contains only 0xFF bytes. This is likely a blank EPROM dump or corrupted file.';
        }

        // Check for suspiciously uniform content (more than 95% same value)
        const threshold = bytes.length * 0.95;
        if (zeroCount > threshold) {
            return `ROM file is mostly zeros (${Math.round(zeroCount / bytes.length * 100)}%). This file may be corrupted or incomplete.`;
        }
        if (ffCount > threshold) {
            return `ROM file is mostly 0xFF bytes (${Math.round(ffCount / bytes.length * 100)}%). This file may be corrupted or incomplete.`;
        }

        // ROM-specific validation
        if (romType === 'kernal') {
            // KERNAL ROM should have reset vector at $FFFC-$FFFD (end of ROM)
            // Offset in 8KB KERNAL: 0x1FFC, 0x1FFD
            const resetLow = bytes[0x1FFC];
            const resetHigh = bytes[0x1FFD];
            const resetVector = (resetHigh << 8) | resetLow;

            // Reset vector should point to $E000-$FFFF range (KERNAL space)
            if (resetVector < 0xE000 || resetVector > 0xFFFF) {
                return `KERNAL ROM has invalid reset vector ($${resetVector.toString(16).toUpperCase()}). Expected $E000-$FFFF. This may not be a valid C64 KERNAL ROM.`;
            }
        }

        if (romType === 'basic') {
            // BASIC ROM typically starts with cold start entry point
            // First two bytes are usually a JMP instruction or pointer
            // Cold start is at $A000 (offset 0)
            // Common first bytes in BASIC ROM: 0x94 (for STA), 0x4C (JMP), etc.
            // Validation is loose here since there are various BASIC versions
        }

        if (romType === 'charrom') {
            // Character ROM should have actual bitmap data
            // The first 8 bytes represent the '@' character in the standard charset
            // A completely blank charrom would make display unusable
        }

        return null; // Validation passed
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

            // Initialize disk status
            this.updateDiskStatus();

            // Initialize audio (user gesture from clicking start button)
            await this.initAudio();

            // Start the main loop
            this.startMainLoop();

            console.log('C64 Emulator started successfully');
        } catch (error) {
            console.error('Failed to start emulator:', error);
            this.showError(`Failed to start emulator: ${error.message}`);
        }
    }

    /**
     * Start the main emulation loop (T122)
     *
     * Uses requestAnimationFrame with frame time accounting to maintain
     * accurate 50 Hz (PAL) or 60 Hz (NTSC) emulation speed.
     */
    startMainLoop() {
        const targetFps = this.region === 'pal' ? 50 : 60;
        const frameTime = 1000 / targetFps;
        let lastFrameTime = performance.now();

        // Maximum frames to catch up if we fall behind
        // This prevents runaway catchup after browser throttling
        const maxFrameSkip = 3;

        const loop = (currentTime) => {
            if (!this.running) {
                return;
            }

            this.animationFrameId = requestAnimationFrame(loop);

            if (this.paused) {
                // Reset timing when paused to prevent catchup on resume
                lastFrameTime = currentTime;
                return;
            }

            // Check if enough time has passed for a frame
            const elapsed = currentTime - lastFrameTime;
            if (elapsed < frameTime) {
                return;
            }

            // Calculate how many frames we need to run (with limit)
            // This handles cases where the browser throttled us
            let framesToRun = Math.floor(elapsed / frameTime);
            if (framesToRun > maxFrameSkip) {
                // Too far behind - skip frames instead of running extra
                // This prevents audio desync and maintains responsiveness
                framesToRun = 1;
                lastFrameTime = currentTime;
            } else {
                lastFrameTime += framesToRun * frameTime;
            }

            try {
                // Execute frames (typically just 1)
                for (let i = 0; i < framesToRun; i++) {
                    this.emulator.step_frame();

                    // Process audio each frame to prevent buffer underruns
                    this.processAudio();
                }

                // Render only the final frame (no point rendering skipped frames)
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

    // =========================================================================
    // Audio System (T089-T091)
    // =========================================================================

    /**
     * Initialize the audio system.
     * Must be called from a user gesture (click/keypress) due to browser autoplay policies.
     */
    async initAudio() {
        if (this.audioInitialized) {
            return true;
        }

        try {
            // Create AudioContext
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
            this.audioSampleRate = this.audioContext.sampleRate;

            console.log(`Audio context created at ${this.audioSampleRate} Hz`);

            // Load the AudioWorklet module
            await this.audioContext.audioWorklet.addModule('components/sid-audio-processor.js');

            // Create the worklet node
            this.audioWorkletNode = new AudioWorkletNode(this.audioContext, 'sid-audio-processor');

            // Connect to audio output
            this.audioWorkletNode.connect(this.audioContext.destination);

            // Set initial volume
            this.audioWorkletNode.port.postMessage({
                type: 'volume',
                value: this.volume
            });

            // Configure emulator sample rate to match audio context
            if (this.emulator) {
                this.emulator.set_sample_rate(this.audioSampleRate);
                this.emulator.set_audio_enabled(true);
            }

            this.audioInitialized = true;
            this.audioEnabled = true;

            // Update mute button text
            const muteBtn = document.getElementById('mute-btn');
            if (muteBtn) {
                muteBtn.textContent = 'Mute';
            }

            console.log('Audio system initialized successfully');
            return true;
        } catch (error) {
            console.error('Failed to initialize audio:', error);
            this.audioInitialized = false;
            return false;
        }
    }

    /**
     * Resume audio context if suspended.
     * Called on user interaction to handle autoplay policy.
     */
    async resumeAudio() {
        if (this.audioContext && this.audioContext.state === 'suspended') {
            try {
                await this.audioContext.resume();
                console.log('Audio context resumed');
            } catch (error) {
                console.warn('Failed to resume audio context:', error);
            }
        }
    }

    /**
     * Process audio samples from the emulator.
     * Should be called each frame to transfer samples to the audio worklet.
     */
    processAudio() {
        if (!this.audioInitialized || !this.audioEnabled || !this.emulator) {
            return;
        }

        // Get samples from emulator
        const samples = this.emulator.get_audio_samples();

        if (samples && samples.length > 0) {
            // Send samples to audio worklet
            this.audioWorkletNode.port.postMessage({
                type: 'samples',
                samples: samples
            });
        }
    }

    /**
     * Set audio mute state
     */
    setMute(muted) {
        if (this.audioWorkletNode) {
            this.audioWorkletNode.port.postMessage({
                type: 'mute',
                value: muted
            });
        }

        // Also tell emulator to stop generating samples if muted (saves CPU)
        if (this.emulator) {
            this.emulator.set_audio_enabled(!muted);
        }
    }

    /**
     * Clean up audio resources
     */
    cleanupAudio() {
        if (this.audioWorkletNode) {
            this.audioWorkletNode.disconnect();
            this.audioWorkletNode = null;
        }

        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }

        this.audioInitialized = false;
        console.log('Audio system cleaned up');
    }

    // =========================================================================
    // Joystick System (T092-T098)
    // =========================================================================

    /**
     * Initialize joystick/gamepad support.
     * Sets up gamepad event listeners and starts polling if needed.
     */
    initJoystick() {
        // Set up gamepad event listeners
        window.addEventListener('gamepadconnected', (e) => this.handleGamepadConnected(e));
        window.addEventListener('gamepaddisconnected', (e) => this.handleGamepadDisconnected(e));

        // Check for already-connected gamepads (some browsers don't fire events)
        this.checkForGamepads();

        console.log('Joystick system initialized');
    }

    /**
     * Handle joystick key press.
     * Returns true if the key was handled as a joystick input.
     */
    handleJoystickKeyDown(code) {
        const mapping = this.mapKeyToJoystick(code);
        if (!mapping) {
            return false;
        }

        // Update joystick state
        if (mapping.port === 1) {
            this.joystickState.port1 |= mapping.bit;
        } else {
            this.joystickState.port2 |= mapping.bit;
        }

        // Send to emulator
        this.updateJoystickInEmulator();
        return true;
    }

    /**
     * Handle joystick key release.
     * Returns true if the key was handled as a joystick input.
     */
    handleJoystickKeyUp(code) {
        const mapping = this.mapKeyToJoystick(code);
        if (!mapping) {
            return false;
        }

        // Update joystick state
        if (mapping.port === 1) {
            this.joystickState.port1 &= ~mapping.bit;
        } else {
            this.joystickState.port2 &= ~mapping.bit;
        }

        // Send to emulator
        this.updateJoystickInEmulator();
        return true;
    }

    /**
     * Send current joystick state to the emulator.
     */
    updateJoystickInEmulator() {
        if (!this.emulator) {
            return;
        }

        // Use the unified set_joystick API which respects port swap
        this.emulator.set_joystick(1, this.joystickState.port1);
        this.emulator.set_joystick(2, this.joystickState.port2);
    }

    /**
     * Handle gamepad connected event.
     */
    handleGamepadConnected(event) {
        console.log(`Gamepad connected: ${event.gamepad.id} (index ${event.gamepad.index})`);
        this.gamepadConnected = true;
        this.gamepadIndex = event.gamepad.index;

        // Start polling for gamepad input
        if (!this.gamepadPollingId) {
            this.startGamepadPolling();
        }

        // Update UI
        this.updateJoystickUI();
    }

    /**
     * Handle gamepad disconnected event.
     */
    handleGamepadDisconnected(event) {
        console.log(`Gamepad disconnected: ${event.gamepad.id}`);

        if (this.gamepadIndex === event.gamepad.index) {
            this.gamepadConnected = false;
            this.gamepadIndex = null;

            // Check if any other gamepads are still connected
            this.checkForGamepads();
        }

        // Update UI
        this.updateJoystickUI();
    }

    /**
     * Check for already-connected gamepads.
     */
    checkForGamepads() {
        const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
        for (let i = 0; i < gamepads.length; i++) {
            if (gamepads[i]) {
                this.gamepadConnected = true;
                this.gamepadIndex = i;
                console.log(`Found connected gamepad: ${gamepads[i].id}`);

                if (!this.gamepadPollingId) {
                    this.startGamepadPolling();
                }
                break;
            }
        }
    }

    /**
     * Start polling for gamepad input.
     * Gamepads must be polled because they don't generate events for button presses.
     */
    startGamepadPolling() {
        const poll = () => {
            if (!this.gamepadConnected || !this.running) {
                this.gamepadPollingId = null;
                return;
            }

            this.pollGamepad();
            this.gamepadPollingId = requestAnimationFrame(poll);
        };

        this.gamepadPollingId = requestAnimationFrame(poll);
    }

    /**
     * Poll gamepad state and update joystick.
     */
    pollGamepad() {
        if (this.gamepadIndex === null) {
            return;
        }

        const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
        const gamepad = gamepads[this.gamepadIndex];

        if (!gamepad) {
            return;
        }

        // Standard gamepad mapping:
        // Buttons: 0=A/X, 1=B/O, 2=X/Square, 3=Y/Triangle, etc.
        // Axes: 0=Left X, 1=Left Y, 2=Right X, 3=Right Y

        let state = 0;
        const deadzone = 0.3;

        // D-pad or left stick for directions
        // D-pad buttons (12=Up, 13=Down, 14=Left, 15=Right)
        if (gamepad.buttons[12]?.pressed) state |= JOY_UP;
        if (gamepad.buttons[13]?.pressed) state |= JOY_DOWN;
        if (gamepad.buttons[14]?.pressed) state |= JOY_LEFT;
        if (gamepad.buttons[15]?.pressed) state |= JOY_RIGHT;

        // Left analog stick
        if (gamepad.axes[0] < -deadzone) state |= JOY_LEFT;
        if (gamepad.axes[0] > deadzone) state |= JOY_RIGHT;
        if (gamepad.axes[1] < -deadzone) state |= JOY_UP;
        if (gamepad.axes[1] > deadzone) state |= JOY_DOWN;

        // Fire buttons (A, B, X, Y, L1, R1 all work as fire)
        if (gamepad.buttons[0]?.pressed ||  // A/X
            gamepad.buttons[1]?.pressed ||  // B/O
            gamepad.buttons[2]?.pressed ||  // X/Square
            gamepad.buttons[3]?.pressed ||  // Y/Triangle
            gamepad.buttons[4]?.pressed ||  // L1
            gamepad.buttons[5]?.pressed) {  // R1
            state |= JOY_FIRE;
        }

        // Gamepad controls joystick 2 by default (most common for C64 games)
        this.joystickState.port2 = (this.joystickState.port2 & ~0x1F) | state;

        // Update emulator
        this.updateJoystickInEmulator();
    }

    /**
     * Stop gamepad polling.
     */
    stopGamepadPolling() {
        if (this.gamepadPollingId) {
            cancelAnimationFrame(this.gamepadPollingId);
            this.gamepadPollingId = null;
        }
    }

    /**
     * Toggle joystick port swap.
     * When swapped, port 2 input goes to physical port 1 and vice versa.
     */
    toggleJoystickSwap() {
        this.joystickSwapped = !this.joystickSwapped;

        if (this.emulator) {
            this.emulator.set_joystick_swap(this.joystickSwapped);
        }

        console.log(`Joystick ports ${this.joystickSwapped ? 'swapped' : 'normal'}`);
        this.updateJoystickUI();
    }

    /**
     * Update joystick-related UI elements.
     */
    updateJoystickUI() {
        // Update swap button text
        const swapBtn = document.getElementById('joystick-swap-btn');
        if (swapBtn) {
            swapBtn.textContent = this.joystickSwapped ? 'Ports: Swapped' : 'Ports: Normal';
            swapBtn.classList.toggle('active', this.joystickSwapped);
        }

        // Update gamepad indicator
        const gamepadIndicator = document.getElementById('gamepad-indicator');
        if (gamepadIndicator) {
            gamepadIndicator.style.display = this.gamepadConnected ? 'inline' : 'none';
        }
    }

    /**
     * Release all joystick buttons.
     */
    releaseAllJoysticks() {
        this.joystickState.port1 = 0;
        this.joystickState.port2 = 0;
        this.updateJoystickInEmulator();
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

        // Check if this is a joystick key first
        if (this.handleJoystickKeyDown(event.code)) {
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

        // Check if this is a joystick key first
        if (this.handleJoystickKeyUp(event.code)) {
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
    async togglePause() {
        this.paused = !this.paused;
        // Clear auto-pause flag since this is a manual toggle
        this.autoPaused = false;

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
                // Resume audio context if needed
                await this.resumeAudio();
                this.updateStatus('Running');
            }
        }

        console.log(this.paused ? 'Emulation paused' : 'Emulation resumed');
    }

    /**
     * Handle browser tab visibility change (T121)
     * Auto-pauses when tab is hidden, auto-resumes when visible again.
     */
    async handleVisibilityChange() {
        if (document.hidden) {
            // Tab lost focus - pause if running
            if (this.running && !this.paused) {
                // Release all inputs first
                if (this.emulator) {
                    this.emulator.release_all_keys();
                    this.releaseAllJoysticks();
                }

                // Auto-pause the emulation
                this.autoPaused = true;
                this.paused = true;

                if (this.emulator) {
                    this.emulator.stop();
                }

                // Suspend audio context to save resources
                if (this.audioContext && this.audioContext.state === 'running') {
                    try {
                        await this.audioContext.suspend();
                    } catch (e) {
                        console.warn('Failed to suspend audio context:', e);
                    }
                }

                this.updateStatus('Paused (background)');
                console.log('Emulation auto-paused (tab hidden)');
            }
        } else {
            // Tab regained focus - resume if we auto-paused
            if (this.autoPaused && this.paused) {
                this.autoPaused = false;
                this.paused = false;

                if (this.emulator) {
                    this.emulator.start();
                }

                // Resume audio context
                await this.resumeAudio();

                this.updateStatus('Running');

                // Update pause button text
                const pauseBtn = document.getElementById('pause-btn');
                if (pauseBtn) {
                    pauseBtn.textContent = 'Pause';
                }

                console.log('Emulation auto-resumed (tab visible)');
            }
        }
    }

    /**
     * Toggle audio mute
     */
    async toggleMute() {
        // Initialize audio on first unmute (requires user gesture)
        if (!this.audioInitialized && !this.audioEnabled) {
            await this.initAudio();
        }

        this.audioEnabled = !this.audioEnabled;

        const muteBtn = document.getElementById('mute-btn');
        if (muteBtn) {
            muteBtn.textContent = this.audioEnabled ? 'Mute' : 'Unmute';
        }

        // Update audio system
        this.setMute(!this.audioEnabled);

        console.log(this.audioEnabled ? 'Audio enabled' : 'Audio muted');
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
        // Clear audio buffer on stop
        if (this.audioWorkletNode) {
            this.audioWorkletNode.port.postMessage({ type: 'clear' });
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

    // =========================================================================
    // File Loading (T064-T066)
    // =========================================================================

    /**
     * Handle dropped or selected files
     * @param {FileList} files - The files to process
     */
    async handleFileDrop(files) {
        if (!files || files.length === 0) {
            return;
        }

        // Process only the first file
        const file = files[0];
        const extension = file.name.toLowerCase().split('.').pop();

        try {
            const data = await this.readFileAsArrayBuffer(file);
            const bytes = new Uint8Array(data);

            switch (extension) {
                case 'd64':
                    await this.loadD64File(file.name, bytes);
                    break;
                case 'prg':
                case 'p00':
                    await this.loadPRGFile(file.name, bytes);
                    break;
                default:
                    this.showError(`Unsupported file type: .${extension}\nSupported: .D64, .PRG, .P00`);
                    return;
            }
        } catch (error) {
            console.error('Failed to load file:', error);
            this.showError(`Failed to load file: ${error.message}`);
        }
    }

    /**
     * Load a D64 disk image
     * @param {string} filename - Name of the file
     * @param {Uint8Array} data - Raw file data
     */
    async loadD64File(filename, data) {
        if (!this.emulator) {
            this.showError('Please start the emulator first');
            return;
        }

        // Standard D64 sizes: 174848 (35 tracks) or 175531 (35 tracks + error info)
        // Extended D64: 196608 (40 tracks) or 197376 (40 tracks + error info)
        const validSizes = [174848, 175531, 196608, 197376];
        if (!validSizes.includes(data.length)) {
            this.showError(`Invalid D64 file size: ${data.length} bytes.\nExpected: 174848 or 175531 bytes (35 tracks)\nor 196608 or 197376 bytes (40 tracks)`);
            return;
        }

        // Show loading indicator
        this.setDiskStatus('reading', 'Mounting disk...');

        try {
            // Use the error-reporting mount function for detailed error messages
            const errorMsg = this.emulator.mount_d64_with_error(data);
            if (errorMsg) {
                // Format user-friendly error message
                let userMessage = errorMsg;
                if (errorMsg.includes('Corrupted D64')) {
                    userMessage = `The disk image appears to be corrupted:\n${errorMsg}`;
                } else if (errorMsg.includes('Invalid D64 size')) {
                    userMessage = `Invalid disk image size:\n${errorMsg}`;
                }
                throw new Error(userMessage);
            }

            // Get disk name from the mounted image
            const diskName = this.emulator.disk_name() || filename;
            this.setDiskStatus('mounted', diskName);

            console.log(`D64 mounted: ${diskName}`);
        } catch (error) {
            this.setDiskStatus('error', 'Mount failed');
            throw error;
        }
    }

    /**
     * Load a PRG file directly into memory
     * @param {string} filename - Name of the file
     * @param {Uint8Array} data - Raw file data
     */
    async loadPRGFile(filename, data) {
        if (!this.emulator) {
            this.showError('Please start the emulator first');
            return;
        }

        // Handle P00 files (PC64 format) - strip the 26-byte header
        let prgData = data;
        if (filename.toLowerCase().endsWith('.p00') && data.length > 26) {
            // P00 header is 26 bytes: "C64File" + original name + 0x00 padding
            const header = String.fromCharCode(...data.slice(0, 7));
            if (header === 'C64File') {
                prgData = data.slice(26);
                console.log(`P00 header stripped from ${filename}`);
            }
        }

        // PRG must have at least 2 bytes for load address
        if (prgData.length < 3) {
            this.showError(`Invalid PRG file: too small (${prgData.length} bytes)`);
            return;
        }

        // Show loading indicator
        this.setDiskStatus('reading', `Loading ${filename}...`);

        try {
            const loadAddress = this.emulator.load_prg(prgData);
            if (loadAddress === null || loadAddress === undefined) {
                throw new Error('Failed to load PRG file');
            }

            console.log(`PRG loaded at $${loadAddress.toString(16).toUpperCase().padStart(4, '0')}: ${filename}`);

            // Update disk status
            this.setDiskStatus('mounted', `${filename} @ $${loadAddress.toString(16).toUpperCase()}`);

            // Auto-run BASIC programs (load address $0801)
            if (loadAddress === 0x0801) {
                // Inject RUN command
                this.emulator.inject_basic_run();
                console.log('Auto-running BASIC program');
            }
        } catch (error) {
            this.setDiskStatus('error', 'Load failed');
            throw error;
        }
    }

    /**
     * Unmount the current disk
     */
    unmountDisk() {
        if (this.emulator && this.emulator.has_mounted_disk()) {
            this.emulator.unmount_d64();
            this.setDiskStatus('none', 'No disk mounted');
            console.log('Disk unmounted');
        }
    }

    // =========================================================================
    // Disk Write Operations (T128-T129)
    // =========================================================================

    /**
     * Save the current disk image to a downloadable file (T129).
     *
     * Downloads the D64 disk image with any modifications made during
     * the session. Useful for saving game progress on disks that support
     * saving, or for preserving changes made to disk-based programs.
     */
    saveDiskToFile() {
        if (!this.emulator) {
            this.showError('Emulator not running');
            return;
        }

        if (!this.emulator.has_mounted_disk()) {
            this.showError('No disk mounted');
            return;
        }

        try {
            // Get disk data from emulator
            const diskData = this.emulator.get_disk_data();
            if (!diskData) {
                this.showError('Failed to retrieve disk data');
                return;
            }

            // Generate filename from disk name or default
            const diskName = this.emulator.disk_name() || 'disk';
            const safeName = diskName.replace(/[^a-z0-9_-]/gi, '_').toLowerCase();
            const filename = `${safeName}.d64`;

            // Create blob and download
            const blob = new Blob([diskData], { type: 'application/octet-stream' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);

            // Clear the modified flag after successful save
            this.emulator.clear_disk_modified();

            console.log(`Disk saved as ${filename}`);
            this.updateStatus(`Saved: ${filename}`);
        } catch (error) {
            console.error('Failed to save disk:', error);
            this.showError('Failed to save disk: ' + error.message);
        }
    }

    /**
     * Unmount disk with confirmation if modified (T129).
     *
     * Prompts the user to save changes before unmounting if the disk
     * has been modified during the session.
     */
    unmountDiskWithConfirm() {
        if (!this.emulator || !this.emulator.has_mounted_disk()) {
            return;
        }

        // Check if disk has unsaved changes
        if (this.emulator.is_disk_modified()) {
            const diskName = this.emulator.disk_name() || 'the disk';
            const shouldSave = confirm(
                `${diskName} has unsaved changes.\n\n` +
                'Do you want to save the disk before unmounting?\n\n' +
                '(Click OK to save first, or Cancel to unmount without saving)'
            );

            if (shouldSave) {
                this.saveDiskToFile();
            }
        }

        this.unmountDisk();
    }

    /**
     * Update disk status indicator
     * @param {string} status - 'none', 'mounted', 'reading', 'error'
     * @param {string} text - Status text to display
     */
    setDiskStatus(status, text) {
        const indicator = document.getElementById('disk-indicator');
        const nameEl = document.getElementById('disk-name');
        const saveDiskBtn = document.getElementById('save-disk-btn');
        const unmountDiskBtn = document.getElementById('unmount-disk-btn');

        if (indicator) {
            // Remove all status classes
            indicator.classList.remove('mounted', 'reading', 'error');

            // Add the appropriate class
            if (status === 'mounted') {
                indicator.classList.add('mounted');
            } else if (status === 'reading') {
                indicator.classList.add('reading');
            }
        }

        if (nameEl) {
            nameEl.textContent = text || 'No disk mounted';
        }

        // Show/hide disk operation buttons based on mount status
        const diskMounted = status === 'mounted';
        if (saveDiskBtn) {
            saveDiskBtn.style.display = diskMounted ? 'inline-block' : 'none';
        }
        if (unmountDiskBtn) {
            unmountDiskBtn.style.display = diskMounted ? 'inline-block' : 'none';
        }
    }

    /**
     * Check current disk status and update UI
     * Called periodically or after disk operations
     */
    updateDiskStatus() {
        if (!this.emulator) {
            this.setDiskStatus('none', 'No disk mounted');
            return;
        }

        if (this.emulator.has_mounted_disk()) {
            const diskName = this.emulator.disk_name() || 'Disk mounted';
            this.setDiskStatus('mounted', diskName);
        } else {
            this.setDiskStatus('none', 'No disk mounted');
        }
    }

    // =========================================================================
    // Save State System (T109-T111)
    // =========================================================================

    /**
     * Save emulator state to a downloadable file (T109)
     */
    saveStateToFile() {
        if (!this.emulator) {
            this.showError('Emulator not running');
            return;
        }

        try {
            // Get state from emulator
            const stateData = this.emulator.save_state();

            // Create blob and download
            const blob = new Blob([stateData], { type: 'application/octet-stream' });
            const url = URL.createObjectURL(blob);

            // Generate filename with timestamp
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
            const filename = `c64-state-${timestamp}.c64state`;

            // Create download link
            const link = document.createElement('a');
            link.href = url;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);

            // Clean up URL
            URL.revokeObjectURL(url);

            console.log(`Save state downloaded: ${filename} (${stateData.length} bytes)`);
        } catch (error) {
            console.error('Failed to save state:', error);
            this.showError(`Failed to save state: ${error.message}`);
        }
    }

    /**
     * Load emulator state from a file (T110)
     * @param {File} file - The state file to load
     */
    async loadStateFromFile(file) {
        if (!file) {
            return;
        }

        if (!this.emulator) {
            this.showError('Please start the emulator first');
            return;
        }

        try {
            const data = await this.readFileAsArrayBuffer(file);
            const stateData = new Uint8Array(data);

            const success = this.emulator.load_state(stateData);
            if (!success) {
                throw new Error('Invalid or incompatible save state file');
            }

            console.log(`Save state loaded: ${file.name} (${stateData.length} bytes)`);

            // Resume emulation if paused
            if (this.paused) {
                this.togglePause();
            }
        } catch (error) {
            console.error('Failed to load state:', error);
            this.showError(`Failed to load state: ${error.message}`);
        }
    }

    /**
     * Save emulator state to a localStorage slot (T111)
     * @param {number} slot - Slot number (1-4)
     */
    saveToSlot(slot) {
        if (!this.emulator) {
            this.showError('Emulator not running');
            return;
        }

        if (slot < 1 || slot > SAVE_SLOT_COUNT) {
            this.showError(`Invalid slot number: ${slot}`);
            return;
        }

        try {
            // Get state from emulator
            const stateData = this.emulator.save_state();

            // Convert to base64 for localStorage
            const base64 = this.arrayBufferToBase64(stateData);

            // Save with metadata
            const saveData = {
                timestamp: Date.now(),
                version: this.emulator.get_state_version(),
                size: stateData.length,
                diskName: this.emulator.disk_name() || null,
                state: base64
            };

            localStorage.setItem(`${SAVE_SLOT_PREFIX}${slot}`, JSON.stringify(saveData));

            console.log(`State saved to slot ${slot} (${stateData.length} bytes)`);

            // Update UI
            this.updateSaveSlotUI();
        } catch (error) {
            console.error(`Failed to save to slot ${slot}:`, error);
            this.showError(`Failed to save to slot ${slot}: ${error.message}`);
        }
    }

    /**
     * Load emulator state from a localStorage slot (T111)
     * @param {number} slot - Slot number (1-4)
     */
    loadFromSlot(slot) {
        if (!this.emulator) {
            this.showError('Please start the emulator first');
            return;
        }

        if (slot < 1 || slot > SAVE_SLOT_COUNT) {
            this.showError(`Invalid slot number: ${slot}`);
            return;
        }

        try {
            const savedJson = localStorage.getItem(`${SAVE_SLOT_PREFIX}${slot}`);
            if (!savedJson) {
                this.showError(`No save state in slot ${slot}`);
                return;
            }

            const saveData = JSON.parse(savedJson);
            const stateData = this.base64ToArrayBuffer(saveData.state);

            const success = this.emulator.load_state(stateData);
            if (!success) {
                throw new Error('Invalid or incompatible save state');
            }

            console.log(`State loaded from slot ${slot} (${stateData.length} bytes)`);

            // Resume emulation if paused
            if (this.paused) {
                this.togglePause();
            }
        } catch (error) {
            console.error(`Failed to load from slot ${slot}:`, error);
            this.showError(`Failed to load from slot ${slot}: ${error.message}`);
        }
    }

    /**
     * Check if a save slot has data
     * @param {number} slot - Slot number (1-4)
     * @returns {object|null} Save metadata or null if empty
     */
    getSlotInfo(slot) {
        try {
            const savedJson = localStorage.getItem(`${SAVE_SLOT_PREFIX}${slot}`);
            if (!savedJson) {
                return null;
            }

            const saveData = JSON.parse(savedJson);
            return {
                timestamp: saveData.timestamp,
                size: saveData.size,
                diskName: saveData.diskName,
                version: saveData.version
            };
        } catch {
            return null;
        }
    }

    /**
     * Clear a specific save slot
     * @param {number} slot - Slot number (1-4)
     */
    clearSlot(slot) {
        localStorage.removeItem(`${SAVE_SLOT_PREFIX}${slot}`);
        this.updateSaveSlotUI();
        console.log(`Slot ${slot} cleared`);
    }

    /**
     * Clear all save slots (T111)
     */
    clearAllSlots() {
        if (!confirm('Clear all save slots? This cannot be undone.')) {
            return;
        }

        for (let slot = 1; slot <= SAVE_SLOT_COUNT; slot++) {
            localStorage.removeItem(`${SAVE_SLOT_PREFIX}${slot}`);
        }

        this.updateSaveSlotUI();
        console.log('All save slots cleared');
    }

    /**
     * Update save slot UI to reflect current state (T111)
     */
    updateSaveSlotUI() {
        for (let slot = 1; slot <= SAVE_SLOT_COUNT; slot++) {
            const slotInfo = this.getSlotInfo(slot);
            const slotEl = document.querySelector(`.save-slot[data-slot="${slot}"]`);
            const loadBtn = document.querySelector(`.slot-load-btn[data-slot="${slot}"]`);
            const saveBtn = document.querySelector(`.slot-save-btn[data-slot="${slot}"]`);

            if (slotEl) {
                if (slotInfo) {
                    slotEl.classList.add('has-data');

                    // Format timestamp for tooltip
                    const date = new Date(slotInfo.timestamp);
                    const timeStr = date.toLocaleString();
                    const diskStr = slotInfo.diskName ? ` | ${slotInfo.diskName}` : '';

                    if (saveBtn) {
                        saveBtn.title = `Save to Slot ${slot}\nLast saved: ${timeStr}${diskStr}`;
                    }
                    if (loadBtn) {
                        loadBtn.title = `Load from Slot ${slot}\nSaved: ${timeStr}${diskStr}`;
                        loadBtn.disabled = false;
                        loadBtn.classList.add('has-save');
                    }
                } else {
                    slotEl.classList.remove('has-data');

                    if (saveBtn) {
                        saveBtn.title = `Save to Slot ${slot}`;
                    }
                    if (loadBtn) {
                        loadBtn.title = `Load from Slot ${slot} (empty)`;
                        loadBtn.disabled = true;
                        loadBtn.classList.remove('has-save');
                    }
                }
            }
        }
    }

    // =========================================================================
    // Settings System (T112-T118)
    // =========================================================================

    /**
     * Set up settings panel event handlers (T112-T118)
     */
    setupSettingsHandlers() {
        // Settings button opens panel
        const settingsBtn = document.getElementById('settings-btn');
        if (settingsBtn) {
            settingsBtn.addEventListener('click', () => this.openSettingsPanel());
        }

        // Close button and overlay close panel
        const closeBtn = document.getElementById('settings-close-btn');
        if (closeBtn) {
            closeBtn.addEventListener('click', () => this.closeSettingsPanel());
        }

        const overlay = document.getElementById('settings-overlay');
        if (overlay) {
            overlay.addEventListener('click', () => this.closeSettingsPanel());
        }

        // Settings panel scale select (mirrors main control)
        const settingsScale = document.getElementById('settings-scale');
        if (settingsScale) {
            settingsScale.addEventListener('change', (e) => {
                this.setScale(e.target.value);
                // Sync main control bar select
                const mainSelect = document.getElementById('scale-select');
                if (mainSelect) mainSelect.value = e.target.value;
            });
        }

        // Settings panel scanlines checkbox (mirrors main control)
        const settingsScanlines = document.getElementById('settings-scanlines');
        if (settingsScanlines) {
            settingsScanlines.addEventListener('change', (e) => {
                this.setScanlines(e.target.checked);
                // Sync main control bar checkbox
                const mainCheckbox = document.getElementById('scanlines-checkbox');
                if (mainCheckbox) mainCheckbox.checked = e.target.checked;
            });
        }

        // Settings panel volume slider (mirrors main control)
        const settingsVolume = document.getElementById('settings-volume');
        const volumeValue = document.getElementById('settings-volume-value');
        if (settingsVolume) {
            settingsVolume.addEventListener('input', (e) => {
                this.setVolume(e.target.value / 100);
                if (volumeValue) volumeValue.textContent = `${e.target.value}%`;
                // Sync main control bar slider
                const mainSlider = document.getElementById('volume-slider');
                if (mainSlider) mainSlider.value = e.target.value;
            });
        }

        // Settings panel region select (mirrors main control)
        const settingsRegion = document.getElementById('settings-region');
        if (settingsRegion) {
            settingsRegion.addEventListener('change', (e) => {
                this.setRegion(e.target.value);
                // Sync main control bar select
                const mainSelect = document.getElementById('region-select');
                if (mainSelect) mainSelect.value = e.target.value;
            });
        }

        // Joystick key remapping buttons (T116)
        document.querySelectorAll('.key-remap-btn').forEach(btn => {
            btn.addEventListener('click', (e) => this.startKeyRemapping(e.target));
        });

        // Reset key bindings button
        const resetKeybindingsBtn = document.getElementById('reset-keybindings-btn');
        if (resetKeybindingsBtn) {
            resetKeybindingsBtn.addEventListener('click', () => this.resetJoystickMappings());
        }

        // Clear ROMs button
        const clearRomsBtn = document.getElementById('clear-roms-btn');
        if (clearRomsBtn) {
            clearRomsBtn.addEventListener('click', () => this.clearCachedRoms());
        }

        // Reset all settings button
        const resetSettingsBtn = document.getElementById('reset-settings-btn');
        if (resetSettingsBtn) {
            resetSettingsBtn.addEventListener('click', () => this.resetAllSettings());
        }

        // Listen for key presses when remapping
        document.addEventListener('keydown', (e) => {
            if (this.listeningForKey) {
                e.preventDefault();
                e.stopPropagation();
                this.completeKeyRemapping(e.code);
            }
        });
    }

    /**
     * Open the settings panel
     */
    openSettingsPanel() {
        const panel = document.getElementById('settings-panel');
        const overlay = document.getElementById('settings-overlay');

        if (panel) panel.classList.add('open');
        if (overlay) overlay.classList.add('open');

        this.settingsPanelOpen = true;

        // Sync settings panel with current state
        this.syncSettingsPanel();
    }

    /**
     * Close the settings panel
     */
    closeSettingsPanel() {
        const panel = document.getElementById('settings-panel');
        const overlay = document.getElementById('settings-overlay');

        if (panel) panel.classList.remove('open');
        if (overlay) overlay.classList.remove('open');

        this.settingsPanelOpen = false;

        // Cancel any key remapping in progress
        if (this.listeningForKey) {
            this.cancelKeyRemapping();
        }
    }

    /**
     * Sync settings panel controls with current settings
     */
    syncSettingsPanel() {
        // Scale
        const settingsScale = document.getElementById('settings-scale');
        if (settingsScale) settingsScale.value = this.settings.scale;

        // Scanlines
        const settingsScanlines = document.getElementById('settings-scanlines');
        if (settingsScanlines) settingsScanlines.checked = this.settings.scanlines;

        // Volume
        const settingsVolume = document.getElementById('settings-volume');
        const volumeValue = document.getElementById('settings-volume-value');
        if (settingsVolume) settingsVolume.value = this.settings.volume;
        if (volumeValue) volumeValue.textContent = `${this.settings.volume}%`;

        // Region
        const settingsRegion = document.getElementById('settings-region');
        if (settingsRegion) settingsRegion.value = this.settings.region;

        // Update joystick mapping button labels
        this.updateJoystickMappingUI();
    }

    /**
     * Set scanline effect on/off (T114)
     */
    setScanlines(enabled) {
        this.scanlines = enabled;
        this.settings.scanlines = enabled;

        const displayEl = document.getElementById('c64-display');
        if (displayEl) {
            if (enabled) {
                displayEl.classList.add('scanlines');
            } else {
                displayEl.classList.remove('scanlines');
            }
        }

        this.saveSettings();
        console.log(`Scanlines ${enabled ? 'enabled' : 'disabled'}`);
    }

    /**
     * Override setScale to persist settings (T118)
     */
    setScale(scale) {
        this.scale = scale;
        this.settings.scale = scale;

        const displayEl = document.getElementById('c64-display');
        if (displayEl) {
            // Remove existing scale classes
            displayEl.classList.remove('scale-1x', 'scale-2x', 'scale-3x', 'scale-fit');
            // Add new scale class
            displayEl.classList.add(`scale-${scale}x`);
        }

        this.saveSettings();
        console.log(`Display scale set to: ${scale}`);
    }

    /**
     * Override setVolume to persist settings (T118)
     */
    setVolume(volume) {
        this.volume = Math.max(0, Math.min(1, volume));
        this.settings.volume = Math.round(this.volume * 100);

        if (this.audioWorkletNode) {
            this.audioWorkletNode.port.postMessage({
                type: 'volume',
                value: this.volume
            });
        }

        this.saveSettings();
        console.log(`Volume set to ${Math.round(this.volume * 100)}%`);
    }

    /**
     * Override setRegion to persist settings (T118)
     */
    setRegion(region) {
        this.region = region;
        this.settings.region = region;
        this.saveSettings();
        console.log(`Region set to: ${region.toUpperCase()}`);
    }

    // =========================================================================
    // Joystick Key Remapping (T116)
    // =========================================================================

    /**
     * Start listening for a key to remap
     */
    startKeyRemapping(button) {
        // Cancel any existing remapping
        if (this.listeningForKey) {
            this.cancelKeyRemapping();
        }

        this.listeningForKey = button;
        button.classList.add('listening');
        button.textContent = 'Press a key...';
    }

    /**
     * Complete the key remapping with the pressed key
     */
    completeKeyRemapping(keyCode) {
        if (!this.listeningForKey) return;

        const button = this.listeningForKey;
        const port = button.dataset.port;
        const action = button.dataset.action;

        // Update settings
        const portKey = `port${port}`;
        this.settings.joystickMappings[portKey][action] = keyCode;

        // Update button label
        button.textContent = this.getKeyDisplayName(keyCode);
        button.classList.remove('listening');

        this.listeningForKey = null;

        // Rebuild the key mapping
        this.rebuildJoystickKeyMap();

        this.saveSettings();
        console.log(`Mapped joystick ${port} ${action} to ${keyCode}`);
    }

    /**
     * Cancel key remapping
     */
    cancelKeyRemapping() {
        if (!this.listeningForKey) return;

        const button = this.listeningForKey;
        const port = button.dataset.port;
        const action = button.dataset.action;

        // Restore original label
        const portKey = `port${port}`;
        const currentKey = this.settings.joystickMappings[portKey][action];
        button.textContent = this.getKeyDisplayName(currentKey);
        button.classList.remove('listening');

        this.listeningForKey = null;
    }

    /**
     * Reset joystick mappings to defaults
     */
    resetJoystickMappings() {
        this.settings.joystickMappings = JSON.parse(JSON.stringify(DEFAULT_SETTINGS.joystickMappings));
        this.rebuildJoystickKeyMap();
        this.updateJoystickMappingUI();
        this.saveSettings();
        console.log('Joystick mappings reset to defaults');
    }

    /**
     * Update the joystick mapping UI to show current bindings
     */
    updateJoystickMappingUI() {
        document.querySelectorAll('.key-remap-btn').forEach(btn => {
            const port = btn.dataset.port;
            const action = btn.dataset.action;
            const portKey = `port${port}`;
            const keyCode = this.settings.joystickMappings[portKey][action];
            btn.textContent = this.getKeyDisplayName(keyCode);
        });
    }

    /**
     * Rebuild the joystick key map from settings
     */
    rebuildJoystickKeyMap() {
        // The mapKeyToJoystick function will use this.settings.joystickMappings
        // No additional action needed as mapKeyToJoystick is called dynamically
    }

    /**
     * Override mapKeyToJoystick to use custom mappings (T116)
     */
    mapKeyToJoystick(code) {
        const mappings = this.settings.joystickMappings;

        // Check joystick 2 mappings
        if (code === mappings.port2.up) return { port: 2, bit: JOY_UP };
        if (code === mappings.port2.down) return { port: 2, bit: JOY_DOWN };
        if (code === mappings.port2.left) return { port: 2, bit: JOY_LEFT };
        if (code === mappings.port2.right) return { port: 2, bit: JOY_RIGHT };
        if (code === mappings.port2.fire) return { port: 2, bit: JOY_FIRE };

        // Check joystick 1 mappings
        if (code === mappings.port1.up) return { port: 1, bit: JOY_UP };
        if (code === mappings.port1.down) return { port: 1, bit: JOY_DOWN };
        if (code === mappings.port1.left) return { port: 1, bit: JOY_LEFT };
        if (code === mappings.port1.right) return { port: 1, bit: JOY_RIGHT };
        if (code === mappings.port1.fire) return { port: 1, bit: JOY_FIRE };

        // Also support secondary fire keys that are hard-coded for convenience
        if (code === 'ControlRight' || code === 'Numpad0') return { port: 2, bit: JOY_FIRE };
        if (code === 'ShiftLeft') return { port: 1, bit: JOY_FIRE };

        return null;
    }

    /**
     * Get a human-readable name for a key code
     */
    getKeyDisplayName(code) {
        const keyNames = {
            'ArrowUp': 'Arrow Up',
            'ArrowDown': 'Arrow Down',
            'ArrowLeft': 'Arrow Left',
            'ArrowRight': 'Arrow Right',
            'Space': 'Space',
            'ControlLeft': 'Left Ctrl',
            'ControlRight': 'Right Ctrl',
            'ShiftLeft': 'Left Shift',
            'ShiftRight': 'Right Shift',
            'AltLeft': 'Left Alt',
            'AltRight': 'Right Alt',
            'Numpad0': 'Numpad 0',
            'Numpad1': 'Numpad 1',
            'Numpad2': 'Numpad 2',
            'Numpad3': 'Numpad 3',
            'Numpad4': 'Numpad 4',
            'Numpad5': 'Numpad 5',
            'Numpad6': 'Numpad 6',
            'Numpad7': 'Numpad 7',
            'Numpad8': 'Numpad 8',
            'Numpad9': 'Numpad 9',
            'Enter': 'Enter',
            'Tab': 'Tab',
            'Backspace': 'Backspace',
            'Escape': 'Escape'
        };

        // Check if it's in the map
        if (keyNames[code]) return keyNames[code];

        // Handle letter keys (KeyA -> A)
        if (code.startsWith('Key')) return code.substring(3);

        // Handle digit keys (Digit1 -> 1)
        if (code.startsWith('Digit')) return code.substring(5);

        // Default: return the code as-is
        return code;
    }

    // =========================================================================
    // Settings Persistence (T118)
    // =========================================================================

    /**
     * Load settings from localStorage
     */
    loadSettings() {
        try {
            const saved = localStorage.getItem(SETTINGS_STORAGE_KEY);
            if (saved) {
                const parsed = JSON.parse(saved);
                // Merge with defaults to handle new settings added in updates
                this.settings = {
                    ...DEFAULT_SETTINGS,
                    ...parsed,
                    joystickMappings: {
                        ...DEFAULT_SETTINGS.joystickMappings,
                        ...parsed.joystickMappings
                    }
                };
                console.log('Settings loaded from localStorage');
            }
        } catch (error) {
            console.warn('Failed to load settings:', error);
            this.settings = { ...DEFAULT_SETTINGS };
        }

        // Apply loaded settings to instance variables
        this.scale = this.settings.scale;
        this.scanlines = this.settings.scanlines;
        this.volume = this.settings.volume / 100;
        this.region = this.settings.region;
    }

    /**
     * Save settings to localStorage
     */
    saveSettings() {
        try {
            localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(this.settings));
        } catch (error) {
            console.warn('Failed to save settings:', error);
        }
    }

    /**
     * Apply loaded settings to UI elements
     */
    applySettingsToUI() {
        // Scale select
        const scaleSelect = document.getElementById('scale-select');
        if (scaleSelect) scaleSelect.value = this.settings.scale;
        this.setScale(this.settings.scale);

        // Scanlines checkbox
        const scanlinesCheckbox = document.getElementById('scanlines-checkbox');
        if (scanlinesCheckbox) scanlinesCheckbox.checked = this.settings.scanlines;
        if (this.settings.scanlines) {
            const displayEl = document.getElementById('c64-display');
            if (displayEl) displayEl.classList.add('scanlines');
        }

        // Volume slider
        const volumeSlider = document.getElementById('volume-slider');
        if (volumeSlider) volumeSlider.value = this.settings.volume;

        // Region select
        const regionSelect = document.getElementById('region-select');
        if (regionSelect) regionSelect.value = this.settings.region;
        this.region = this.settings.region;
    }

    /**
     * Clear cached ROMs from localStorage
     */
    clearCachedRoms() {
        if (!confirm('Clear cached ROMs? You will need to upload them again.')) {
            return;
        }

        for (const key of Object.values(ROM_STORAGE_KEYS)) {
            localStorage.removeItem(key);
        }

        // Clear ROM status indicators
        for (const romType of Object.keys(ROM_STORAGE_KEYS)) {
            const statusEl = document.getElementById(`${romType}-status`);
            if (statusEl) {
                statusEl.textContent = '';
                statusEl.className = 'rom-status';
            }
            this.roms[romType] = null;
        }

        this.updateStartButton();
        console.log('Cached ROMs cleared');
    }

    /**
     * Reset all settings to defaults
     */
    resetAllSettings() {
        if (!confirm('Reset all settings to defaults?')) {
            return;
        }

        this.settings = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
        this.saveSettings();
        this.applySettingsToUI();
        this.syncSettingsPanel();
        console.log('All settings reset to defaults');
    }
}

// Initialize the application when the page loads
const app = new C64App();
document.addEventListener('DOMContentLoaded', () => {
    app.init();
});

// Export for debugging and potential use by components
window.c64App = app;
export { C64App, C64_PALETTE, ROM_SIZES, SAVE_SLOT_PREFIX, SAVE_SLOT_COUNT, SETTINGS_STORAGE_KEY, DEFAULT_SETTINGS };
