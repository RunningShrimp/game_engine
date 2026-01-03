// Main History Panel Component

import React, { useState, useEffect } from 'react';
import { HistoryManager } from '../../utils/HistoryManager';
import {
  HistoryState,
  HistoryBookmark,
  HistoryBranch,
  TimelineEvent,
} from '../../types/history';
import { HistoryToolbar } from './HistoryToolbar';
import { HistoryTimeline } from './HistoryTimeline';
import { HistoryBookmarkList } from './HistoryBookmarkList';
import { HistoryDiffView } from './HistoryDiff';
import { HistorySearch } from './HistorySearch';
import { HistoryBranchView } from './HistoryBranchView';
import { HistoryStatistics } from './HistoryStatistics';
import './HistoryPanel.css';

interface HistoryPanelProps {
  historyManager: HistoryManager;
  onClose?: () => void;
}

export function HistoryPanel({ historyManager, onClose }: HistoryPanelProps) {
  const [history, setHistory] = useState<HistoryState | null>(null);
  const [selectedBookmark, setSelectedBookmark] = useState<HistoryBookmark | null>(null);
  const [selectedBranch, setSelectedBranch] = useState<HistoryBranch | null>(null);
  const [timelineEvents, setTimelineEvents] = useState<TimelineEvent[]>([]);
  const [compareBefore, setCompareBefore] = useState<string | null>(null);
  const [compareAfter, setCompareAfter] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'timeline' | 'bookmarks' | 'branches' | 'search' | 'stats'>('timeline');

  useEffect(() => {
    // Subscribe to history changes
    const unsubscribe = historyManager.subscribe((state) => {
      setHistory(state);
      setTimelineEvents(historyManager.getTimelineEvents());
    });

    // Load initial state
    setTimelineEvents(historyManager.getTimelineEvents());

    return () => {
      unsubscribe();
    };
  }, [historyManager]);

  const handleUndo = async () => {
    try {
      await historyManager.undo();
    } catch (error) {
      console.error('Undo failed:', error);
    }
  };

  const handleRedo = async () => {
    try {
      await historyManager.redo();
    } catch (error) {
      console.error('Redo failed:', error);
    }
  };

  const handleSave = async () => {
    try {
      await historyManager.save();
      alert('History saved successfully!');
    } catch (error) {
      console.error('Save failed:', error);
      alert('Failed to save history');
    }
  };

  const handleClear = () => {
    if (confirm('Are you sure you want to clear all history?')) {
      historyManager.clear();
    }
  };

  const handleExport = async () => {
    try {
      const blob = await historyManager.export('json');
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `history_${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Export failed:', error);
      alert('Failed to export history');
    }
  };

  const handleImport = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      await historyManager.import(file);
      alert('History imported successfully!');
    } catch (error) {
      console.error('Import failed:', error);
      alert('Failed to import history');
    }

    // Reset input
    event.target.value = '';
  };

  const handleCreateBookmark = () => {
    const name = prompt('Enter bookmark name:');
    if (!name) return;

    const description = prompt('Enter description (optional):');
    const tags = prompt('Enter tags (comma-separated, optional):');
    const tagArray = tags ? tags.split(',').map(t => t.trim()) : [];

    try {
      historyManager.createBookmark(name, description || undefined, tagArray);
      alert('Bookmark created!');
    } catch (error) {
      console.error('Failed to create bookmark:', error);
      alert('Failed to create bookmark');
    }
  };

  const handleJumpTo = async (stateId: string) => {
    try {
      // Find state and jump to it
      await historyManager.undo(); // This is simplified - real implementation needs proper state navigation
    } catch (error) {
      console.error('Failed to jump to state:', error);
    }
  };

  const handleSelectBookmark = (bookmark: HistoryBookmark) => {
    setSelectedBookmark(bookmark);
    setCompareAfter(bookmark.stateId);
  };

  const handleJumpToBookmark = async (bookmark: HistoryBookmark) => {
    try {
      await historyManager.jumpToBookmark(bookmark.id);
    } catch (error) {
      console.error('Failed to jump to bookmark:', error);
      alert('Failed to jump to bookmark');
    }
  };

  const handleCreateBranch = () => {
    const name = prompt('Enter branch name:');
    if (!name) return;

    try {
      const branch = historyManager.createBranch(name);
      alert(`Branch "${name}" created!`);
      setSelectedBranch(branch);
    } catch (error) {
      console.error('Failed to create branch:', error);
      alert('Failed to create branch');
    }
  };

  const handleSwitchBranch = async (branch: HistoryBranch) => {
    try {
      await historyManager.switchBranch(branch.id);
      setSelectedBranch(branch);
    } catch (error) {
      console.error('Failed to switch branch:', error);
      alert('Failed to switch branch');
    }
  };

  const handleDeleteBranch = (branchId: string) => {
    if (confirm('Are you sure you want to delete this branch?')) {
      try {
        historyManager.deleteBranch(branchId);
        alert('Branch deleted!');
      } catch (error) {
        console.error('Failed to delete branch:', error);
        alert('Failed to delete branch');
      }
    }
  };

  const handleOptimize = async () => {
    if (confirm('Optimize history? This will merge and compress commands.')) {
      try {
        await historyManager.optimize({
          mergeConsecutiveCommands: true,
          removeDuplicateCommands: true,
          keepBookmarks: true,
        });
        alert('History optimized!');
      } catch (error) {
        console.error('Optimization failed:', error);
        alert('Failed to optimize history');
      }
    }
  };

  if (!history) {
    return <div className="history-panel loading">Loading history...</div>;
  }

  return (
    <div className="history-panel">
      {onClose && (
        <div className="history-panel-header">
          <h2>History</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>
      )}

      <HistoryToolbar
        canUndo={historyManager.canUndo()}
        canRedo={historyManager.canRedo()}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onSave={handleSave}
        onClear={handleClear}
        onExport={handleExport}
        onImport={handleImport}
        onOptimize={handleOptimize}
      />

      <div className="history-panel-tabs">
        <button
          className={activeTab === 'timeline' ? 'active' : ''}
          onClick={() => setActiveTab('timeline')}
        >
          Timeline
        </button>
        <button
          className={activeTab === 'bookmarks' ? 'active' : ''}
          onClick={() => setActiveTab('bookmarks')}
        >
          Bookmarks ({history.bookmarks.length})
        </button>
        <button
          className={activeTab === 'branches' ? 'active' : ''}
          onClick={() => setActiveTab('branches')}
        >
          Branches ({history.branches.length})
        </button>
        <button
          className={activeTab === 'search' ? 'active' : ''}
          onClick={() => setActiveTab('search')}
        >
          Search
        </button>
        <button
          className={activeTab === 'stats' ? 'active' : ''}
          onClick={() => setActiveTab('stats')}
        >
          Statistics
        </button>
      </div>

      <div className="history-panel-content">
        {activeTab === 'timeline' && (
          <>
            <div className="timeline-actions">
              <button onClick={handleCreateBookmark}>Create Bookmark</button>
            </div>
            <HistoryTimeline
              events={timelineEvents}
              currentBranch={history.currentBranch}
              onJumpTo={handleJumpTo}
              onSelectState={(stateId) => setCompareBefore(stateId)}
            />
          </>
        )}

        {activeTab === 'bookmarks' && (
          <HistoryBookmarkList
            bookmarks={history.bookmarks}
            selectedBookmark={selectedBookmark}
            onSelect={handleSelectBookmark}
            onJumpTo={handleJumpToBookmark}
          />
        )}

        {activeTab === 'branches' && (
          <HistoryBranchView
            branches={history.branches}
            currentBranch={history.currentBranch}
            selectedBranch={selectedBranch}
            onCreateBranch={handleCreateBranch}
            onSwitchBranch={handleSwitchBranch}
            onDeleteBranch={handleDeleteBranch}
          />
        )}

        {activeTab === 'search' && (
          <HistorySearch
            historyManager={historyManager}
            onJumpTo={handleJumpTo}
          />
        )}

        {activeTab === 'stats' && (
          <HistoryStatistics
            statistics={historyManager.getStatistics()}
          />
        )}
      </div>

      {(compareBefore || compareAfter) && (
        <div className="history-diff-section">
          <h3>Compare States</h3>
          <HistoryDiffView
            before={compareBefore}
            after={compareAfter}
            historyManager={historyManager}
          />
        </div>
      )}
    </div>
  );
}
