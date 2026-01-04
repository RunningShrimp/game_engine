import React from 'react';
import { cn } from '../../../../../utils/cn';

export interface HistoryControlsProps {
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  className?: string;
}

export const HistoryControls: React.FC<HistoryControlsProps> = ({
  canUndo,
  canRedo,
  onUndo,
  onRedo,
  className,
}) => {
  return (
    <div className={cn('flex items-center gap-1', className)}>
      <button
        onClick={onUndo}
        disabled={!canUndo}
        className={cn(
          'px-3 py-1.5 rounded text-sm font-medium transition-colors',
          canUndo
            ? 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            : 'bg-slate-800 text-slate-600 cursor-not-allowed'
        )}
        title="Undo (Ctrl+Z)"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
        </svg>
      </button>
      <button
        onClick={onRedo}
        disabled={!canRedo}
        className={cn(
          'px-3 py-1.5 rounded text-sm font-medium transition-colors',
          canRedo
            ? 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            : 'bg-slate-800 text-slate-600 cursor-not-allowed'
        )}
        title="Redo (Ctrl+Shift+Z)"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 10h-10a8 8 0 00-8 8v2M21 10l-6 6m6-6l-6-6" />
        </svg>
      </button>
    </div>
  );
};