/**
 * lib6502 Web Demo - Main Application
 * Interactive 6502 assembly playground
 */

import init, { Emulator6502 } from './lib6502_wasm/lib6502.js';
import { CodeEditor } from './components/editor.js';
import { RegisterDisplay } from './components/registers.js';
import { FlagsDisplay } from './components/flags.js';
import { MemoryViewer } from './components/memory.js';
import { ControlPanel } from './components/controls.js';
import { ErrorDisplay } from './components/error.js';
import { ExampleSelector } from './components/examples.js';

class App {
    constructor() {
        this.emulator = null;
        this.editor = null;
        this.registerDisplay = null;
        this.flagsDisplay = null;
        this.memoryViewer = null;
        this.controlPanel = null;
        this.errorDisplay = null;
        this.exampleSelector = null;

        this.mode = 'idle'; // idle, running, stepping
        this.assembled = false;
        this.programStart = 0x0600;
        this.programEnd = 0x0600;
        this.speed = 1000000; // 1 MHz default
        this.animationFrameId = null;
    }

    async init() {
        try {
            // Initialize WASM module
            await init();
            this.emulator = new Emulator6502();

            // Initialize UI components
            this.editor = new CodeEditor('editor-container');
            this.registerDisplay = new RegisterDisplay('registers-container');
            this.flagsDisplay = new FlagsDisplay('flags-container');
            this.memoryViewer = new MemoryViewer('memory-container');
            this.controlPanel = new ControlPanel('controls-container');
            this.errorDisplay = new ErrorDisplay('error-container');
            this.exampleSelector = new ExampleSelector(this.editor);

            // Set up event listeners
            this.setupEventListeners();

            // Initial display update
            this.updateDisplay();

            // Start animation loop
            this.startAnimationLoop();

            console.log('✓ lib6502 demo initialized successfully');
        } catch (error) {
            console.error('Failed to initialize demo:', error);
            this.showError('Failed to load WebAssembly module. Please refresh the page.');
        }
    }

    setupEventListeners() {
        // Control panel events
        document.addEventListener('assemble-clicked', () => this.handleAssemble());
        document.addEventListener('run-clicked', () => this.handleRun());
        document.addEventListener('step-clicked', () => this.handleStep());
        document.addEventListener('stop-clicked', () => this.handleStop());
        document.addEventListener('reset-clicked', () => this.handleReset());
        document.addEventListener('speed-changed', (e) => this.handleSpeedChange(e.detail.speed));

        // Editor events
        document.addEventListener('code-changed', () => {
            this.assembled = false;
            this.controlPanel.setAssembled(false);
            this.errorDisplay.clear();
        });

        // Example events
        document.addEventListener('example-loaded', (e) => {
            this.assembled = false;
            this.controlPanel.setAssembled(false);
            this.errorDisplay.clear();
            this.handleReset();
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => this.handleKeyboard(e));
    }

    handleKeyboard(e) {
        // Only handle shortcuts when not typing in editor
        if (e.target.tagName === 'TEXTAREA') return;

        switch (e.key) {
            case ' ':
                e.preventDefault();
                if (this.assembled && this.mode !== 'running') {
                    this.handleStep();
                }
                break;
            case 'r':
            case 'R':
                if (this.assembled && this.mode !== 'running') {
                    this.handleRun();
                }
                break;
            case 's':
            case 'S':
                if (this.mode === 'running') {
                    this.handleStop();
                }
                break;
            case 'Escape':
                if (this.assembled) {
                    this.handleReset();
                }
                break;
        }
    }

    handleAssemble() {
        const code = this.editor.getValue();
        if (!code.trim()) {
            this.showError('No code to assemble');
            return;
        }

        try {
            const result = this.emulator.assemble_and_load(code, this.programStart);
            this.programEnd = result.end_addr;
            this.assembled = true;
            this.controlPanel.setAssembled(true);
            this.errorDisplay.clear();
            this.emulator.set_pc(this.programStart);
            this.updateDisplay();
            console.log(`✓ Assembled program: $${this.programStart.toString(16)} - $${this.programEnd.toString(16)}`);
        } catch (error) {
            this.showError(error.toString());
            this.assembled = false;
            this.controlPanel.setAssembled(false);
        }
    }

    handleRun() {
        if (!this.assembled) {
            this.showError('Please assemble the code first');
            return;
        }

        this.mode = 'running';
        this.controlPanel.setMode('running');
        this.errorDisplay.clear();
    }

    handleStep() {
        if (!this.assembled) {
            this.showError('Please assemble the code first');
            return;
        }

        try {
            this.mode = 'stepping';
            this.controlPanel.setMode('stepping');
            this.emulator.step();
            this.updateDisplay();

            // Check if program completed
            if (this.isProgramComplete()) {
                console.log('✓ Program completed');
                this.mode = 'idle';
                this.controlPanel.setMode('idle');
            } else {
                this.mode = 'idle';
                this.controlPanel.setMode('idle');
            }
        } catch (error) {
            this.showError(`Execution error: ${error}`);
            this.mode = 'idle';
            this.controlPanel.setMode('idle');
        }
    }

    handleStop() {
        this.mode = 'idle';
        this.controlPanel.setMode('idle');
        this.updateDisplay();
        console.log('✓ Execution stopped');
    }

    handleReset() {
        this.emulator.reset();
        if (this.assembled) {
            this.emulator.set_pc(this.programStart);
        }
        this.mode = 'idle';
        this.controlPanel.setMode('idle');
        this.updateDisplay();
        this.errorDisplay.clear();
        console.log('✓ CPU reset');
    }

    handleSpeedChange(speed) {
        this.speed = speed;
        console.log(`Speed changed to ${speed === -1 ? 'unlimited' : (speed / 1000000) + ' MHz'}`);
    }

    showError(message) {
        this.errorDisplay.show(message);
        console.error(message);
    }

    isProgramComplete() {
        const pc = this.emulator.pc;
        return pc >= this.programEnd;
    }

    updateDisplay() {
        if (!this.emulator) return;

        // Update registers
        this.registerDisplay.update({
            a: this.emulator.a,
            x: this.emulator.x,
            y: this.emulator.y,
            pc: this.emulator.pc,
            sp: this.emulator.sp,
            cycles: this.emulator.cycles
        });

        // Update flags
        this.flagsDisplay.update({
            n: this.emulator.flag_n,
            v: this.emulator.flag_v,
            d: this.emulator.flag_d,
            i: this.emulator.flag_i,
            z: this.emulator.flag_z,
            c: this.emulator.flag_c
        });

        // Update memory viewer (visible pages only)
        this.memoryViewer.update(this.emulator);
    }

    startAnimationLoop() {
        const loop = () => {
            if (this.mode === 'running') {
                this.runCycles();
            }
            this.updateDisplay();
            this.animationFrameId = requestAnimationFrame(loop);
        };
        loop();
    }

    runCycles() {
        try {
            // Calculate cycles per frame based on speed (60fps)
            const cyclesPerFrame = this.speed === -1 ? 100000 : Math.floor(this.speed / 60);

            const executedCycles = this.emulator.run_for_cycles(cyclesPerFrame);

            // Check if program completed
            if (this.isProgramComplete()) {
                console.log('✓ Program completed');
                this.mode = 'idle';
                this.controlPanel.setMode('idle');
            }
        } catch (error) {
            this.showError(`Runtime error: ${error}`);
            this.mode = 'idle';
            this.controlPanel.setMode('idle');
        }
    }
}

// Initialize application when page loads
const app = new App();
app.init();

// Expose for debugging
window.app = app;
