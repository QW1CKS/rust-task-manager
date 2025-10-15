/**
 * Performance Monitoring Service
 * 
 * Manages polling of performance metrics with caching and error handling.
 * Polls every 1-2 seconds with 100ms cache to prevent excessive refreshes.
 */

import { invokeGetPerformanceData } from './tauri';
import type { PerformanceMetrics } from '../types/performance';

const POLL_INTERVAL_MS = 1500; // 1.5 seconds
const CACHE_DURATION_MS = 100; // 100ms cache

interface CachedMetrics {
    data: PerformanceMetrics;
    timestamp: number;
}

let cachedMetrics: CachedMetrics | null = null;
let pollingInterval: number | null = null;
let subscribers: Array<(metrics: PerformanceMetrics | null, error?: Error) => void> = [];

/**
 * Fetch performance metrics with caching
 * 
 * Returns cached data if less than CACHE_DURATION_MS old,
 * otherwise fetches fresh data from backend.
 * 
 * @returns Promise with performance metrics
 */
export async function getPerformanceMetrics(): Promise<PerformanceMetrics> {
    const now = Date.now();

    // Return cached data if still valid
    if (cachedMetrics && (now - cachedMetrics.timestamp) < CACHE_DURATION_MS) {
        return cachedMetrics.data;
    }

    // Fetch fresh data
    try {
        const metrics = await invokeGetPerformanceData();
        cachedMetrics = { data: metrics, timestamp: now };
        return metrics;
    } catch (error) {
        console.error('[Performance] Failed to fetch metrics:', error);
        throw error;
    }
}

/**
 * Start polling performance metrics at regular intervals
 * 
 * Calls all registered subscribers with updated metrics or errors.
 * Implements FR-021: Shows "Error" if metrics fail.
 */
export function startPerformancePolling(): void {
    if (pollingInterval !== null) {

        console.warn('[Performance] Polling already started');
        return;
    }

    // Initial fetch
    void fetchAndNotify();

    // Start polling
    pollingInterval = window.setInterval(() => {
        void fetchAndNotify();
    }, POLL_INTERVAL_MS);

}

/**
 * Stop polling performance metrics
 */
export function stopPerformancePolling(): void {
    if (pollingInterval !== null) {
        window.clearInterval(pollingInterval);
        pollingInterval = null;

    }
}

/**
 * Subscribe to performance metric updates
 * 
 * @param callback - Function called with metrics or error on each update
 * @returns Unsubscribe function
 */
export function subscribeToPerformance(
    callback: (metrics: PerformanceMetrics | null, error?: Error) => void
): () => void {
    subscribers.push(callback);

    // Send current cached data immediately if available
    if (cachedMetrics) {
        callback(cachedMetrics.data);
    }

    return () => {
        subscribers = subscribers.filter(cb => cb !== callback);
    };
}

/**
 * Fetch metrics and notify all subscribers
 */
async function fetchAndNotify(): Promise<void> {
    try {
        const metrics = await getPerformanceMetrics();
        subscribers.forEach(callback => callback(metrics));
    } catch (error) {
        // Notify subscribers of error per FR-021
        const err = error instanceof Error ? error : new Error(String(error));
        subscribers.forEach(callback => callback(null, err));
    }
}
