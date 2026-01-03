import React from 'react';

/**
 * 时间轴的加载骨架屏
 */
export function TimelineLoadingSkeleton() {
  return (
    <div className="h-64 bg-slate-900 border-t border-slate-700 flex flex-col">
      {/* 工具栏骨架 */}
      <div className="h-10 bg-slate-800 border-b border-slate-700 flex items-center px-4 gap-4">
        <div className="flex gap-2">
          {[...Array(5)].map((_, i) => (
            <div
              key={i}
              className="w-8 h-8 bg-slate-700 rounded animate-pulse"
              style={{ animationDelay: `${i * 50}ms` }}
            />
          ))}
        </div>
      </div>

      {/* 时间刻度骨架 */}
      <div className="h-6 bg-slate-800 border-b border-slate-700 flex items-center px-4">
        <div className="flex gap-8 w-full">
          {[...Array(12)].map((_, i) => (
            <div
              key={i}
              className="h-3 bg-slate-700 rounded w-16 animate-pulse"
              style={{ animationDelay: `${i * 30}ms` }}
            />
          ))}
        </div>
      </div>

      {/* 轨道区域骨架 */}
      <div className="flex-1 overflow-y-auto">
        <div className="p-2 space-y-2">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="flex items-center gap-2">
              {/* 轨道标签 */}
              <div className="w-32 h-8 bg-slate-800 rounded animate-pulse" />

              {/* 轨道内容 */}
              <div className="flex-1 h-8 bg-slate-800 rounded relative overflow-hidden">
                <div
                  className="absolute left-0 top-0 bottom-0 bg-blue-600/30 rounded animate-pulse"
                  style={{ width: '40%', animationDelay: `${i * 100}ms` }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
