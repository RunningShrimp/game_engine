// Asset Store Loading Skeleton
// 资源商店加载骨架屏

import React from 'react';

const AssetStoreLoadingSkeleton: React.FC = () => {
  return (
    <div className="flex h-full bg-gray-900 text-white">
      {/* 主内容区域 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 顶部工具栏 */}
        <div className="p-4 border-b border-gray-700 space-y-4">
          <div className="flex gap-2">
            <div className="flex-1 h-10 bg-gray-800 rounded-lg animate-pulse" />
            <div className="w-32 h-10 bg-gray-800 rounded-lg animate-pulse" />
            <div className="flex bg-gray-700 rounded-lg">
              <div className="w-12 h-10 bg-gray-600 animate-pulse" />
              <div className="w-12 h-10 bg-gray-600 animate-pulse ml-px" />
            </div>
          </div>
          <div className="h-4 w-48 bg-gray-800 rounded animate-pulse" />
        </div>

        {/* 资源网格 */}
        <div className="flex-1 overflow-y-auto p-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {Array.from({ length: 12 }).map((_, index) => (
              <div key={index} className="bg-gray-800 rounded-lg overflow-hidden">
                <div className="aspect-square bg-gray-900 animate-pulse" />
                <div className="p-4 space-y-2">
                  <div className="h-4 bg-gray-700 rounded animate-pulse" />
                  <div className="h-3 bg-gray-700 rounded animate-pulse w-3/4" />
                  <div className="h-3 bg-gray-700 rounded animate-pulse w-1/2" />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default AssetStoreLoadingSkeleton;
