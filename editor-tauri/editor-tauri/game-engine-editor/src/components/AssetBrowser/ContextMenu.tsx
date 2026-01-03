import React, { useEffect, useRef } from 'react';
import {
  Copy,
  Trash2,
  Edit3,
  FolderOpen,
  FileText,
  ExternalLink,
} from 'lucide-react';

interface ContextMenuItem {
  label: string;
  icon?: React.ReactNode;
  action: () => void;
  disabled?: boolean;
  danger?: boolean;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
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

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [onClose]);

  // Adjust position if menu goes off screen
  const adjustPosition = () => {
    if (!menuRef.current) return { x, y };

    const rect = menuRef.current.getBoundingClientRect();
    const screenWidth = window.innerWidth;
    const screenHeight = window.innerHeight;

    let adjustedX = x;
    let adjustedY = y;

    if (x + rect.width > screenWidth) {
      adjustedX = screenWidth - rect.width - 10;
    }

    if (y + rect.height > screenHeight) {
      adjustedY = screenHeight - rect.height - 10;
    }

    return { x: adjustedX, y: adjustedY };
  };

  const position = adjustPosition();

  return (
    <div
      ref={menuRef}
      className="fixed bg-slate-800 border border-slate-700 rounded-lg shadow-xl py-1 min-w-[200px] z-50"
      style={{ left: position.x, top: position.y }}
    >
      {items.map((item, index) => (
        <button
          key={index}
          onClick={() => {
            if (!item.disabled) {
              item.action();
              onClose();
            }
          }}
          disabled={item.disabled}
          className={`w-full flex items-center gap-3 px-4 py-2 text-sm transition-colors ${
            item.disabled
              ? 'text-slate-500 cursor-not-allowed'
              : item.danger
              ? 'text-red-400 hover:bg-red-500/10'
              : 'text-slate-200 hover:bg-slate-700'
          }`}
        >
          {item.icon && <span className="w-4 h-4">{item.icon}</span>}
          <span className="flex-1 text-left">{item.label}</span>
        </button>
      ))}
    </div>
  );
}

// Helper function to create context menu items for assets
export function createAssetContextMenuItems(
  assetPath: string,
  assetName: string,
  onRename: () => void,
  onDelete: () => void,
  onCopyPath: () => void,
  onShowInFinder: () => void,
  onViewDependencies: () => void
): ContextMenuItem[] {
  return [
    {
      label: 'Rename',
      icon: <Edit3 className="w-4 h-4" />,
      action: onRename,
    },
    {
      label: 'Copy Path',
      icon: <Copy className="w-4 h-4" />,
      action: onCopyPath,
    },
    {
      label: 'Show in Finder',
      icon: <FolderOpen className="w-4 h-4" />,
      action: onShowInFinder,
    },
    {
      label: 'View Dependencies',
      icon: <FileText className="w-4 h-4" />,
      action: onViewDependencies,
    },
    {
      label: 'Delete',
      icon: <Trash2 className="w-4 h-4" />,
      action: onDelete,
      danger: true,
    },
  ];
}
