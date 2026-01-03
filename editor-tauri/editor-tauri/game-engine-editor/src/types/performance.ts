/**
 * Performance Monitoring Types
 * Types for real-time performance metrics, alerts, and analysis
 */

/**
 * Core performance metrics collected from the engine
 */
export interface PerformanceMetrics {
  // Frame metrics
  fps: number;
  frameTime: number;

  // CPU metrics
  cpuUsage: number;
  cpuUsagePerCore: number[];

  // GPU metrics
  gpuUsage: number;
  gpuMemory: number;
  gpuMemoryTotal: number;

  // Memory metrics
  memoryUsed: number;
  memoryTotal: number;
  memoryUsedBySystem: number;

  // Rendering metrics
  drawCalls: number;
  triangles: number;
  vertices: number;

  // Physics metrics
  physicsTime: number;
  rigidBodyCount: number;
  collisionCount: number;

  // Script metrics
  scriptTime: number;
  scriptCount: number;

  // Audio metrics
  audioTime: number;
  audioSourceCount: number;

  // Network metrics
  networkTime: number;
  networkBytesReceived: number;
  networkBytesSent: number;

  // Timestamp
  timestamp: number;
}

/**
 * Performance hotspot representing a function or system
 */
export interface PerformanceHotspot {
  name: string;
  duration: number;
  percentage: number;
  callCount: number;
  category: 'render' | 'physics' | 'script' | 'audio' | 'network' | 'other';
  children?: PerformanceHotspot[];
}

/**
 * Performance alert
 */
export interface PerformanceAlert {
  id: string;
  timestamp: number;
  type: 'fps' | 'memory' | 'gpu' | 'frame_time' | 'cpu' | 'leak';
  severity: 'info' | 'warning' | 'critical';
  message: string;
  value: number;
  threshold: number;
  acknowledged: boolean;
}

/**
 * Alert threshold configuration
 */
export interface AlertThreshold {
  fps: {
    warning: number;
    critical: number;
  };
  memory: {
    warning: number;
    critical: number;
  };
  gpu: {
    warning: number;
    critical: number;
  };
  frameTime: {
    warning: number;
    critical: number;
  };
  cpu: {
    warning: number;
    critical: number;
  };
}

/**
 * Historical performance data point
 */
export interface PerformanceHistoryPoint {
  timestamp: number;
  metrics: PerformanceMetrics;
}

/**
 * Performance statistics over a time period
 */
export interface PerformanceStatistics {
  avgFps: number;
  minFps: number;
  maxFps: number;
  avgFrameTime: number;
  avgCpuUsage: number;
  avgGpuUsage: number;
  avgMemoryUsage: number;
  peakMemoryUsage: number;
  totalFrames: number;
  timeRange: {
    start: number;
    end: number;
  };
}

/**
 * Chart data point
 */
export interface ChartDataPoint {
  time: string;
  value: number;
  timestamp: number;
}

/**
 * System usage breakdown
 */
export interface SystemUsageBreakdown {
  render: number;
  physics: number;
  script: number;
  audio: number;
  network: number;
  other: number;
}

/**
 * Export format options
 */
export type ExportFormat = 'json' | 'csv';

/**
 * Export configuration
 */
export interface ExportConfig {
  format: ExportFormat;
  startTime?: number;
  endTime?: number;
  includeHotspots?: boolean;
  includeAlerts?: boolean;
}

/**
 * Memory leak detection result
 */
export interface MemoryLeakDetection {
  hasLeak: boolean;
  leakRate: number; // bytes per second
  confidence: number; // 0-1
  suspectedSources: string[];
}

/**
 * Performance comparison data
 */
export interface PerformanceComparison {
  current: PerformanceMetrics;
  baseline: PerformanceMetrics;
  diff: {
    fps: number;
    frameTime: number;
    cpuUsage: number;
    gpuUsage: number;
    memoryUsage: number;
  };
  improvement: number; // percentage
}

/**
 * View mode for the dashboard
 */
export type DashboardView = 'realtime' | 'history' | 'comparison' | 'alerts';

/**
 * Time range for historical data
 */
export type TimeRange = '1h' | '6h' | '24h' | '7d' | 'custom';

/**
 * Chart update interval
 */
export type UpdateInterval = 100 | 250 | 500 | 1000;
