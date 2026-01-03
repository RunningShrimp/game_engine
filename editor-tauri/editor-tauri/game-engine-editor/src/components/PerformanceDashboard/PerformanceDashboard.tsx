/**
 * Performance Dashboard
 * Main container for real-time performance monitoring
 */

import React, { useState, useEffect, useCallback } from 'react';
import { MetricsPanel } from './MetricsPanel';
import { FPSChart } from './Charts/FPSChart';
import { UsagePieChart } from './Charts/UsagePieChart';
import { MemoryTrendChart } from './Charts/MemoryTrendChart';
import { HotspotPanel } from './HotspotPanel';
import { AlertSystem } from './AlertSystem';
import type {
  PerformanceMetrics,
  PerformanceHotspot,
  PerformanceAlert,
  DashboardView,
} from '../../types/performance';
import {
  getPerformanceMetrics,
  getPerformanceHotspots,
  getAlertHistory,
  startMonitoring,
  stopMonitoring,
} from '../../api/performance';

interface PerformanceDashboardProps {
  onClose?: () => void;
}

export const PerformanceDashboard: React.FC<PerformanceDashboardProps> = ({ onClose }) => {
  const [view, setView] = useState<DashboardView>('realtime');
  const [metrics, setMetrics] = useState<PerformanceMetrics>({
    fps: 60,
    frameTime: 16.67,
    cpuUsage: 0,
    cpuUsagePerCore: [],
    gpuUsage: 0,
    gpuMemory: 0,
    gpuMemoryTotal: 0,
    memoryUsed: 0,
    memoryTotal: 0,
    memoryUsedBySystem: 0,
    drawCalls: 0,
    triangles: 0,
    vertices: 0,
    physicsTime: 0,
    rigidBodyCount: 0,
    collisionCount: 0,
    scriptTime: 0,
    scriptCount: 0,
    audioTime: 0,
    audioSourceCount: 0,
    networkTime: 0,
    networkBytesReceived: 0,
    networkBytesSent: 0,
    timestamp: Date.now(),
  });
  const [history, setHistory] = useState<PerformanceMetrics[]>([]);
  const [hotspots, setHotspots] = useState<PerformanceHotspot[]>([]);
  const [alerts, setAlerts] = useState<PerformanceAlert[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [updateInterval, setUpdateInterval] = useState(250); // ms

  // Start monitoring on mount
  useEffect(() => {
    const initMonitoring = async () => {
      try {
        await startMonitoring();
        setIsMonitoring(true);
      } catch (error) {
        console.error('Failed to start monitoring:', error);
      }
    };

    initMonitoring();

    return () => {
      stopMonitoring().catch(console.error);
    };
  }, []);

  // Fetch metrics periodically
  useEffect(() => {
    if (!isMonitoring) return;

    const fetchMetrics = async () => {
      try {
        const [newMetrics, newHotspots, newAlerts] = await Promise.all([
          getPerformanceMetrics(),
          getPerformanceHotspots(),
          getAlertHistory(),
        ]);

        setMetrics(newMetrics);
        setHotspots(newHotspots);
        setAlerts(newAlerts);

        // Add to history
        setHistory((prev) => {
          const newHistory = [...prev, newMetrics];
          // Keep only last 60 seconds worth of data for real-time view
          if (newHistory.length > 240) {
            // 60 seconds * 4 (assuming 250ms interval)
            return newHistory.slice(-240);
          }
          return newHistory;
        });
      } catch (error) {
        console.error('Failed to fetch performance data:', error);
      }
    };

    fetchMetrics();
    const interval = setInterval(fetchMetrics, updateInterval);

    return () => clearInterval(interval);
  }, [isMonitoring, updateInterval]);

  const handleAlertUpdate = useCallback(() => {
    getAlertHistory().then(setAlerts).catch(console.error);
  }, []);

  return (
    <div className="fixed inset-0 bg-slate-900/95 backdrop-blur-sm z-50 flex">
      {/* Main Panel */}
      <div className="flex-1 flex flex-col bg-slate-900 border border-slate-700 m-4 rounded-lg overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
          <div className="flex items-center gap-4">
            <h1 className="text-xl font-bold text-slate-200 flex items-center gap-2">
              <span>📊</span>
              <span>Performance Monitor</span>
            </h1>
            <div className="flex items-center gap-1 text-xs">
              <span
                className={`w-2 h-2 rounded-full ${
                  isMonitoring ? 'bg-green-500 animate-pulse' : 'bg-red-500'
                }`}
              />
              <span className="text-slate-400">
                {isMonitoring ? 'Live' : 'Stopped'}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-4">
            {/* View Tabs */}
            <div className="flex bg-slate-800 rounded-lg p-1">
              {(
                ['realtime', 'history', 'alerts'] as DashboardView[]
              ).map((v) => (
                <button
                  key={v}
                  onClick={() => setView(v)}
                  className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                    view === v
                      ? 'bg-blue-500 text-white'
                      : 'text-slate-400 hover:text-slate-200'
                  }`}
                >
                  {v.charAt(0).toUpperCase() + v.slice(1)}
                </button>
              ))}
            </div>

            {/* Update Interval */}
            {view === 'realtime' && (
              <select
                value={updateInterval}
                onChange={(e) => setUpdateInterval(Number(e.target.value))}
                className="bg-slate-800 text-slate-200 text-sm rounded px-3 py-1.5 border border-slate-700"
              >
                <option value={100}>100ms</option>
                <option value={250}>250ms</option>
                <option value={500}>500ms</option>
                <option value={1000}>1s</option>
              </select>
            )}

            {/* Close Button */}
            {onClose && (
              <button
                onClick={onClose}
                className="text-slate-400 hover:text-white text-2xl transition-colors"
              >
                ×
              </button>
            )}
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 flex overflow-hidden">
          {/* Left Panel - Metrics */}
          <div className="w-72 border-r border-slate-700 overflow-y-auto">
            <MetricsPanel metrics={metrics} />
          </div>

          {/* Center Panel - Charts */}
          <div className="flex-1 flex flex-col overflow-y-auto">
            {view === 'realtime' && (
              <div className="flex-1 p-4 space-y-4">
                {/* FPS Chart */}
                <div className="bg-slate-800 rounded-lg p-4 border border-slate-700 h-64">
                  <h3 className="text-sm font-semibold text-slate-200 mb-2">FPS Over Time</h3>
                  <FPSChart history={history} targetFps={60} />
                </div>

                {/* Charts Grid */}
                <div className="grid grid-cols-2 gap-4 h-64">
                  {/* CPU/GPU Usage */}
                  <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
                    <h3 className="text-sm font-semibold text-slate-200 mb-2">
                      System Usage
                    </h3>
                    <UsagePieChart
                      render={metrics.drawCalls > 0 ? 30 : 0}
                      physics={metrics.physicsTime > 0 ? 25 : 0}
                      script={metrics.scriptTime > 0 ? 20 : 0}
                      audio={metrics.audioTime > 0 ? 10 : 0}
                      network={metrics.networkTime > 0 ? 5 : 0}
                      other={10}
                    />
                  </div>

                  {/* Memory Trend */}
                  <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
                    <h3 className="text-sm font-semibold text-slate-200 mb-2">
                      Memory Trend
                    </h3>
                    <MemoryTrendChart history={history} />
                  </div>
                </div>
              </div>
            )}

            {view === 'history' && (
              <div className="flex-1 p-4">
                <div className="bg-slate-800 rounded-lg p-6 border border-slate-700 h-full">
                  <h3 className="text-lg font-semibold text-slate-200 mb-4">
                    Historical Data
                  </h3>
                  <p className="text-slate-400">
                    Historical data view will be implemented with date range picker
                    and detailed statistics.
                  </p>
                </div>
              </div>
            )}

            {view === 'alerts' && (
              <div className="flex-1 overflow-hidden">
                <AlertSystem alerts={alerts} onAlertUpdate={handleAlertUpdate} />
              </div>
            )}
          </div>

          {/* Right Panel - Hotspots */}
          <div className="w-80 border-l border-slate-700 overflow-hidden">
            <HotspotPanel hotspots={hotspots} />
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-2 border-t border-slate-700 flex items-center justify-between text-xs text-slate-400">
          <div className="flex items-center gap-4">
            <span>Memory: {(metrics.memoryUsed / 1024 / 1024 / 1024).toFixed(2)} GB</span>
            <span>•</span>
            <span>Draw Calls: {metrics.drawCalls.toLocaleString()}</span>
            <span>•</span>
            <span>Triangles: {metrics.triangles.toLocaleString()}</span>
          </div>
          <div>
            Update Rate: {updateInterval}ms
          </div>
        </div>
      </div>
    </div>
  );
};
