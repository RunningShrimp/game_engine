import React, { useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';

export interface ContextMenuItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
  danger?: boolean;
}

export interface EntityTreeContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

/**
 * EntityTreeContextMenu - Right-click context menu component
 * Displays context menu for entity actions
 */
export const EntityTreeContextMenu: React.FC<EntityTreeContextMenuProps> = ({
  visible,
  x,
  y,
  items,
  onClose,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    if (visible) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleEscape);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
        document.removeEventListener('keydown', handleEscape);
      };
    }
  }, [visible, onClose]);

  if (!visible) {
    return null;
  }

  const menuContent = (
    <div
      ref={menuRef}
      className="fixed z-50 bg-slate-800 border border-slate-700 rounded shadow-xl py-1 min-w-48"
      style={{
        left: `${x}px`,
        top: `${y}px`,
      }}
      role="menu"
      aria-orientation="vertical"
    >
      {items.map((item, index) => (
        <React.Fragment key={item.id}>
          {index > 0 && item.danger && (
            <div className="border-t border-slate-700 my-1" role="separator" />
          )}
          <button
            className={`w-full px-3 py-1.5 text-left text-sm hover:bg-slate-700 flex items-center gap-2 ${
              item.danger ? 'text-red-400' : 'text-slate-200'
            }`}
            onClick={() => {
              item.onClick();
              onClose();
            }}
            role="menuitem"
            tabIndex={0}
          >
            {item.icon}
            <span>{item.label}</span>
          </button>
        </React.Fragment>
      ))}
    </div>
  );

  return createPortal(menuContent, document.body);
};

export default EntityTreeContextMenu;
