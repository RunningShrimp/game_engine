import React from 'react';

export interface EntityTreeToggleProps {
  expanded: boolean;
  onToggle: () => void;
  className?: string;
}

/**
 * EntityTreeToggle - Expand/collapse toggle component
 * Displays a toggle button for entities with children
 */
export const EntityTreeToggle: React.FC<EntityTreeToggleProps> = ({
  expanded,
  onToggle,
  className = 'w-4 h-4 flex items-center justify-center text-slate-400 hover:text-slate-200',
}) => {
  return (
    <button
      className={className}
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
      aria-label={expanded ? 'Collapse' : 'Expand'}
      aria-expanded={expanded}
    >
      <svg
        className={`w-3 h-3 transition-transform ${expanded ? 'rotate-90' : ''}`}
        fill="currentColor"
        viewBox="0 0 20 20"
      >
        <path d="M6 6L14 10L6 14V6Z" />
      </svg>
    </button>
  );
};

export default EntityTreeToggle;
