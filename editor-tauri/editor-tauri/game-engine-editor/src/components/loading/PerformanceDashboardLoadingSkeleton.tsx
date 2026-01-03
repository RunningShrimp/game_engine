import React from 'react';

/**
 * 性能仪表板的加载骨架屏
 */
export function PerformanceDashboardLoadingSkeleton() {
  return (
    <div className="fixed inset-0 bg-slate-900/95 flex items-center justify-center z-50">
      <div className="w-full max-w-6xl h-4/5 flex flex-col">
        {/* 头部骨架 */}
        <div className="h-16 border-b border-slate-700 flex items-center justify-between px-6">
          <div className="h-8 bg-slate-700 rounded w-64 animate-pulse" />
          <div className="w-8 h-8 bg-slate-700 rounded-full animate-pulse" />
        </div>

        {/* 内容区域 */}
        <div className="flex-1 p-6 overflow-y-auto">
          {/* 统计卡片骨架 */}
          <div className="grid grid-cols-4 gap-4 mb-6">
            {[...Array(4)].map((_, i) => (
              <div
                key={i}
                className="bg-slate-800 rounded-lg p-4 space-y-2 animate-pulse"
                style={{ animationDelay: `${i * 50}ms` }}
              >
                <div className="h-4 bg-slate-700 rounded w-3/4" />
                <div className="h-8 bg-slate-700 rounded w-1/2" />
              </div>
            ))}
          </div>

          {/* 图表区域骨架 */}
          <div className="grid grid-cols-2 gap-6">
            {/* FPS图表 */}
            <div className="bg-slate-800 rounded-lg p-6 space-y-4">
              <div className="h-6 bg-slate-700 rounded w-1/3 animate-pulse" />
              <div className="h-48 bg-slate-700 rounded animate-pulse" />
            </div>

            {/* 内存使用图表 */}
            <div className="bg-slate-800 rounded-lg p-6 space-y-4">
              <div className="h-6 bg-slate-700 rounded w-1/3 animate-pulse" />
              <div className="h-48 bg-slate-700 rounded animate-pulse" />
            </div>

            {/* GPU使用图表 */}
            <div className="bg-slate-800 rounded-lg p-6 space-y-4">
              <div className="h-6 bg-slate-700 rounded w-1/3 animate-pulse" />
              <div className="h-48 bg-slate-700 rounded animate-pulse" />
            </div>

            {/* 热点分析 */}
            <div className="bg-slate-800 rounded-lg p-6 space-y-4">
              <div className="h-6 bg-slate-700 rounded w-1/3 animate-pulse" />
              <div className="space-y-2">
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="flex items-center gap-2">
                    <div className="w-4 h-4 bg-slate-700 rounded animate-pulse" />
                    <div className="h-4 bg-slate-700 rounded flex-1 animate-pulse" />
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
