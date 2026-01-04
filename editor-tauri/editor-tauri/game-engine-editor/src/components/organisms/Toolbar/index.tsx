import React from 'react';
import { HistoryControls } from './HistoryControls';
import { ClipboardControls } from './ClipboardControls';
import { TransformControls } from './TransformControls';
import { SpaceControls } from './SpaceControls';
import { PlaybackControls } from './PlaybackControls';
import { ToolbarGroup } from '../../molecules';
import type { TransformMode, Space, Entity } from '../../../types/engine';

export interface ToolbarProps {
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
  className?: string;
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
  className,
}) => {
  return (
    <div
      className={`bg-slate-800 border-b border-slate-700 px-4 py-2 flex items-center justify-between ${className || ''}`}
    >
      {/* Left Section - File operations and Undo/Redo */}
      <div className="flex items-center gap-2">
        {/* Undo/Redo */}
        <ToolbarGroup showDivider>
          <HistoryControls
            canUndo={canUndo}
            canRedo={canRedo}
            onUndo={onUndo}
            onRedo={onRedo}
          />
        </ToolbarGroup>

        {/* Copy/Paste */}
        <ToolbarGroup showDivider>
          <ClipboardControls
            copiedEntity={copiedEntity}
            onCopy={onCopy}
            onPaste={onPaste}
          />
        </ToolbarGroup>

        {/* Transform Tools */}
        <TransformControls
          transformMode={transformMode}
          onTransformModeChange={onTransformModeChange}
        />
      </div>

      {/* Center Section - Space & Snap */}
      <SpaceControls
        space={space}
        snapEnabled={snapEnabled}
        onSpaceChange={onSpaceChange}
        onSnapToggle={onSnapToggle}
      />

      {/* Right Section - Playback Controls */}
      <PlaybackControls
        isPlaying={isPlaying}
        isPaused={isPaused}
        onPlay={onPlay}
        onPause={onPause}
        onStop={onStop}
      />
    </div>
  );
};