/**
 * FPS Chart Component
 * Real-time line chart showing FPS over time
 */

import React, { useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import type { PerformanceMetrics } from '../../../types/performance';

interface FPSChartProps {
  history: PerformanceMetrics[];
  targetFps?: number;
}

export const FPSChart: React.FC<FPSChartProps> = ({ history, targetFps = 60 }) => {
  // Transform data for chart
  const chartData = useMemo(() => {
    return history
      .slice(-60) // Last 60 data points
      .map((metrics, index) => ({
        time: new Date(metrics.timestamp).toLocaleTimeString(),
        fps: metrics.fps,
        timestamp: metrics.timestamp,
      }));
  }, [history]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (chartData.length === 0) return { min: 0, max: 0, avg: 0 };
    const fpsValues = chartData.map((d) => d.fps);
    return {
      min: Math.min(...fpsValues),
      max: Math.max(...fpsValues),
      avg: fpsValues.reduce((a, b) => a + b, 0) / fpsValues.length,
    };
  }, [chartData]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-3 shadow-lg">
          <p className="text-slate-400 text-sm">{data.time}</p>
          <p className="text-2xl font-bold text-slate-200">{data.fps.toFixed(1)} FPS</p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="w-full h-full">
      {/* Statistics */}
      <div className="flex items-center justify-between mb-4 px-2">
        <div className="flex gap-4 text-sm">
          <div>
            <span className="text-slate-400">Min: </span>
            <span className="text-slate-200 font-semibold">{stats.min.toFixed(1)}</span>
          </div>
          <div>
            <span className="text-slate-400">Avg: </span>
            <span className="text-slate-200 font-semibold">{stats.avg.toFixed(1)}</span>
          </div>
          <div>
            <span className="text-slate-400">Max: </span>
            <span className="text-slate-200 font-semibold">{stats.max.toFixed(1)}</span>
          </div>
        </div>
        <div className="text-sm text-slate-400">
          Target: {targetFps} FPS
        </div>
      </div>

      {/* Chart */}
      <ResponsiveContainer width="100%" height="90%">
        <LineChart data={chartData} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
          <XAxis
            dataKey="time"
            stroke="#94a3b8"
            fontSize={12}
            tickLine={false}
            axisLine={false}
          />
          <YAxis
            stroke="#94a3b8"
            fontSize={12}
            tickLine={false}
            axisLine={false}
            domain={[0, 'dataMax + 10']}
          />
          <Tooltip content={<CustomTooltip />} />
          <ReferenceLine
            y={targetFps}
            stroke="#22c55e"
            strokeDasharray="5 5"
            label={{ value: 'Target', fill: '#22c55e', fontSize: 12 }}
          />
          <Line
            type="monotone"
            dataKey="fps"
            stroke="#3b82f6"
            strokeWidth={2}
            dot={false}
            isAnimationActive={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};
