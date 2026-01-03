import React from 'react';

interface ProgressBarProps {
  current: number;
  total: number;
  percent: number;
}

const ProgressBar: React.FC<ProgressBarProps> = ({ current, total, percent }) => {
  return (
    <div className="w-full">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          教程进度
        </span>
        <span className="text-sm text-gray-600 dark:text-gray-400">
          {current} / {total} ({Math.round(percent)}%)
        </span>
      </div>

      {/* 进度条 */}
      <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden">
        <div
          className="bg-gradient-to-r from-blue-500 to-purple-600 h-full transition-all duration-500 ease-out"
          style={{ width: `${percent}%` }}
        />
      </div>

      {/* 步骤指示器 */}
      <div className="flex gap-1 mt-2">
        {Array.from({ length: total }).map((_, index) => (
          <div
            key={index}
            className={`flex-1 h-1 rounded-full transition-all duration-300 ${
              index < current
                ? 'bg-blue-500'
                : index === current
                ? 'bg-blue-300'
                : 'bg-gray-300 dark:bg-gray-600'
            }`}
          />
        ))}
      </div>
    </div>
  );
};

export default ProgressBar;
