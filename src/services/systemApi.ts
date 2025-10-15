import { invoke } from '@tauri-apps/api/core';
import type { SystemInfo } from '../types/system';

/**
 * Fetch system information from the backend
 * @returns Promise resolving to SystemInfo object
 * @throws Error if the backend command fails
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  try {
    const info = await invoke<SystemInfo>('get_system_info');
    return info;
  } catch (error) {
    console.error('Failed to fetch system information:', error);
    const message = error instanceof Error ? error.message : String(error);
    throw new Error('Failed to fetch system information: ' + message);
  }
}
