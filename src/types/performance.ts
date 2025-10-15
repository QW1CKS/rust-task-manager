/**
 * Performance metrics interface matching Rust PerformanceMetrics struct
 *
 * Represents a real-time resource utilization snapshot.
 * Collected every 1-2 seconds.
 */
export interface PerformanceMetrics {
  timestamp: number; // Unix epoch milliseconds
  cpuUsagePercent: number; // 0-100
  cpuPerCore: number[]; // Array of 0-100 per core
  memoryUsed: number; // bytes
  memoryTotal: number; // bytes
  memoryPercent: number; // 0-100
  diskReadBps: number; // bytes per second
  diskWriteBps: number; // bytes per second
  networkUploadBps: number; // bytes per second
  networkDownloadBps: number; // bytes per second
}

/**
 * Performance history data structure for charting
 * Maintains a 60-point rolling buffer (60 seconds)
 */
export interface PerformanceHistory {
  cpu: Array<{ timestamp: number; value: number }>;
  memory: Array<{ timestamp: number; value: number }>;
  maxPoints: number; // 60 for 60-second window
}
