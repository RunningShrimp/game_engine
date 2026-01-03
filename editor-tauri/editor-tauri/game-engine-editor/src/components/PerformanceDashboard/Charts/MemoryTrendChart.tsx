/**
 * Memory Trend Chart Component
 * Shows memory usage over time with leak detection
 */

import React, { useMemo } from 'react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import type { PerformanceMetrics } from '../../../types/performance';

interface MemoryTrendChartProps {
  history: PerformanceMetrics[];
}

export const MemoryTrendChart: React.FC<MemoryTrendChartProps> = ({ history }) => {
  const chartData = useMemo(() => {
    return history
      .slice(-60)
      .map((metrics) => ({
        time: new Date(metrics.timestamp).toLocaleTimeString(),
        memory: metrics.memoryUsed / (1024 * 1024 * 1024), // Convert to GB
        timestamp: metrics.timestamp,
      }));
  }, [history]);

  const stats = useMemo(() => {
    if (chartData.length === 0) return { min: 0, max: 0, avg: 0 };
    const memValues = chartData.map((d) => d.memory);
    return {
      min: Math.min(...memValues),
      max: Math.max(...memValues),
      avg: memValues.reduce((a, b) => a + b, 0) / memValues.length,
    };
  }, [chartData]);

  // Detect potential memory leak
  const memoryLeakDetected = useMemo(() => {
    if (chartData.length < 10) return false;
    const firstHalf = chartData.slice(0, Math.floor(chartData.length / 2));
    const secondHalf = chartData.slice(Math.floor(chartData.length / 2));

    const avgFirst = firstHalf.reduce((sum, d) => sum + d.memory, 0) / firstHalf.length;
    const avgSecond = secondHalf.reduce((sum, d) => sum + d.memory, 0) / secondHalf.length;

    // If second half average is 20% higher than first half, potential leak
    return avgSecond > avgFirst * 1.2;
  }, [chartData]);

  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-3 shadow-lg">
          <p className="text-slate-400 text-sm">{data.time}</p>
          <p className="text-2xl font-bold text-slate-200">{data.memory.toFixed(2)} GB</p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="w-full h-full">
      {/* Header with stats */}
      <div className="flex items-center justify-between mb-4 px-2">
        <div className="flex gap-4 text-sm">
          <div>
            <span className="text-slate-400">Min: </span>
            <span className="text-slate-200 font-semibold">{stats.min.toFixed(2)} GB</span>
          </div>
          <div>
            <span className="text-slate-400">Avg: </span>
            <span className="text-slate-200 font-semibold">{stats.avg.toFixed(2)} GB</span>
          </div>
          <div>
            <span className="text-slate-400">Max: </span>
            <span className="text-slate-200 font-semibold">{stats.max.toFixed(2)} GB</span>
          </div>
        </div>
        {memoryLeakDetected && (
          <div className="flex items-center gap-2 text-xs text-red-400 bg-red-400/10 px-2 py-1 rounded">
            <span className="animate-pulse">⚠️</span>
            <span>Possible Memory Leak</span>
          </div>
        )}
      </div>

      {/* Chart */}
      <ResponsiveContainer width="100%" height="85%">
        <AreaChart data={chartData} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
          <defs>
            <linearGradient id="memoryGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.3}/>
              <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0}/>
            </linearGradient>
          </defs>
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
            domain={[0, 'dataMax + 1']}
          />
          <Tooltip content={<CustomTooltip />} />
          <ReferenceLine
            y={stats.avg}
            stroke="#8b5cf6"
            strokeDasharray="5 5"
            label={{ value: 'Avg', fill: '#8b5cf6', fontSize: 10 }}
          />
          <Area
            type="monotone"
            dataKey="memory"
            stroke="#8b5cf6"
            strokeWidth={2}
            fill="url(#memoryGradient)"
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};
