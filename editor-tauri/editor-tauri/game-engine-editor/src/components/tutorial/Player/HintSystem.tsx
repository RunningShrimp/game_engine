import React, { useState } from 'react';
import { Lightbulb, ChevronLeft, ChevronRight, X } from 'lucide-react';

interface HintSystemProps {
  hints: string[];
  currentIndex: number;
  onNext: () => void;
  onPrevious: () => void;
  onClose?: () => void;
}

const HintSystem: React.FC<HintSystemProps> = ({
  hints,
  currentIndex,
  onNext,
  onPrevious,
  onClose
}) => {
  const [isRevealed, setIsRevealed] = useState(false);

  return (
    <div className="bg-yellow-50 dark:bg-yellow-900/20 border-t border-yellow-200 dark:border-yellow-800 p-4">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <Lightbulb className="w-5 h-5 text-yellow-600 dark:text-yellow-400" />
          <h4 className="font-semibold text-yellow-900 dark:text-yellow-100">
            提示 {currentIndex + 1} / {hints.length}
          </h4>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onPrevious}
            disabled={currentIndex === 0}
            className="p-1 hover:bg-yellow-200 dark:hover:bg-yellow-800 rounded disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <ChevronLeft className="w-4 h-4 text-yellow-700 dark:text-yellow-300" />
          </button>
          <button
            onClick={onNext}
            disabled={currentIndex === hints.length - 1}
            className="p-1 hover:bg-yellow-200 dark:hover:bg-yellow-800 rounded disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <ChevronRight className="w-4 h-4 text-yellow-700 dark:text-yellow-300" />
          </button>
          {onClose && (
            <button
              onClick={onClose}
              className="p-1 hover:bg-yellow-200 dark:hover:bg-yellow-800 rounded"
            >
              <X className="w-4 h-4 text-yellow-700 dark:text-yellow-300" />
            </button>
          )}
        </div>
      </div>

      <div className="relative">
        {/* 模糊遮罩 */}
        {!isRevealed && (
          <div
            className="absolute inset-0 backdrop-blur-sm bg-yellow-50/50 dark:bg-yellow-900/50 cursor-pointer z-10"
            onClick={() => setIsRevealed(true)}
          >
            <div className="flex items-center justify-center h-full">
              <span className="text-sm text-yellow-700 dark:text-yellow-300 font-medium">
                点击显示提示
              </span>
            </div>
          </div>
        )}

        <p className="text-sm text-yellow-800 dark:text-yellow-200 leading-relaxed">
          {hints[currentIndex]}
        </p>
      </div>

      {/* 提示使用警告 */}
      {currentIndex > 0 && (
        <p className="mt-2 text-xs text-yellow-700 dark:text-yellow-300">
          使用提示会减少你完成这一步获得的经验值
        </p>
      )}
    </div>
  );
};

export default HintSystem;
