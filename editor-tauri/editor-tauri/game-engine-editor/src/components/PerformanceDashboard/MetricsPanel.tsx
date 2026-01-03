/**
 * Real-time Metrics Panel
 * Displays current performance metrics with color-coded indicators
 */

import React from 'react';
import type { PerformanceMetrics } from '../../types/performance';

interface MetricsPanelProps {
  metrics: PerformanceMetrics;
}

export const MetricsPanel: React.FC<MetricsPanelProps> = ({ metrics }) => {
  // Helper function to get FPS color
  const getFpsColor = (fps: number): string => {
    if (fps >= 55) return 'text-green-400';
    if (fps >= 30) return 'text-yellow-400';
    return 'text-red-400';
  };

  // Helper function to get usage color
  const getUsageColor = (usage: number): string => {
    if (usage < 70) return 'text-green-400';
    if (usage < 85) return 'text-yellow-400';
    return 'text-red-400';
  };

  // Helper function to format memory
  const formatMemory = (bytes: number): string => {
    const gb = bytes / (1024 * 1024 * 1024);
    const mb = bytes / (1024 * 1024);
    if (gb >= 1) {
      return `${gb.toFixed(1)} GB`;
    }
    return `${mb.toFixed(0)} MB`;
  };

  // Helper function to format percentage
  const formatPercentage = (value: number, total: number): string => {
    if (total === 0) return '0%';
    return `${((value / total) * 100).toFixed(1)}%`;
  };

  return (
    <div className="flex flex-col gap-4 p-4">
      {/* FPS Display */}
      <div className={`text-6xl font-bold ${getFpsColor(metrics.fps)}`}>
        {metrics.fps.toFixed(1)}
      </div>
      <div className="text-sm text-slate-400">FPS</div>

      {/* Frame Time */}
      <div className="flex items-center gap-2">
        <div className="text-3xl font-semibold text-slate-200">
          {metrics.frameTime.toFixed(1)}
        </div>
        <div className="text-sm text-slate-400">ms/frame</div>
      </div>

      {/* CPU Usage */}
      <div className="bg-slate-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-slate-400">CPU</span>
          <span className={`text-lg font-semibold ${getUsageColor(metrics.cpuUsage)}`}>
            {metrics.cpuUsage.toFixed(1)}%
          </span>
        </div>
        <div className="w-full bg-slate-700 rounded-full h-2">
          <div
            className={`h-2 rounded-full transition-all ${
              metrics.cpuUsage < 70
                ? 'bg-green-500'
                : metrics.cpuUsage < 85
                ? 'bg-yellow-500'
                : 'bg-red-500'
            }`}
            style={{ width: `${Math.min(metrics.cpuUsage, 100)}%` }}
          />
        </div>
      </div>

      {/* GPU Usage */}
      <div className="bg-slate-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-slate-400">GPU</span>
          <span className={`text-lg font-semibold ${getUsageColor(metrics.gpuUsage)}`}>
            {metrics.gpuUsage.toFixed(1)}%
          </span>
        </div>
        <div className="w-full bg-slate-700 rounded-full h-2">
          <div
            className={`h-2 rounded-full transition-all ${
              metrics.gpuUsage < 70
                ? 'bg-green-500'
                : metrics.gpuUsage < 85
                ? 'bg-yellow-500'
                : 'bg-red-500'
            }`}
            style={{ width: `${Math.min(metrics.gpuUsage, 100)}%` }}
          />
        </div>
      </div>

      {/* Memory Usage */}
      <div className="bg-slate-800 rounded-lg p-3">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-slate-400">Memory</span>
          <span className={`text-lg font-semibold ${getUsageColor(
            (metrics.memoryUsed / metrics.memoryTotal) * 100
          )}`}>
            {formatMemory(metrics.memoryUsed)}
          </span>
        </div>
        <div className="w-full bg-slate-700 rounded-full h-2">
          <div
            className={`h-2 rounded-full transition-all ${
              (metrics.memoryUsed / metrics.memoryTotal) * 100 < 70
                ? 'bg-green-500'
                : (metrics.memoryUsed / metrics.memoryTotal) * 100 < 85
                ? 'bg-yellow-500'
                : 'bg-red-500'
            }`}
            style={{ width: `${Math.min((metrics.memoryUsed / metrics.memoryTotal) * 100, 100)}%` }}
          />
        </div>
        <div className="text-xs text-slate-500 mt-1">
          {formatPercentage(metrics.memoryUsed, metrics.memoryTotal)} of {formatMemory(metrics.memoryTotal)}
        </div>
      </div>

      {/* Rendering Stats */}
      <div className="grid grid-cols-2 gap-2 mt-2">
        <div className="bg-slate-800 rounded p-2">
          <div className="text-xs text-slate-400">Draw Calls</div>
          <div className="text-lg font-semibold text-slate-200">
            {metrics.drawCalls.toLocaleString()}
          </div>
        </div>
        <div className="bg-slate-800 rounded p-2">
          <div className="text-xs text-slate-400">Triangles</div>
          <div className="text-lg font-semibold text-slate-200">
            {metrics.triangles.toLocaleString()}
          </div>
        </div>
      </div>

      {/* Physics Stats */}
      <div className="bg-slate-800 rounded p-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-slate-400">Physics</span>
          <span className="text-sm text-slate-200">{metrics.physicsTime.toFixed(2)} ms</span>
        </div>
        <div className="flex items-center justify-between mt-1">
          <span className="text-xs text-slate-500">Bodies: {metrics.rigidBodyCount}</span>
          <span className="text-xs text-slate-500">Collisions: {metrics.collisionCount}</span>
        </div>
      </div>

      {/* Script Stats */}
      <div className="bg-slate-800 rounded p-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-slate-400">Scripts</span>
          <span className="text-sm text-slate-200">{metrics.scriptTime.toFixed(2)} ms</span>
        </div>
        <div className="text-xs text-slate-500 mt-1">
          Active: {metrics.scriptCount}
        </div>
      </div>
    </div>
  );
};
