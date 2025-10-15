/**
 * System information interface matching Rust SystemInfo struct
 *
 * Contains static hardware and software configuration gathered at startup.
 */
export interface SystemInfo {
  osName: string;
  osVersion: string;
  kernelVersion: string;
  cpuModel: string;
  cpuArchitecture: string;
  cpuCores: number;
  totalMemory: number; // bytes
  hostname: string;
}
