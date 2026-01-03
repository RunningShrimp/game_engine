import React from 'react';

/**
 * 资源浏览器的加载骨架屏
 */
export function AssetBrowserLoadingSkeleton() {
  return (
    <div className="h-full bg-slate-900 flex">
      {/* 左侧目录树骨架 */}
      <div className="w-64 bg-slate-800 border-r border-slate-700 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-slate-700 animate-pulse" />

        {/* 目录树 */}
        <div className="flex-1 p-4 space-y-2">
          {[...Array(8)].map((_, i) => (
            <div key={i} className="flex items-center gap-2">
              <div className="w-4 h-4 bg-slate-700 rounded animate-pulse" />
              <div className="h-6 bg-slate-700 rounded flex-1 animate-pulse" />
            </div>
          ))}
        </div>
      </div>

      {/* 右侧资源网格骨架 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏和筛选器 */}
        <div className="h-16 bg-slate-800 border-b border-slate-700 p-4 flex items-center gap-4">
          <div className="h-8 bg-slate-700 rounded flex-1 animate-pulse" />
          <div className="w-32 h-8 bg-slate-700 rounded animate-pulse" />
        </div>

        {/* 资源网格 */}
        <div className="flex-1 p-4 overflow-y-auto">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
            {[...Array(20)].map((_, i) => (
              <div key={i} className="space-y-2">
                {/* 资源缩略图 */}
                <div
                  className="aspect-square bg-slate-800 rounded-lg animate-pulse"
                  style={{ animationDelay: `${i * 30}ms` }}
                />

                {/* 资源名称 */}
                <div className="h-4 bg-slate-800 rounded animate-pulse" />
              </div>
            ))}
          </div>
        </div>

        {/* 底部状态栏 */}
        <div className="h-8 bg-slate-800 border-t border-slate-700 flex items-center px-4">
          <div className="h-4 bg-slate-700 rounded w-48 animate-pulse" />
        </div>
      </div>
    </div>
  );
}
