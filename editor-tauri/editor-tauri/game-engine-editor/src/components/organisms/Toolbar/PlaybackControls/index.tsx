import React from 'react';
import { cn } from '../../../../../utils/cn';

export interface PlaybackControlsProps {
  isPlaying: boolean;
  isPaused: boolean;
  onPlay: () => void;
  onPause: () => void;
  onStop: () => void;
  className?: string;
}

export const PlaybackControls: React.FC<PlaybackControlsProps> = ({
  isPlaying,
  isPaused,
  onPlay,
  onPause,
  onStop,
  className,
}) => {
  return (
    <div className={cn('flex items-center gap-2', className)}>
      <button
        onClick={onPlay}
        disabled={isPlaying && !isPaused}
        className="px-3 py-1.5 rounded text-sm font-medium bg-green-600 text-white hover:bg-green-700 disabled:bg-slate-700 disabled:text-slate-500 transition-colors"
        title="Play (Ctrl+P)"
      >
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M8 5v14l11-7z" />
        </svg>
      </button>
      <button
        onClick={onPause}
        disabled={!isPlaying}
        className="px-3 py-1.5 rounded text-sm font-medium bg-yellow-600 text-white hover:bg-yellow-700 disabled:bg-slate-700 disabled:text-slate-500 transition-colors"
        title="Pause"
      >
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
        </svg>
      </button>
      <button
        onClick={onStop}
        disabled={!isPlaying}
        className="px-3 py-1.5 rounded text-sm font-medium bg-red-600 text-white hover:bg-red-700 disabled:bg-slate-700 disabled:text-slate-500 transition-colors"
        title="Stop"
      >
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M6 6h12v12H6z" />
        </svg>
      </button>
    </div>
  );
};