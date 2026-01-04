import React from 'react';

export interface EntityTreeIconProps {
  className?: string;
}

/**
 * EntityTreeIcon - Entity tree icon component
 * Displays the entity icon in the hierarchy tree
 */
export const EntityTreeIcon: React.FC<EntityTreeIconProps> = ({
  className = 'w-4 h-4 text-slate-400',
}) => {
  return (
    <svg
      className={className}
      fill="currentColor"
      viewBox="0 0 20 20"
      aria-hidden="true"
    >
      <path d="M10 2L2 10V18H18V10L10 2ZM10 4L16 10V16H4V10L10 4Z" />
    </svg>
  );
};

export default EntityTreeIcon;
