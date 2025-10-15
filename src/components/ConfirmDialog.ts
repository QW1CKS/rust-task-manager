/**
 * ConfirmDialog Component
 * 
 * Provides three types of confirmation dialogs:
 * 1. Standard confirmation (data loss warning per FR-006)
 * 2. Critical process warning (strong warning per FR-023)
 * 3. UAC denial retry dialog (informative message per FR-022)
 */

export type DialogType = 'standard' | 'critical' | 'uac-denied';

export interface DialogOptions {
    type: DialogType;
    processName: string;
    processPid: number;
    onConfirm: () => void;
    onCancel: () => void;
}

export class ConfirmDialogComponent {
    private containerElement: HTMLElement | null = null;
    private currentOptions: DialogOptions | null = null;

    /**
     * Show a confirmation dialog
     * 
     * @param options - Dialog configuration
     */
    show(options: DialogOptions): void {
        this.currentOptions = options;
        this.render();

        // Focus on cancel button by default (especially for critical process dialog per FR-023)
        setTimeout(() => {
            const cancelButton = document.querySelector('.dialog-cancel-button') as HTMLButtonElement;
            cancelButton?.focus();
        }, 50);
    }

    /**
     * Hide the confirmation dialog
     */
    hide(): void {
        if (this.containerElement) {
            this.containerElement.remove();
            this.containerElement = null;
        }
        this.currentOptions = null;
    }

    /**
     * Render the appropriate dialog based on type
     */
    private render(): void {
        // Remove existing dialog if any
        this.hide();

        if (!this.currentOptions) return;

        const { type, processName, processPid } = this.currentOptions;

        // Create modal overlay
        this.containerElement = document.createElement('div');
        this.containerElement.className = 'dialog-overlay';
        this.containerElement.innerHTML = this.getDialogHTML(type, processName, processPid);

        // Add event listeners
        this.containerElement.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;

            if (target.classList.contains('dialog-overlay')) {
                // Clicked outside dialog - treat as cancel
                this.handleCancel();
            } else if (target.classList.contains('dialog-confirm-button')) {
                this.handleConfirm();
            } else if (target.classList.contains('dialog-cancel-button')) {
                this.handleCancel();
            }
        });

        // ESC key to cancel
        const escHandler = (e: KeyboardEvent): void => {
            if (e.key === 'Escape') {
                this.handleCancel();
                document.removeEventListener('keydown', escHandler);
            }
        };
        document.addEventListener('keydown', escHandler);

        document.body.appendChild(this.containerElement);
    }

    /**
     * Generate HTML for the dialog based on type
     */
    private getDialogHTML(type: DialogType, processName: string, processPid: number): string {
        if (type === 'standard') {
            return this.getStandardDialogHTML(processName, processPid);
        } else if (type === 'critical') {
            return this.getCriticalDialogHTML(processName, processPid);
        } else if (type === 'uac-denied') {
            return this.getUACDeniedDialogHTML(processName, processPid);
        }
        return '';
    }

    /**
     * Standard confirmation dialog (FR-006)
     */
    private getStandardDialogHTML(processName: string, processPid: number): string {
        return `
            <div class="dialog-content" role="dialog" aria-labelledby="dialog-title" aria-describedby="dialog-description">
                <div class="dialog-header">
                    <h3 id="dialog-title">End Process</h3>
                </div>
                <div class="dialog-body">
                    <p id="dialog-description">
                        Are you sure you want to terminate <strong>${this.escapeHTML(processName)}</strong> (PID: ${processPid})?
                    </p>
                    <p class="dialog-warning">
                        ⚠️ <strong>Warning:</strong> Terminating this process may cause data loss if it has unsaved work.
                    </p>
                </div>
                <div class="dialog-footer">
                    <button class="dialog-cancel-button" type="button">Cancel</button>
                    <button class="dialog-confirm-button dialog-confirm-danger" type="button">End Process</button>
                </div>
            </div>
        `;
    }

    /**
     * Critical process warning dialog (FR-023)
     */
    private getCriticalDialogHTML(processName: string, processPid: number): string {
        return `
            <div class="dialog-content dialog-critical" role="dialog" aria-labelledby="dialog-title" aria-describedby="dialog-description">
                <div class="dialog-header">
                    <h3 id="dialog-title">⚠️ Critical System Process</h3>
                </div>
                <div class="dialog-body">
                    <p id="dialog-description" class="dialog-critical-warning">
                        <strong>WARNING:</strong> <strong>${this.escapeHTML(processName)}</strong> (PID: ${processPid}) is a critical system process.
                    </p>
                    <p class="dialog-critical-message">
                        Terminating this process will cause:
                    </p>
                    <ul class="dialog-critical-list">
                        <li>Immediate system instability</li>
                        <li>Possible data loss across all applications</li>
                        <li>Potential system crash or shutdown</li>
                    </ul>
                    <p class="dialog-critical-question">
                        <strong>Are you absolutely sure you want to proceed?</strong>
                    </p>
                </div>
                <div class="dialog-footer">
                    <button class="dialog-cancel-button" type="button" autofocus>Cancel</button>
                    <button class="dialog-confirm-button dialog-confirm-critical" type="button">I Understand, Terminate</button>
                </div>
            </div>
        `;
    }

    /**
     * UAC denial retry dialog (FR-022)
     */
    private getUACDeniedDialogHTML(processName: string, processPid: number): string {
        return `
            <div class="dialog-content" role="dialog" aria-labelledby="dialog-title" aria-describedby="dialog-description">
                <div class="dialog-header">
                    <h3 id="dialog-title">Administrator Privileges Required</h3>
                </div>
                <div class="dialog-body">
                    <p id="dialog-description">
                        Cannot terminate <strong>${this.escapeHTML(processName)}</strong> (PID: ${processPid}):
                    </p>
                    <p class="dialog-info">
                        Administrator privileges are required but were not granted.
                    </p>
                    <p>
                        Would you like to try again?
                    </p>
                </div>
                <div class="dialog-footer">
                    <button class="dialog-cancel-button" type="button">Cancel</button>
                    <button class="dialog-confirm-button" type="button">Retry</button>
                </div>
            </div>
        `;
    }

    /**
     * Escape HTML to prevent XSS
     */
    private escapeHTML(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Handle confirm button click
     */
    private handleConfirm(): void {
        if (this.currentOptions) {
            this.currentOptions.onConfirm();
        }
        this.hide();
    }

    /**
     * Handle cancel button click
     */
    private handleCancel(): void {
        if (this.currentOptions) {
            this.currentOptions.onCancel();
        }
        this.hide();
    }
}
