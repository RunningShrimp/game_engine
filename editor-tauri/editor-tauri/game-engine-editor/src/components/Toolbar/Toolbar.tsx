import React from 'react';
import { TransformMode, Space, Entity } from '../../types/engine';

interface ToolbarProps {
  transformMode: TransformMode;
  space: Space;
  isPlaying: boolean;
  isPaused: boolean;
  snapEnabled: boolean;
  canUndo: boolean;
  canRedo: boolean;
  copiedEntity: Entity | null;
  onTransformModeChange: (mode: TransformMode) => void;
  onSpaceChange: (space: Space) => void;
  onPlay: () => void;
  onPause: () => void;
  onStop: () => void;
  onSnapToggle: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onCopy: () => void;
  onPaste: () => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  transformMode,
  space,
  isPlaying,
  isPaused,
  snapEnabled,
  canUndo,
  canRedo,
  copiedEntity,
  onTransformModeChange,
  onSpaceChange,
  onPlay,
  onPause,
  onStop,
  onSnapToggle,
  onUndo,
  onRedo,
  onCopy,
  onPaste,
}) => {
  return (
    <div className="bg-slate-800 border-b border-slate-700 px-4 py-2 flex items-center justify-between">
      {/* Left Section - File operations and Undo/Redo */}
      <div className="flex items-center gap-2">
        {/* Undo/Redo */}
        <div className="flex items-center gap-1 border-r border-slate-700 pr-3">
          <button
            onClick={onUndo}
            disabled={!canUndo}
            className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
              canUndo
                ? 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                : 'bg-slate-800 text-slate-600 cursor-not-allowed'
            }`}
            title="Undo (Ctrl+Z)"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
            </svg>
          </button>
          <button
            onClick={onRedo}
            disabled={!canRedo}
            className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
              canRedo
                ? 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                : 'bg-slate-800 text-slate-600 cursor-not-allowed'
            }`}
            title="Redo (Ctrl+Shift+Z)"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 10h-10a8 8 0 00-8 8v2M21 10l-6 6m6-6l-6-6" />
            </svg>
          </button>
        </div>

        {/* Copy/Paste */}
        <div className="flex items-center gap-1 border-r border-slate-700 pr-3">
          <button
            onClick={onCopy}
            className="px-3 py-1.5 rounded text-sm font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors"
            title="Copy (Ctrl+C)"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
          <button
            onClick={onPaste}
            disabled={!copiedEntity}
            className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
              copiedEntity
                ? 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                : 'bg-slate-800 text-slate-600 cursor-not-allowed'
            }`}
            title="Paste (Ctrl+V)"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
            </svg>
          </button>
        </div>

        {/* Transform Tools */}
        <div className="flex items-center gap-1">
          <span className="text-slate-400 text-sm mr-1">Transform:</span>
          <button
            onClick={() => onTransformModeChange(TransformMode.Translate)}
            className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
              transformMode === TransformMode.Translate
                ? 'bg-blue-500 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
            title="Translate (W)"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
            </svg>
          </button>
          <button
            onClick={() => onTransformModeChange(TransformMode.Rotate)}
            className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
              transformMode === TransformMode.Rotate
                ? 'bg-blue-500 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
            title="Rotate (E)"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            onClick={() => onTransformModeChange(TransformMode.Scale)}
            className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
              transformMode === TransformMode.Scale
                ? 'bg-blue-500 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
            title="Scale (R)"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
            </svg>
          </button>
        </div>
      </div>

      {/* Center Section - Space & Snap */}
      <div className="flex items-center gap-2">
        <button
          onClick={() => onSpaceChange(space === Space.World ? Space.Local : Space.World)}
          className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
            space === Space.Local
              ? 'bg-blue-500 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
          }`}
          title="Toggle World/Local space"
        >
          {space === Space.World ? 'World' : 'Local'}
        </button>
        <button
          onClick={onSnapToggle}
          className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
            snapEnabled
              ? 'bg-green-500 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
          }`}
          title="Toggle Grid Snap"
        >
          Snap
        </button>
      </div>

      {/* Right Section - Playback Controls */}
      <div className="flex items-center gap-2">
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
    </div>
  );
};
