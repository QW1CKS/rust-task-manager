/**
 * Data Formatting Utilities
 * 
 * Utilities for formatting system metrics for display:
 * - Byte sizes (KB, MB, GB, TB)
 * - Percentages
 * - Speed rates (bps, KB/s, MB/s, GB/s)
 * - Color coding based on usage thresholds
 */

/**
 * Format bytes to human-readable string (KB, MB, GB, TB)
 * 
 * @param bytes - Number of bytes
 * @param decimals - Number of decimal places (default: 2)
 * @returns Formatted string like "1.23 GB"
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
    if (bytes === 0) return '0 B';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const value = bytes / Math.pow(k, i);

    return `${value.toFixed(dm)} ${sizes[i]}`;
}

/**
 * Format percentage with fixed decimal places
 * 
 * @param value - Percentage value (0-100)
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted string like "45.3%"
 */
export function formatPercent(value: number, decimals: number = 1): string {
    return `${value.toFixed(decimals)}%`;
}

/**
 * Format speed in bytes/second to human-readable rate
 * 
 * @param bytesPerSecond - Speed in bytes per second
 * @param decimals - Number of decimal places (default: 2)
 * @returns Formatted string like "1.23 MB/s"
 */
export function formatSpeed(bytesPerSecond: number, decimals: number = 2): string {
    if (bytesPerSecond === 0) return '0 B/s';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];

    const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
    const value = bytesPerSecond / Math.pow(k, i);

    return `${value.toFixed(dm)} ${sizes[i]}`;
}

/**
 * Get color class based on usage percentage
 * 
 * Color coding:
 * - Green: < 50% (low usage)
 * - Yellow: 50-80% (moderate usage)
 * - Red: > 80% (high usage)
 * 
 * @param percent - Usage percentage (0-100)
 * @returns CSS class name ('text-success', 'text-warning', or 'text-error')
 */
export function getUsageColor(percent: number): string {
    if (percent < 50) {
        return 'text-success';
    } else if (percent < 80) {
        return 'text-warning';
    } else {
        return 'text-error';
    }
}

/**
 * Format memory bytes to most appropriate unit with color coding
 * 
 * @param used - Used memory in bytes
 * @param total - Total memory in bytes
 * @returns Object with formatted string and color class
 */
export function formatMemory(used: number, total: number): { text: string; color: string } {
    const percent = (used / total) * 100;
    return {
        text: `${formatBytes(used)} / ${formatBytes(total)} (${formatPercent(percent)})`,
        color: getUsageColor(percent)
    };
}
