import React from 'react';

export interface EntityTreeActionsProps {
  visible: boolean;
  locked: boolean;
  onToggleVisibility: (e: React.MouseEvent) => void;
  onToggleLock: (e: React.MouseEvent) => void;
  className?: string;
}

/**
 * EntityTreeActions - Action buttons component
 * Displays visibility and lock toggle buttons
 */
export const EntityTreeActions: React.FC<EntityTreeActionsProps> = ({
  visible,
  locked,
  onToggleVisibility,
  onToggleLock,
  className,
}) => {
  const buttonClassName = 'w-5 h-5 flex items-center justify-center rounded hover:bg-slate-600';

  return (
    <div className={`flex items-center gap-1 ${className || ''}`}>
      {/* Visibility Toggle */}
      <button
        className={`${buttonClassName} ${visible ? 'text-slate-300' : 'text-slate-600'}`}
        onClick={onToggleVisibility}
        title={visible ? 'Visible' : 'Hidden'}
        aria-label={visible ? 'Hide entity' : 'Show entity'}
        aria-pressed={visible}
      >
        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          {visible ? (
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
          ) : (
            <path
              fillRule="evenodd"
              d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-1.473-1.473A10.014 10.014 0 0019.542 10C18.268 5.943 14.478 3 10 3a9.958 9.958 0 00-4.512 1.074l-1.78-1.781zm4.261 4.26l1.514 1.515a2.003 2.003 0 012.45 2.45l1.514 1.514a4 4 0 00-5.478-5.478z"
              clipRule="evenodd"
            />
          )}
        </svg>
      </button>

      {/* Lock Toggle */}
      <button
        className={`${buttonClassName} ${locked ? 'text-yellow-500' : 'text-slate-500'}`}
        onClick={onToggleLock}
        title={locked ? 'Locked' : 'Unlocked'}
        aria-label={locked ? 'Unlock entity' : 'Lock entity'}
        aria-pressed={locked}
      >
        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          {locked ? (
            <path
              fillRule="evenodd"
              d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z"
              clipRule="evenodd"
            />
          ) : (
            <path d="M10 2a5 5 0 00-5 5v2a2 2 0 00-2 2v5a2 2 0 002 2h10a2 2 0 002-2v-5a2 2 0 00-2-2H7V7a3 3 0 015.905-.75 1 1 0 001.937-.5A5.002 5.002 0 0010 2z" />
          )}
        </svg>
      </button>
    </div>
  );
};

export default EntityTreeActions;
