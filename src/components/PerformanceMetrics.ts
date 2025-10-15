/**
 * PerformanceMetrics Component
 * 
 * Displays real-time performance metrics with color-coded indicators.
 * Updates every 1-2 seconds via polling service.
 */

import type { PerformanceMetrics } from '../types/performance';
import { formatBytes, formatPercent, formatSpeed, getUsageColor } from '../utils/formatters';

export class PerformanceMetricsComponent {
    private container: HTMLElement;

    constructor(containerId: string) {
        const element = document.getElementById(containerId);
        if (!element) {
            throw new Error(`Container element #${containerId} not found`);
        }
        this.container = element;
    }

    /**
     * Render performance metrics with color coding
     * 
     * @param metrics - Current performance metrics from backend
     */
    render(metrics: PerformanceMetrics): void {
        const cpuColor = getUsageColor(metrics.cpuUsagePercent);
        const memoryColor = getUsageColor(metrics.memoryPercent);

        this.container.innerHTML = `
      <div class="metric">
        <div class="metric-header">
          <span class="metric-label">CPU Usage</span>
          <span class="metric-value ${cpuColor}">${formatPercent(metrics.cpuUsagePercent)}</span>
        </div>
        <div class="metric-details">
          ${this.renderCpuCores(metrics.cpuPerCore)}
        </div>
      </div>
      
      <div class="metric">
        <div class="metric-header">
          <span class="metric-label">Memory Usage</span>
          <span class="metric-value ${memoryColor}">${formatPercent(metrics.memoryPercent)}</span>
        </div>
        <div class="metric-details">
          <span class="text-muted">${formatBytes(metrics.memoryUsed)} / ${formatBytes(metrics.memoryTotal)}</span>
        </div>
      </div>
      
      <div class="metric">
        <div class="metric-header">
          <span class="metric-label">Disk I/O</span>
          <span class="metric-value">
            <span class="text-muted">R:</span> ${formatSpeed(metrics.diskReadBps)}
            <span class="text-muted">W:</span> ${formatSpeed(metrics.diskWriteBps)}
          </span>
        </div>
      </div>
      
      <div class="metric">
        <div class="metric-header">
          <span class="metric-label">Network</span>
          <span class="metric-value">
            <span class="text-muted">↑</span> ${formatSpeed(metrics.networkUploadBps)}
            <span class="text-muted">↓</span> ${formatSpeed(metrics.networkDownloadBps)}
          </span>
        </div>
      </div>
    `;
    }

    /**
     * Render per-core CPU usage (shows first 8 cores if more)
     * 
     * @param cores - Array of per-core usage percentages
     * @returns HTML string with core usage indicators
     */
    private renderCpuCores(cores: number[]): string {
        const displayCores = cores.slice(0, 8); // Show max 8 cores
        const items = displayCores.map((usage, index) => {
            const color = getUsageColor(usage);
            return `<span class="core-usage ${color}">Core ${index}: ${formatPercent(usage, 0)}</span>`;
        });

        if (cores.length > 8) {
            items.push(`<span class="text-muted">+${cores.length - 8} more</span>`);
        }

        return items.join(' ');
    }

    /**
     * Show loading state
     */
    showLoading(): void {
        this.container.innerHTML = `
      <div class="metric">
        <div class="metric-header">
          <span class="metric-label">Loading...</span>
          <span class="metric-value text-muted">--</span>
        </div>
      </div>
    `;
    }

    /**
     * Show error state per FR-021
     * 
     * @param error - Error message or Error object
     */
    showError(error: string | Error): void {
        const message = error instanceof Error ? error.message : error;
        this.container.innerHTML = `
      <div class="metric">
        <div class="metric-header">
          <span class="metric-label text-error">Error</span>
          <span class="metric-value text-error">${this.escapeHtml(message)}</span>
        </div>
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
