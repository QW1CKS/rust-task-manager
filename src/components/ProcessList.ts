/**
 * ProcessList Component
 * 
 * Displays all running processes in a sortable, filterable table.
 * Updates every 1-2 seconds via polling service.
 */

import type { ProcessInfo } from '../types/process';
import { formatBytes, formatPercent, getUsageColor } from '../utils/formatters';
import { invokeKillProcess } from '../services/tauri';
import { ConfirmDialogComponent } from './ConfirmDialog';

type SortColumn = 'pid' | 'name' | 'cpuPercent' | 'memoryBytes' | 'status';
type SortOrder = 'asc' | 'desc';

export class ProcessListComponent {
    private container: HTMLElement;
    private processes: ProcessInfo[] = [];
    private filteredProcesses: ProcessInfo[] = [];
    private sortColumn: SortColumn = 'cpuPercent';
    private sortOrder: SortOrder = 'desc';
    private searchQuery: string = '';
    private searchDebounceTimer: number | null = null;
    private contextMenu: HTMLElement | null = null;
    private confirmDialog: ConfirmDialogComponent;
    private onProcessListUpdate?: () => void;

    constructor(containerId: string) {
        const element = document.getElementById(containerId);
        if (!element) {
            throw new Error(`Container element #${containerId} not found`);
        }
        this.container = element;
        this.confirmDialog = new ConfirmDialogComponent();

        // Close context menu on any click outside
        document.addEventListener('click', (e) => {
            if (this.contextMenu && !this.contextMenu.contains(e.target as Node)) {
                this.hideContextMenu();
            }
        });
    }

    /**
     * Set callback for when process list should be refreshed
     */
    setOnProcessListUpdate(callback: () => void): void {
        this.onProcessListUpdate = callback;
    }

    /**
     * Set search query with 300ms debounce
     * 
     * @param query - Search string to filter process names
     */
    setSearchQuery(query: string): void {
        if (this.searchDebounceTimer !== null) {
            window.clearTimeout(this.searchDebounceTimer);
        }

        this.searchDebounceTimer = window.setTimeout(() => {
            this.searchQuery = query.toLowerCase().trim();
            this.applyFilterAndSort();
            this.render();
            this.searchDebounceTimer = null;
        }, 300);
    }

    /**
     * Update process list and re-render
     * 
     * @param processes - New list of processes from backend
     */
    updateProcesses(processes: ProcessInfo[]): void {
        this.processes = processes;
        this.applyFilterAndSort();
        this.render();
    }

    /**
     * Apply search filter and sorting to processes
     */
    private applyFilterAndSort(): void {
        // Filter by search query
        let filtered = this.processes;
        if (this.searchQuery) {
            filtered = this.processes.filter(p =>
                p.name.toLowerCase().includes(this.searchQuery)
            );
        }

        // Sort by current column and order
        filtered.sort((a, b) => {
            let aValue: number | string;
            let bValue: number | string;

            switch (this.sortColumn) {
                case 'pid':
                    aValue = a.pid;
                    bValue = b.pid;
                    break;
                case 'name':
                    aValue = a.name.toLowerCase();
                    bValue = b.name.toLowerCase();
                    break;
                case 'cpuPercent':
                    aValue = a.cpuPercent;
                    bValue = b.cpuPercent;
                    break;
                case 'memoryBytes':
                    aValue = a.memoryBytes;
                    bValue = b.memoryBytes;
                    break;
                case 'status':
                    aValue = a.status;
                    bValue = b.status;
                    break;
            }

            if (aValue < bValue) return this.sortOrder === 'asc' ? -1 : 1;
            if (aValue > bValue) return this.sortOrder === 'asc' ? 1 : -1;
            return 0;
        });

        this.filteredProcesses = filtered;
    }

