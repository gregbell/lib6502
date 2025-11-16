/**
 * Control Panel Component
 * Assemble, Run, Step, Stop, Reset buttons and speed control
 */

export class ControlPanel {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.mode = 'idle'; // idle, running, stepping
        this.assembled = false;
        this.render();
        this.setupEventListeners();
    }

    render() {
        this.container.innerHTML = `
            <div class="controls-panel">
                <div class="control-group">
                    <button id="btn-assemble" class="btn btn-primary">Assemble</button>
                    <button id="btn-run" class="btn btn-success" disabled>Run</button>
                    <button id="btn-step" class="btn btn-info" disabled>Step</button>
                    <button id="btn-stop" class="btn btn-warning" disabled>Stop</button>
                    <button id="btn-reset" class="btn btn-secondary">Reset</button>
                </div>
                <div class="control-group speed-control">
                    <label for="speed-select">Speed:</label>
                    <select id="speed-select">
                        <option value="500000">0.5 MHz (Slow)</option>
                        <option value="1000000" selected>1 MHz (Authentic)</option>
                        <option value="1790000">1.79 MHz (NES/C64)</option>
                        <option value="2000000">2 MHz (Apple IIgs)</option>
                        <option value="-1">Unlimited</option>
                    </select>
                </div>
            </div>
        `;
    }

    setupEventListeners() {
        document.getElementById('btn-assemble').addEventListener('click', () => {
            document.dispatchEvent(new CustomEvent('assemble-clicked'));
        });

        document.getElementById('btn-run').addEventListener('click', () => {
            document.dispatchEvent(new CustomEvent('run-clicked'));
        });

        document.getElementById('btn-step').addEventListener('click', () => {
            document.dispatchEvent(new CustomEvent('step-clicked'));
        });

        document.getElementById('btn-stop').addEventListener('click', () => {
            document.dispatchEvent(new CustomEvent('stop-clicked'));
        });

        document.getElementById('btn-reset').addEventListener('click', () => {
            document.dispatchEvent(new CustomEvent('reset-clicked'));
        });

        document.getElementById('speed-select').addEventListener('change', (e) => {
            const speed = parseInt(e.target.value);
            document.dispatchEvent(new CustomEvent('speed-changed', { detail: { speed } }));
        });
    }

    setMode(mode) {
        this.mode = mode;
        this.updateButtons();
    }

    setAssembled(assembled) {
        this.assembled = assembled;
        this.updateButtons();
    }

    updateButtons() {
        const btnAssemble = document.getElementById('btn-assemble');
        const btnRun = document.getElementById('btn-run');
        const btnStep = document.getElementById('btn-step');
        const btnStop = document.getElementById('btn-stop');
        const btnReset = document.getElementById('btn-reset');

        // Assemble always enabled
        btnAssemble.disabled = false;

        // Run and Step enabled when assembled and not running
        btnRun.disabled = !this.assembled || this.mode === 'running';
        btnStep.disabled = !this.assembled || this.mode === 'running';

        // Stop enabled only when running
        btnStop.disabled = this.mode !== 'running';

        // Reset always enabled
        btnReset.disabled = false;
    }

    getSpeed() {
        return parseInt(document.getElementById('speed-select').value);
    }
}
