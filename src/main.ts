/**
 * Rust Task Manager - Main Entry Point
 * 
 * Initializes the application and wires up all components:
 * 1. Shows loading state
 * 2. Fetches system info (once)
 * 3. Starts performance polling (continuous)
 * 4. Handles errors gracefully
 */

import './style.css';
import { SystemInfoComponent } from './components/SystemInfo';
import { PerformanceMetricsComponent } from './components/PerformanceMetrics';
import { ProcessListComponent } from './components/ProcessList';
import { invokeGetSystemInfo } from './services/tauri';
import {
    startPerformancePolling,
    subscribeToPerformance,
    startProcessPolling,
    subscribeToProcessList,
} from './services/performance';

// Initialize components
const systemInfoComponent = new SystemInfoComponent('system-info');
const performanceComponent = new PerformanceMetricsComponent('performance');
const processListComponent = new ProcessListComponent('process-list');

/**
 * Application initialization
 */
async function initializeApp(): Promise<void> {


    // Show loading states (FR-024)
    systemInfoComponent.showLoading();
    performanceComponent.showLoading();

    try {
        // Fetch static system information (called once at startup)

        const systemInfo = await invokeGetSystemInfo();
        systemInfoComponent.render(systemInfo);


    } catch (error) {

        console.error('[App] Failed to load system info:', error);
        systemInfoComponent.showError(error instanceof Error ? error : String(error));
    }

    // Start performance polling and subscribe to updates
    startPerformancePolling();

    subscribeToPerformance((metrics, error) => {
        if (error) {
            // Show error per FR-021
            performanceComponent.showError(error);
        } else if (metrics) {
            // Update UI with latest metrics
            performanceComponent.render(metrics);
        }
    });

    // Start process list polling and subscribe to updates
    processListComponent.showLoading();
    startProcessPolling();

    subscribeToProcessList((processes, error) => {
        if (error) {
            // Show error if process list fails
            processListComponent.showError(error);
        } else if (processes) {
            // Update process list table
            processListComponent.updateProcesses(processes);
        }
    });

    // Set process list update callback (for manual refresh after termination)
    processListComponent.setOnProcessListUpdate(() => {
        // Manually trigger process list refresh
        startProcessPolling();
    });

    // Wire up search box
    const searchInput = document.getElementById('process-search') as HTMLInputElement;
    if (searchInput) {
        searchInput.addEventListener('input', (e) => {
            const target = e.target as HTMLInputElement;
            processListComponent.setSearchQuery(target.value);
        });
    }
}

// Start application when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => void initializeApp());
} else {
    void initializeApp();
}
