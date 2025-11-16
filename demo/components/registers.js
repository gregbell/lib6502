/**
 * Register Display Component
 * Shows CPU registers (A, X, Y, PC, SP) and cycle count
 */

export class RegisterDisplay {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.render();
    }

    render() {
        this.container.innerHTML = `
            <div class="registers-grid">
                <div class="register-row">
                    <div class="register-item">
                        <span class="register-label">A:</span>
                        <span class="register-value" id="reg-a">00</span>
                    </div>
                    <div class="register-item">
                        <span class="register-label">X:</span>
                        <span class="register-value" id="reg-x">00</span>
                    </div>
                    <div class="register-item">
                        <span class="register-label">Y:</span>
                        <span class="register-value" id="reg-y">00</span>
                    </div>
                </div>
                <div class="register-row">
                    <div class="register-item">
                        <span class="register-label">PC:</span>
                        <span class="register-value" id="reg-pc">0000</span>
                    </div>
                    <div class="register-item">
                        <span class="register-label">SP:</span>
                        <span class="register-value" id="reg-sp">00</span>
                    </div>
                </div>
                <div class="register-row">
                    <div class="register-item wide">
                        <span class="register-label">Cycles:</span>
                        <span class="register-value" id="reg-cycles">0</span>
                    </div>
                </div>
            </div>
        `;
    }

    update(registers) {
        document.getElementById('reg-a').textContent = this.formatHex(registers.a, 2);
        document.getElementById('reg-x').textContent = this.formatHex(registers.x, 2);
        document.getElementById('reg-y').textContent = this.formatHex(registers.y, 2);
        document.getElementById('reg-pc').textContent = this.formatHex(registers.pc, 4);
        document.getElementById('reg-sp').textContent = this.formatHex(registers.sp, 2);
        document.getElementById('reg-cycles').textContent = this.formatNumber(registers.cycles);
    }

    formatHex(value, digits) {
        return value.toString(16).toUpperCase().padStart(digits, '0');
    }

    formatNumber(value) {
        return value.toLocaleString();
    }
}
