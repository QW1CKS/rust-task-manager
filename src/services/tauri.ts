/**
 * Tauri Command Wrappers
 * 
 * Type-safe wrappers around Tauri invoke commands.
 * All functions are async and handle error conversion.
 */

import { invoke } from '@tauri-apps/api/core';
import type { SystemInfo } from '../types/system';
import type { PerformanceMetrics } from '../types/performance';

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
