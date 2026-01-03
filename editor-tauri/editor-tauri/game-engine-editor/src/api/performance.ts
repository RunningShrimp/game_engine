/**
 * Tauri API wrapper for performance monitoring
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  PerformanceMetrics,
  PerformanceHotspot,
  PerformanceAlert,
  AlertThreshold,
  PerformanceStatistics,
} from '../types/performance';

/**
 * Get current performance metrics
 */
export async function getPerformanceMetrics(): Promise<PerformanceMetrics> {
  return await invoke<PerformanceMetrics>('get_performance_metrics');
}

/**
 * Get performance hotspots
 */
export async function getPerformanceHotspots(): Promise<PerformanceHotspot[]> {
  return await invoke<PerformanceHotspot[]>('get_performance_hotspots');
}

/**
 * Get alert history
 */
export async function getAlertHistory(): Promise<PerformanceAlert[]> {
  return await invoke<PerformanceAlert[]>('get_alert_history');
}

/**
 * Acknowledge an alert
 */
export async function acknowledgeAlert(alertId: string): Promise<void> {
  await invoke('acknowledge_alert', { alertId });
}

/**
 * Clear all alerts
 */
export async function clearAlerts(): Promise<void> {
  await invoke('clear_alerts');
}

/**
 * Set alert threshold
 */
export async function setAlertThreshold(
  alertType: string,
  threshold: number
): Promise<void> {
  await invoke('set_alert_threshold', {
    alertType,
    threshold,
  });
}

/**
 * Get alert thresholds
 */
export async function getAlertThresholds(): Promise<AlertThreshold> {
  return await invoke<any>('get_alert_thresholds');
}

/**
 * Get historical performance data
 */
export async function getPerformanceHistory(
  startTime: number,
  endTime: number
): Promise<PerformanceMetrics[]> {
  return await invoke<PerformanceMetrics[]>('get_performance_history', {
    startTime,
    endTime,
  });
}

/**
 * Get performance statistics
 */
export async function getPerformanceStatistics(
  startTime: number,
  endTime: number
): Promise<PerformanceStatistics> {
  return await invoke<any>('get_performance_statistics', {
    startTime,
    endTime,
  });
}

/**
 * Export performance data
 */
export async function exportPerformanceData(
  format: 'json' | 'csv',
  startTime: number,
  endTime: number
): Promise<string> {
  return await invoke<string>('export_performance_data', {
    format,
    startTime,
    endTime,
  });
}

/**
 * Update performance metrics (called by engine)
 */
export async function updatePerformanceMetrics(
  metrics: Partial<PerformanceMetrics>
): Promise<void> {
  await invoke('update_performance_metrics', {
    metrics: {
      drawCalls: metrics.drawCalls,
      triangles: metrics.triangles,
      vertices: metrics.vertices,
      physicsTime: metrics.physicsTime,
      rigidBodyCount: metrics.rigidBodyCount,
      collisionCount: metrics.collisionCount,
      scriptTime: metrics.scriptTime,
      scriptCount: metrics.scriptCount,
      audioTime: metrics.audioTime,
      audioSourceCount: metrics.audioSourceCount,
      networkTime: metrics.networkTime,
      networkBytesReceived: metrics.networkBytesReceived,
      networkBytesSent: metrics.networkBytesSent,
    },
  });
}

/**
 * Start monitoring
 */
export async function startMonitoring(): Promise<void> {
  await invoke('start_monitoring');
}

/**
 * Stop monitoring
 */
export async function stopMonitoring(): Promise<void> {
  await invoke('stop_monitoring');
}

/**
 * Check if monitoring is active
 */
export async function isMonitoringActive(): Promise<boolean> {
  return await invoke<boolean>('is_monitoring_active');
}
