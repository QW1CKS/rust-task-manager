/**
 * Tauri Command Wrappers
 * 
 * Type-safe wrappers around Tauri invoke commands.
 * All functions are async and handle error conversion.
 */

import { invoke } from '@tauri-apps/api/core';
import type { SystemInfo } from '../types/system';
import type { PerformanceMetrics } from '../types/performance';
import type { ProcessInfo } from '../types/process';

/**
 * Fetch static system information (called once at startup)
 * 
 * @returns Promise with system hardware/software details
 * @throws Error with message from Rust if command fails
 */
export async function invokeGetSystemInfo(): Promise<SystemInfo> {
    try {
        return await invoke<SystemInfo>('get_system_info');
    } catch (error) {

        console.error('[Tauri] get_system_info failed:', error);
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(`Failed to get system info: ${message}`);
    }
}/**
 * Fetch current performance metrics (called every 1-2 seconds)
 * 
 * @returns Promise with current CPU, memory, disk, network metrics
 * @throws Error with message from Rust if command fails
 */
export async function invokeGetPerformanceData(): Promise<PerformanceMetrics> {
    try {
        return await invoke<PerformanceMetrics>('get_performance_data');
    } catch (error) {

        console.error('[Tauri] get_performance_data failed:', error);
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(`Failed to get performance data: ${message}`);
    }
}

/**
 * Fetch list of all running processes (called every 1-2 seconds)
 * 
 * @returns Promise with array of process information
 * @throws Error with message from Rust if command fails
 */
export async function invokeGetProcesses(): Promise<ProcessInfo[]> {
    try {
        return await invoke<ProcessInfo[]>('get_processes');
    } catch (error) {
        console.error('[Tauri] get_processes failed:', error);
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(`Failed to get processes: ${message}`);
    }
}

/**
 * Terminate a process by PID
 * 
 * @param pid - Process ID to terminate
 * @returns Promise that resolves on successful termination
 * @throws Error with message from Rust explaining failure:
 *   - "Process not found" - Process doesn't exist
 *   - "Cannot terminate critical system process" - Attempting to kill critical process (FR-023)
 *   - "Administrator privileges may be required" - Permission denied (FR-022)
 *   - "Process did not terminate" - Kill signal sent but process still running
 */
export async function invokeKillProcess(pid: number): Promise<void> {
    try {
        await invoke<void>('kill_process', { pid });
    } catch (error) {
        console.error('[Tauri] kill_process failed:', error);
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(message);
    }
}
