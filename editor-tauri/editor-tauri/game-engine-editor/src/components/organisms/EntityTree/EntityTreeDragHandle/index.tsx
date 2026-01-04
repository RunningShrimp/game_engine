import React from 'react';

export interface EntityTreeDragHandleProps {
  className?: string;
}

/**
 * EntityTreeDragHandle - Drag handle component
 * Visual indicator for draggable entities
 */
export const EntityTreeDragHandle: React.FC<EntityTreeDragHandleProps> = ({
  className = 'w-4',
}) => {
  return (
    <div className={className} aria-hidden="true">
      {/* Empty spacer for alignment */}
    </div>
  );
};

export default EntityTreeDragHandle;
