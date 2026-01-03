// History Bookmark List Component

import React from 'react';
import { HistoryBookmark } from '../../types/history';
import './HistoryBookmarkList.css';

interface HistoryBookmarkListProps {
  bookmarks: HistoryBookmark[];
  selectedBookmark: HistoryBookmark | null;
  onSelect: (bookmark: HistoryBookmark) => void;
  onJumpTo: (bookmark: HistoryBookmark) => void;
  onDelete?: (bookmarkId: string) => void;
}

export function HistoryBookmarkList({
  bookmarks,
  selectedBookmark,
  onSelect,
  onJumpTo,
  onDelete,
}: HistoryBookmarkListProps) {
  if (bookmarks.length === 0) {
    return (
      <div className="bookmark-list empty">
        <p>No bookmarks yet</p>
        <p className="hint">Create bookmarks to mark important points in history</p>
      </div>
    );
  }

  const sortedBookmarks = [...bookmarks].sort(
    (a, b) => b.timestamp.getTime() - a.timestamp.getTime()
  );

  return (
    <div className="bookmark-list">
      <div className="bookmark-list-header">
        <h3>Bookmarks ({bookmarks.length})</h3>
      </div>

      <div className="bookmark-items">
        {sortedBookmarks.map((bookmark) => (
          <BookmarkItem
            key={bookmark.id}
            bookmark={bookmark}
            isSelected={selectedBookmark?.id === bookmark.id}
            onSelect={() => onSelect(bookmark)}
            onJumpTo={() => onJumpTo(bookmark)}
            onDelete={() => onDelete?.(bookmark.id)}
          />
        ))}
      </div>
    </div>
  );
}

interface BookmarkItemProps {
  bookmark: HistoryBookmark;
  isSelected: boolean;
  onSelect: () => void;
  onJumpTo: () => void;
  onDelete: () => void;
}

function BookmarkItem({
  bookmark,
  isSelected,
  onSelect,
  onJumpTo,
  onDelete,
}: BookmarkItemProps) {
  const formatDate = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (hours < 1) return 'Just now';
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString();
  };

  return (
    <div
      className={`bookmark-item ${isSelected ? 'selected' : ''}`}
      onClick={onSelect}
    >
      <div className="bookmark-header">
        <div className="bookmark-icon">🔖</div>
        <div className="bookmark-info">
          <div className="bookmark-name">{bookmark.name}</div>
          <div className="bookmark-time">
            {formatDate(bookmark.timestamp)}
          </div>
        </div>
        <div className="bookmark-actions">
          <button
            className="bookmark-action-btn"
            onClick={(e) => {
              e.stopPropagation();
              onJumpTo();
            }}
            title="Jump to bookmark"
          >
            →
          </button>
          {onDelete && (
            <button
              className="bookmark-action-btn danger"
              onClick={(e) => {
                e.stopPropagation();
                if (confirm('Delete this bookmark?')) {
                  onDelete();
                }
              }}
              title="Delete bookmark"
            >
              ×
            </button>
          )}
        </div>
      </div>

      {bookmark.description && (
        <div className="bookmark-description">{bookmark.description}</div>
      )}

      {bookmark.tags.length > 0 && (
        <div className="bookmark-tags">
          {bookmark.tags.map((tag) => (
            <span key={tag} className="bookmark-tag">
              {tag}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
