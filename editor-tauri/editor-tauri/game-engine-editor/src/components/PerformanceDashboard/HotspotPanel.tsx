/**
 * Performance Hotspot Panel
 * Displays top performance bottlenecks
 */

import React, { useState } from 'react';
import type { PerformanceHotspot } from '../../types/performance';

interface HotspotPanelProps {
  hotspots: PerformanceHotspot[];
}

export const HotspotPanel: React.FC<HotspotPanelProps> = ({ hotspots }) => {
  const [selectedHotspot, setSelectedHotspot] = useState<PerformanceHotspot | null>(null);

  const getCategoryColor = (category: string): string => {
    switch (category) {
      case 'render':
        return 'bg-blue-500';
      case 'physics':
        return 'bg-red-500';
      case 'script':
        return 'bg-green-500';
      case 'audio':
        return 'bg-amber-500';
      case 'network':
        return 'bg-purple-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getCategoryIcon = (category: string): string => {
    switch (category) {
      case 'render':
        return '🎨';
      case 'physics':
        return '⚡';
      case 'script':
        return '📜';
      case 'audio':
        return '🔊';
      case 'network':
        return '🌐';
      default:
        return '📦';
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 py-3 border-b border-slate-700">
        <h3 className="text-sm font-semibold text-slate-200">Performance Hotspots</h3>
      </div>

      <div className="flex-1 overflow-y-auto">
        {hotspots.length === 0 ? (
          <div className="flex items-center justify-center h-full text-slate-500 text-sm">
            No hotspots detected
          </div>
        ) : (
          <div className="divide-y divide-slate-700">
            {hotspots.map((hotspot, index) => (
              <div
                key={index}
                className="px-4 py-3 hover:bg-slate-800/50 cursor-pointer transition-colors"
                onClick={() => setSelectedHotspot(hotspot)}
              >
                <div className="flex items-start gap-3">
                  {/* Rank */}
                  <div className="flex-shrink-0 w-6 h-6 rounded-full bg-slate-700 flex items-center justify-center text-xs font-semibold text-slate-300">
                    {index + 1}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-lg">
                        {getCategoryIcon(hotspot.category)}
                      </span>
                      <h4 className="text-sm font-medium text-slate-200 truncate">
                        {hotspot.name}
                      </h4>
                    </div>

                    <div className="flex items-center gap-3 text-xs text-slate-400">
                      <span className="text-slate-200 font-semibold">
                        {hotspot.duration.toFixed(2)} ms
                      </span>
                      <span>({hotspot.percentage.toFixed(1)}%)</span>
                      <span>• {hotspot.callCount.toLocaleString()} calls</span>
                    </div>

                    {/* Duration bar */}
                    <div className="mt-2 w-full bg-slate-700 rounded-full h-1.5">
                      <div
                        className={`h-1.5 rounded-full ${getCategoryColor(hotspot.category)}`}
                        style={{ width: `${Math.min(hotspot.percentage, 100)}%` }}
                      />
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Hotspot Detail Modal */}
      {selectedHotspot && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
          onClick={() => setSelectedHotspot(null)}
        >
          <div
            className="bg-slate-800 rounded-lg shadow-xl max-w-lg w-full mx-4 border border-slate-700"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700">
              <h3 className="text-lg font-semibold text-slate-200">
                {getCategoryIcon(selectedHotspot.category)} {selectedHotspot.name}
              </h3>
              <button
                onClick={() => setSelectedHotspot(null)}
                className="text-slate-400 hover:text-slate-200 transition-colors"
              >
                ✕
              </button>
            </div>

            <div className="p-4 space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="bg-slate-900/50 rounded p-3">
                  <div className="text-xs text-slate-400 mb-1">Duration</div>
                  <div className="text-2xl font-bold text-slate-200">
                    {selectedHotspot.duration.toFixed(2)} ms
                  </div>
                </div>
                <div className="bg-slate-900/50 rounded p-3">
                  <div className="text-xs text-slate-400 mb-1">Percentage</div>
                  <div className="text-2xl font-bold text-slate-200">
                    {selectedHotspot.percentage.toFixed(1)}%
                  </div>
                </div>
                <div className="bg-slate-900/50 rounded p-3">
                  <div className="text-xs text-slate-400 mb-1">Call Count</div>
                  <div className="text-2xl font-bold text-slate-200">
                    {selectedHotspot.callCount.toLocaleString()}
                  </div>
                </div>
                <div className="bg-slate-900/50 rounded p-3">
                  <div className="text-xs text-slate-400 mb-1">Category</div>
                  <div className="text-2xl font-bold text-slate-200 capitalize">
                    {selectedHotspot.category}
                  </div>
                </div>
              </div>

              {selectedHotspot.children && selectedHotspot.children.length > 0 && (
                <div>
                  <h4 className="text-sm font-semibold text-slate-300 mb-2">Breakdown</h4>
                  <div className="space-y-2">
                    {selectedHotspot.children.map((child, index) => (
                      <div
                        key={index}
                        className="flex items-center justify-between bg-slate-900/30 rounded p-2"
                      >
                        <span className="text-sm text-slate-300">{child.name}</span>
                        <span className="text-sm text-slate-400">
                          {child.duration.toFixed(2)} ms
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
