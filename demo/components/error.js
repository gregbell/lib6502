/**
 * Error Display Component
 * Shows assembly and runtime errors
 */

export class ErrorDisplay {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.render();
    }

    render() {
        this.container.innerHTML = `
            <div id="error-message" class="error-message hidden"></div>
        `;
        this.errorElement = document.getElementById('error-message');
    }

    show(message) {
        this.errorElement.textContent = message;
        this.errorElement.classList.remove('hidden');
    }

    clear() {
        this.errorElement.textContent = '';
        this.errorElement.classList.add('hidden');
    }
}
