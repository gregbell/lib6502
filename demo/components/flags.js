/**
 * Flags Display Component
 * Shows processor status flags (N, V, D, I, Z, C)
 */

export class FlagsDisplay {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.render();
    }

    render() {
        this.container.innerHTML = `
            <div class="flags-container">
                <h3>Status Flags</h3>
                <div class="flags-grid">
                    <div class="flag-item" id="flag-n">
                        <span class="flag-label">N</span>
                        <span class="flag-value">0</span>
                    </div>
                    <div class="flag-item" id="flag-v">
                        <span class="flag-label">V</span>
                        <span class="flag-value">0</span>
                    </div>
                    <div class="flag-item" id="flag-d">
                        <span class="flag-label">D</span>
                        <span class="flag-value">0</span>
                    </div>
                    <div class="flag-item" id="flag-i">
                        <span class="flag-label">I</span>
                        <span class="flag-value">0</span>
                    </div>
                    <div class="flag-item" id="flag-z">
                        <span class="flag-label">Z</span>
                        <span class="flag-value">0</span>
                    </div>
                    <div class="flag-item" id="flag-c">
                        <span class="flag-label">C</span>
                        <span class="flag-value">0</span>
                    </div>
                </div>
            </div>
        `;
    }

    update(flags) {
        this.updateFlag('n', flags.n);
        this.updateFlag('v', flags.v);
        this.updateFlag('d', flags.d);
        this.updateFlag('i', flags.i);
        this.updateFlag('z', flags.z);
        this.updateFlag('c', flags.c);
    }

    updateFlag(name, value) {
        const element = document.getElementById(`flag-${name}`);
        const valueSpan = element.querySelector('.flag-value');
        valueSpan.textContent = value ? '1' : '0';

        if (value) {
            element.classList.add('flag-set');
        } else {
            element.classList.remove('flag-set');
        }
    }
}
