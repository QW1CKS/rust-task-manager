import type { SystemInfo } from '../types/system';

/**
 * Format bytes to human-readable string
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

/**
 * Create an info item element
 */
function createInfoItem(label: string, value: string): HTMLElement {
  const item = document.createElement('div');
  item.className = 'info-item';

  const labelEl = document.createElement('span');
  labelEl.className = 'info-label';
  labelEl.textContent = label;

  const valueEl = document.createElement('span');
  valueEl.className = 'info-value';
  valueEl.textContent = value;

  item.appendChild(labelEl);
  item.appendChild(valueEl);

  return item;
}

/**
 * Render system information to the DOM
 * @param systemInfo - The system information to display
 * @param containerId - The ID of the container element (default: 'system-details')
 */
export function renderSystemInfo(
  systemInfo: SystemInfo,
  containerId: string = 'system-details'
): void {
  const container = document.getElementById(containerId);
  if (!container) {
    console.error(`Container element with ID '${containerId}' not found`);
    return;
  }

  // Clear previous content
  container.innerHTML = '';

  // Create info items
  const items = [
    createInfoItem('Operating System', systemInfo.osName),
    createInfoItem('OS Version', systemInfo.osVersion),
    createInfoItem('Hostname', systemInfo.hostname),
    createInfoItem('CPU Model', systemInfo.cpuModel),
    createInfoItem('CPU Architecture', systemInfo.cpuArchitecture),
    createInfoItem('CPU Cores', systemInfo.cpuCores.toString()),
    createInfoItem('Total Memory', formatBytes(systemInfo.totalMemory)),
  ];

  // Append all items to container
  items.forEach((item) => container.appendChild(item));
}

/**
 * Show loading state in system info container
 */
export function showSystemInfoLoading(containerId: string = 'system-details'): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  container.innerHTML = '<p style="color: var(--text-secondary)">Loading system information...</p>';
}

/**
 * Show error state in system info container
 */
export function showSystemInfoError(error: string, containerId: string = 'system-details'): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  container.innerHTML = `<p style="color: var(--danger)">Error: ${error}</p>`;
}
