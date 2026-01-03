/**
 * Usage Pie Chart Component
 * Shows CPU/GPU usage breakdown by system
 */

import React from 'react';
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from 'recharts';

interface UsagePieChartProps {
  render: number;
  physics: number;
  script: number;
  audio: number;
  network: number;
  other: number;
}

const COLORS = {
  render: '#3b82f6',   // blue
  physics: '#ef4444',  // red
  script: '#22c55e',   // green
  audio: '#f59e0b',    // amber
  network: '#8b5cf6',  // purple
  other: '#6b7280',    // gray
};

export const UsagePieChart: React.FC<UsagePieChartProps> = ({
  render,
  physics,
  script,
  audio,
  network,
  other,
}) => {
  const data = [
    { name: 'Render', value: render, color: COLORS.render },
    { name: 'Physics', value: physics, color: COLORS.physics },
    { name: 'Script', value: script, color: COLORS.script },
    { name: 'Audio', value: audio, color: COLORS.audio },
    { name: 'Network', value: network, color: COLORS.network },
    { name: 'Other', value: other, color: COLORS.other },
  ].filter((item) => item.value > 0);

  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0];
      return (
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-3 shadow-lg">
          <p className="text-slate-200 font-semibold">{data.name}</p>
          <p className="text-slate-400">{data.value.toFixed(1)}%</p>
        </div>
      );
    }
    return null;
  };

  const CustomLegend = ({ payload }: any) => {
    return (
      <div className="flex flex-wrap gap-2 justify-center mt-4">
        {payload.map((entry: any, index: number) => (
          <div key={index} className="flex items-center gap-1">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-xs text-slate-400">{entry.value}</span>
          </div>
        ))}
      </div>
    );
  };

  return (
    <div className="w-full h-full">
      <ResponsiveContainer width="100%" height="85%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={40}
            outerRadius={60}
            paddingAngle={2}
            dataKey="value"
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          <Tooltip content={<CustomTooltip />} />
          <Legend content={<CustomLegend />} />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};
