import React from 'react';

/**
 * 材质编辑器的加载骨架屏
 */
export function MaterialEditorLoadingSkeleton() {
  return (
    <div className="h-full bg-slate-900 flex">
      {/* 左侧节点画布区域 */}
      <div className="flex-1 flex flex-col border-r border-slate-700">
        {/* 工具栏骨架 */}
        <div className="h-12 bg-slate-800 border-b border-slate-700 animate-pulse" />

        {/* 节点画布骨架 */}
        <div className="flex-1 relative bg-slate-900">
          <div className="absolute inset-0 grid grid-cols-4 gap-4 p-8">
            {[...Array(8)].map((_, i) => (
              <div
                key={i}
                className="bg-slate-800 rounded-lg h-32 animate-pulse"
                style={{ animationDelay: `${i * 100}ms` }}
              />
            ))}
          </div>
        </div>
      </div>

      {/* 右侧属性面板骨架 */}
      <div className="w-80 bg-slate-800 flex flex-col">
        {/* 面板头部 */}
        <div className="h-12 border-b border-slate-700 animate-pulse" />

        {/* 属性列表 */}
        <div className="flex-1 p-4 space-y-3">
          {[...Array(6)].map((_, i) => (
            <div key={i} className="space-y-2">
              <div className="h-4 bg-slate-700 rounded animate-pulse" />
              <div className="h-8 bg-slate-700 rounded animate-pulse" />
            </div>
          ))}
        </div>

        {/* 底部预览骨架 */}
        <div className="h-48 border-t border-slate-700 animate-pulse" />
      </div>
    </div>
  );
}
