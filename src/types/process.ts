/**
 * Process status type matching Rust ProcessStatus enum
 */
export type ProcessStatus = 'running' | 'sleeping' | 'stopped' | 'other';

/**
 * Process information interface matching Rust ProcessInfo struct
 *
 * Represents a running process or service.
 * Dynamic - collected every refresh cycle.
 */
export interface ProcessInfo {
  pid: number;
  name: string;
  exePath?: string; // Optional: access may be denied
  cmdArgs?: string[]; // Optional: access may be denied
  cpuPercent: number; // 0-100
  memoryBytes: number;
  status: ProcessStatus;
  parentPid: number; // 0 if no parent
  startTime: number; // Unix epoch seconds
  user?: string; // Optional: access may be denied
}

/**
 * Process list state for managing the process table
 */
export interface ProcessListState {
  processes: ProcessInfo[];
  sortColumn: keyof ProcessInfo;
  sortOrder: 'asc' | 'desc';
  filterQuery: string;
}
