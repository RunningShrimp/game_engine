// History Branch View Component

import React from 'react';
import { HistoryBranch } from '../../types/history';
import './HistoryBranchView.css';

interface HistoryBranchViewProps {
  branches: HistoryBranch[];
  currentBranch: string;
  selectedBranch: HistoryBranch | null;
  onCreateBranch: () => void;
  onSwitchBranch: (branch: HistoryBranch) => void;
  onDeleteBranch: (branchId: string) => void;
}

export function HistoryBranchView({
  branches,
  currentBranch,
  selectedBranch,
  onCreateBranch,
  onSwitchBranch,
  onDeleteBranch,
}: HistoryBranchViewProps) {
  return (
    <div className="history-branch-view">
      <div className="branch-view-header">
        <h3>Branches ({branches.length})</h3>
        <button className="create-branch-btn" onClick={onCreateBranch}>
          + New Branch
        </button>
      </div>

      <div className="branch-list">
        {branches.map((branch) => (
          <BranchItem
            key={branch.id}
            branch={branch}
            isCurrent={branch.id === currentBranch}
            isSelected={selectedBranch?.id === branch.id}
            onSwitch={() => onSwitchBranch(branch)}
            onDelete={() => onDeleteBranch(branch.id)}
          />
        ))}
      </div>

      <div className="branch-view-footer">
        <p className="branch-hint">
          Branches allow you to work on parallel timelines.
          Create a branch to experiment without affecting your main work.
        </p>
      </div>
    </div>
  );
}

interface BranchItemProps {
  branch: HistoryBranch;
  isCurrent: boolean;
  isSelected: boolean;
  onSwitch: () => void;
  onDelete: () => void;
}

function BranchItem({ branch, isCurrent, isSelected, onSwitch, onDelete }: BranchItemProps) {
  const formatDate = (date: Date) => {
    return date.toLocaleDateString([], {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div
      className={`branch-item ${isCurrent ? 'current' : ''} ${isSelected ? 'selected' : ''}`}
    >
      <div className="branch-header">
        <div className="branch-icon">🌿</div>
        <div className="branch-info">
          <div className="branch-name-row">
            <span className="branch-name">{branch.name}</span>
            {isCurrent && <span className="current-badge">Current</span>}
          </div>
          <div className="branch-meta">
            <span className="branch-id">ID: {branch.id.slice(0, 8)}...</span>
            <span className="branch-date">{formatDate(branch.createdAt)}</span>
          </div>
        </div>

        <div className="branch-actions">
          {!isCurrent && (
            <button
              className="branch-action-btn"
              onClick={onSwitch}
              title="Switch to this branch"
            >
              Switch
            </button>
          )}
          {!isCurrent && (
            <button
              className="branch-action-btn danger"
              onClick={() => {
                if (confirm('Delete this branch?')) {
                  onDelete();
                }
              }}
              title="Delete branch"
            >
              Delete
            </button>
          )}
        </div>
      </div>

      {branch.parentId && (
        <div className="branch-parent">
          Parent: {branch.parentId.slice(0, 8)}...
        </div>
      )}
    </div>
  );
}
