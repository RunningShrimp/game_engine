import React from 'react';

/**
 * 行为树编辑器的加载骨架屏
 */
export function BehaviorEditorLoadingSkeleton() {
  return (
    <div className="h-full bg-slate-900 flex">
      {/* 左侧节点面板骨架 */}
      <div className="w-64 bg-slate-800 border-r border-slate-700 flex flex-col">
        {/* 搜索框骨架 */}
        <div className="p-4 border-b border-slate-700">
          <div className="h-10 bg-slate-700 rounded animate-pulse" />
        </div>

        {/* 节点列表骨架 */}
        <div className="flex-1 p-4 space-y-2">
          {[...Array(10)].map((_, i) => (
            <div
              key={i}
              className="h-8 bg-slate-700 rounded animate-pulse"
              style={{ animationDelay: `${i * 50}ms` }}
            />
          ))}
        </div>

        {/* 黑板编辑器骨架 */}
        <div className="border-t border-slate-700 p-4">
          <div className="h-32 bg-slate-700 rounded animate-pulse" />
        </div>
      </div>

      {/* 中间节点画布骨架 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏骨架 */}
        <div className="h-12 bg-slate-800 border-b border-slate-700 animate-pulse" />

        {/* 节点画布 */}
        <div className="flex-1 relative bg-slate-900">
          <div className="absolute inset-0 grid grid-cols-5 gap-4 p-8">
            {[...Array(15)].map((_, i) => (
              <div
                key={i}
                className="bg-slate-800 rounded-lg h-24 animate-pulse"
                style={{ animationDelay: `${i * 75}ms` }}
              />
            ))}
          </div>
        </div>
      </div>

      {/* 右侧属性面板骨架 */}
      <div className="w-80 bg-slate-800 border-l border-slate-700 flex flex-col">
        <div className="h-12 border-b border-slate-700 animate-pulse" />
        <div className="flex-1 p-4 space-y-3">
          {[...Array(8)].map((_, i) => (
            <div key={i} className="space-y-2">
              <div className="h-4 bg-slate-700 rounded animate-pulse" />
              <div className="h-8 bg-slate-700 rounded animate-pulse" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
