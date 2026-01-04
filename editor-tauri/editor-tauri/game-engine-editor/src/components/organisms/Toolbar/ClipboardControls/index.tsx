import React from 'react';
import { cn } from '../../../../../utils/cn';
import type { Entity } from '../../../../../types/engine';

export interface ClipboardControlsProps {
  copiedEntity: Entity | null;
  onCopy: () => void;
  onPaste: () => void;
  className?: string;
}

export const ClipboardControls: React.FC<ClipboardControlsProps> = ({
  copiedEntity,
  onCopy,
  onPaste,
  className,
}) => {
  return (
    <div className={cn('flex items-center gap-1', className)}>
      <button
        onClick={onCopy}
        className="px-3 py-1.5 rounded text-sm font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors"
        title="Copy (Ctrl+C)"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
          />
        </svg>
      </button>
      <button
        onClick={onPaste}
        disabled={!copiedEntity}
        className={cn(
          'px-3 py-1.5 rounded text-sm font-medium transition-colors',
          copiedEntity
            ? 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            : 'bg-slate-800 text-slate-600 cursor-not-allowed'
        )}
        title="Paste (Ctrl+V)"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
          />
        </svg>
      </button>
    </div>
  );
};