    /**
     * Handle column header click for sorting
     * 
     * @param column - Column to sort by
     */
    private handleSort(column: SortColumn): void {
        if (this.sortColumn === column) {
            // Toggle order if same column
            this.sortOrder = this.sortOrder === 'asc' ? 'desc' : 'asc';
        } else {
            // Default to descending for numeric columns, ascending for text
            this.sortColumn = column;
            this.sortOrder = column === 'name' || column === 'status' ? 'asc' : 'desc';
        }

        this.applyFilterAndSort();
        this.render();
    }

    /**
     * Render process table
     */
    render(): void {
        const sortIndicator = (col: SortColumn): string => {
            if (this.sortColumn !== col) return '';
            return this.sortOrder === 'asc' ? ' ▲' : ' ▼';
        };

        const tableHTML = `
            <table class="process-table">
                <thead>
                    <tr>
                        <th data-column="pid">PID${sortIndicator('pid')}</th>
                        <th data-column="name">Name${sortIndicator('name')}</th>
                        <th data-column="cpuPercent">CPU %${sortIndicator('cpuPercent')}</th>
                        <th data-column="memoryBytes">Memory${sortIndicator('memoryBytes')}</th>
                        <th data-column="status">Status${sortIndicator('status')}</th>
                    </tr>
                </thead>
                <tbody>
                    ${this.renderRows()}
                </tbody>
            </table>
        `;

        this.container.innerHTML = tableHTML;

        // Attach click handlers to column headers
        const headers = this.container.querySelectorAll('th[data-column]');
        headers.forEach(header => {
            const column = header.getAttribute('data-column') as SortColumn;
            header.addEventListener('click', () => this.handleSort(column));
        });

        // Attach right-click context menu to process rows
        const rows = this.container.querySelectorAll('tbody tr[data-pid]');
        rows.forEach(row => {
            row.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                const pid = parseInt(row.getAttribute('data-pid') || '0');
                const process = this.filteredProcesses.find(p => p.pid === pid);
                if (process) {
                    this.showContextMenu(e as MouseEvent, process);
                }
            });
        });
    }

    /**
     * Render table rows
     * 
     * @returns HTML string with table rows
     */
    private renderRows(): string {
        if (this.filteredProcesses.length === 0) {
            return `
                <tr>
                    <td colspan="5" class="text-muted" style="text-align: center; padding: 2rem;">
                        ${this.searchQuery ? 'No processes match your search' : 'No processes found'}
                    </td>
                </tr>
            `;
        }

        return this.filteredProcesses
            .slice(0, 500) // Limit to 500 for performance (can add virtualization later)
            .map(process => {
                const cpuColor = getUsageColor(process.cpuPercent);
                const memoryColor = process.memoryBytes > 1024 * 1024 * 100 ? 'text-warning' : '';

                return `
                    <tr data-pid="${process.pid}">
                        <td>${process.pid}</td>
                        <td class="process-name" title="${this.escapeHtml(process.name)}">
                            ${this.escapeHtml(process.name)}
                        </td>
                        <td class="${cpuColor}">${formatPercent(process.cpuPercent)}</td>
                        <td class="${memoryColor}">${formatBytes(process.memoryBytes)}</td>
                        <td class="status-${process.status}">${process.status}</td>
                    </tr>
                `;
            })
            .join('');
    }

    /**
     * Show loading state
     */
    showLoading(): void {
        this.container.innerHTML = `
            <div style="text-align: center; padding: 2rem;">
                <p class="text-muted">Loading processes...</p>
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
            <div style="text-align: center; padding: 2rem;">
                <p class="text-error">Error loading processes:</p>
                <p class="text-muted">${this.escapeHtml(message)}</p>
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

    /**
     * Show context menu for a process
     * 
     * @param event - Mouse event with position
     * @param process - Process to show menu for
     */
    private showContextMenu(event: MouseEvent, process: ProcessInfo): void {
        this.hideContextMenu();

        this.contextMenu = document.createElement('div');
        this.contextMenu.className = 'context-menu';
        this.contextMenu.style.left = `${event.pageX}px`;
        this.contextMenu.style.top = `${event.pageY}px`;
        this.contextMenu.innerHTML = `
            <div class="context-menu-item" data-action="kill">
                <span class="context-menu-icon">⛔</span>
                End Process
            </div>
        `;

        // Handle menu item click
        this.contextMenu.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
            const menuItem = target.closest('.context-menu-item') as HTMLElement;
            if (menuItem && menuItem.dataset.action === 'kill') {
                this.handleKillProcess(process);
            }
            this.hideContextMenu();
        });

        document.body.appendChild(this.contextMenu);
    }

    /**
     * Hide context menu
     */
    private hideContextMenu(): void {
        if (this.contextMenu) {
            this.contextMenu.remove();
            this.contextMenu = null;
        }
    }

    /**
     * Handle process termination request
     * 
     * @param process - Process to terminate
     */
    private handleKillProcess(process: ProcessInfo): void {
        // Show standard confirmation dialog first
        this.confirmDialog.show({
            type: 'standard',
            processName: process.name,
            processPid: process.pid,
            onConfirm: () => {
                void this.attemptKillProcess(process);
            },
            onCancel: () => {
                // Process termination cancelled by user
            },
        });
    }

    /**
     * Attempt to terminate a process and handle errors
     * 
     * @param process - Process to terminate
     */
    private async attemptKillProcess(process: ProcessInfo): Promise<void> {
        try {
            await invokeKillProcess(process.pid);

            // Show success message briefly
            this.showTransientMessage(`Process terminated: ${process.name}`, 'success');

            // Refresh process list
            if (this.onProcessListUpdate) {
                this.onProcessListUpdate();
            }
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);

            // Handle different error types per FR-022 and FR-023
            if (errorMessage.includes('critical system process')) {
                // Critical process - show strong warning (FR-023)
                this.confirmDialog.show({
                    type: 'critical',
                    processName: process.name,
                    processPid: process.pid,
                    onConfirm: () => {
                        // User confirmed critical process termination
                        // This shouldn't actually work since backend blocks it
                        void this.attemptKillProcess(process);
                    },
                    onCancel: () => {
                        // Critical process termination cancelled
                    },
                });
            } else if (errorMessage.includes('Administrator privileges') || errorMessage.includes('Permission denied')) {
                // UAC denial - show retry dialog (FR-022)
                this.confirmDialog.show({
                    type: 'uac-denied',
                    processName: process.name,
                    processPid: process.pid,
                    onConfirm: () => {
                        // Retry termination
                        void this.attemptKillProcess(process);
                    },
                    onCancel: () => {
                        // Process termination retry cancelled
                    },
                });
            } else if (errorMessage.includes('not found')) {
                // Process disappeared - show transient message
                this.showTransientMessage(`Process ${process.name} is no longer running`, 'info');

                // Refresh process list
                if (this.onProcessListUpdate) {
                    this.onProcessListUpdate();
                }
            } else {
                // Generic error
                this.showTransientMessage(`Failed to terminate process: ${errorMessage}`, 'error');
            }
        }
    }

    /**
     * Show a transient message that disappears after a few seconds
     * 
     * @param message - Message to display
     * @param type - Message type ('success', 'error', 'info')
     */
    private showTransientMessage(message: string, type: 'success' | 'error' | 'info'): void {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.textContent = message;
        toast.style.cssText = `
            position: fixed;
            bottom: 20px;
            right: 20px;
            padding: 1rem 1.5rem;
            border-radius: 6px;
            background-color: ${type === 'success' ? '#10b981' : type === 'error' ? '#ef4444' : '#3b82f6'};
            color: white;
            font-weight: 500;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            z-index: 10000;
            animation: slideInUp 0.3s ease-out;
        `;

        document.body.appendChild(toast);

        // Remove after 3 seconds
        setTimeout(() => {
            toast.style.animation = 'slideOutDown 0.3s ease-in';
            setTimeout(() => toast.remove(), 300);
        }, 3000);
    }
}
