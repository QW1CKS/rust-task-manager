/**
 * SystemInfo Component
 * 
 * Displays static system hardware and software information in a grid layout.
 * Rendered once at application startup.
 */

import type { SystemInfo } from '../types/system';
import { formatBytes } from '../utils/formatters';

export class SystemInfoComponent {
    private container: HTMLElement;

    constructor(containerId: string) {
        const element = document.getElementById(containerId);
        if (!element) {
            throw new Error(`Container element #${containerId} not found`);
        }
        this.container = element;
    }

    /**
     * Render system information in grid layout
     * 
     * @param info - System information from backend
     */
    render(info: SystemInfo): void {
        this.container.innerHTML = `
      <div class="info-item">
        <span class="info-label">Operating System</span>
        <span class="info-value">${this.escapeHtml(info.osName)} ${this.escapeHtml(info.osVersion)}</span>
      </div>
      
      <div class="info-item">
        <span class="info-label">Kernel Version</span>
        <span class="info-value">${this.escapeHtml(info.kernelVersion)}</span>
      </div>
      
      <div class="info-item">
        <span class="info-label">CPU Model</span>
        <span class="info-value">${this.escapeHtml(info.cpuModel)}</span>
      </div>
      
      <div class="info-item">
        <span class="info-label">CPU Architecture</span>
        <span class="info-value">${this.escapeHtml(info.cpuArchitecture)}</span>
      </div>
      
      <div class="info-item">
        <span class="info-label">CPU Cores</span>
        <span class="info-value">${info.cpuCores} cores</span>
      </div>
      
      <div class="info-item">
        <span class="info-label">Total Memory</span>
        <span class="info-value">${formatBytes(info.totalMemory)}</span>
      </div>
      
      <div class="info-item">
        <span class="info-label">Hostname</span>
        <span class="info-value">${this.escapeHtml(info.hostname)}</span>
      </div>
    `;
    }

    /**
     * Show loading state
     */
    showLoading(): void {
        this.container.innerHTML = `
      <div class="info-item">
        <span class="info-label">Loading...</span>
        <span class="info-value">Please wait</span>
      </div>
    `;
    }

    /**
     * Show error state
     * 
     * @param error - Error message or Error object
     */
    showError(error: string | Error): void {
        const message = error instanceof Error ? error.message : error;
        this.container.innerHTML = `
      <div class="info-item">
        <span class="info-label text-error">Error</span>
        <span class="info-value text-error">${this.escapeHtml(message)}</span>
      </div>
    `;
    }

    /**
     * Escape HTML to prevent XSS
     * 
     * @param text - Raw text to escape
     * @returns HTML-safe text
     */
    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}
