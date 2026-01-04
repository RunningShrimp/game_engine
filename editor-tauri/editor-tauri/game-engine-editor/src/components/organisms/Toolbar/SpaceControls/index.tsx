import React from 'react';
import { cn } from '../../../../../utils/cn';
import type { Space } from '../../../../../types/engine';

export interface SpaceControlsProps {
  space: Space;
  snapEnabled: boolean;
  onSpaceChange: (space: Space) => void;
  onSnapToggle: () => void;
  className?: string;
}

export const SpaceControls: React.FC<SpaceControlsProps> = ({
  space,
  snapEnabled,
  onSpaceChange,
  onSnapToggle,
  className,
}) => {
  return (
    <div className={cn('flex items-center gap-2', className)}>
      <button
        onClick={() => onSpaceChange(space === Space.World ? Space.Local : Space.World)}
        className={cn(
          'px-3 py-1.5 rounded text-sm font-medium transition-colors',
          space === Space.Local
            ? 'bg-blue-500 text-white'
            : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
        )}
        title="Toggle World/Local space"
      >
        {space === Space.World ? 'World' : 'Local'}
      </button>
      <button
        onClick={onSnapToggle}
        className={cn(
          'px-3 py-1.5 rounded text-sm font-medium transition-colors',
          snapEnabled
            ? 'bg-green-500 text-white'
            : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
        )}
        title="Toggle Grid Snap"
      >
        Snap
      </button>
    </div>
  );
};