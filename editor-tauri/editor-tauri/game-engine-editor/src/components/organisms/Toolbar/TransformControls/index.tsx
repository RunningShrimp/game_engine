import React from 'react';
import { cn } from '../../../../../utils/cn';
import type { TransformMode } from '../../../../../types/engine';

export interface TransformControlsProps {
  transformMode: TransformMode;
  onTransformModeChange: (mode: TransformMode) => void;
  className?: string;
}

export const TransformControls: React.FC<TransformControlsProps> = ({
  transformMode,
  onTransformModeChange,
  className,
}) => {
  const modes = [
    { mode: TransformMode.Translate, icon: 'translate', label: 'Translate (W)' },
    { mode: TransformMode.Rotate, icon: 'rotate', label: 'Rotate (E)' },
    { mode: TransformMode.Scale, icon: 'scale', label: 'Scale (R)' },
  ] as const;

  return (
    <div className={cn('flex items-center gap-1', className)}>
      <span className="text-slate-400 text-sm mr-1">Transform:</span>
      {modes.map(({ mode, icon, label }) => (
        <button
          key={mode}
          onClick={() => onTransformModeChange(mode)}
          className={cn(
            'px-3 py-1.5 rounded text-sm font-medium transition-colors',
            transformMode === mode
              ? 'bg-blue-500 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
          )}
          title={label}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            {icon === 'translate' && (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
              />
            )}
            {icon === 'rotate' && (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            )}
            {icon === 'scale' && (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"
              />
            )}
          </svg>
        </button>
      ))}
    </div>
  );
};
