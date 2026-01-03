// History Diff View Component

import React from 'react';
import { HistoryManager } from '../../utils/HistoryManager';
import { HistoryDiff } from '../../types/history';
import './HistoryDiff.css';

interface HistoryDiffViewProps {
  before: string | null;
  after: string | null;
  historyManager: HistoryManager;
}

export function HistoryDiff({ before, after, historyManager }: HistoryDiffViewProps) {
  const [diff, setDiff] = React.useState<HistoryDiff | null>(null);

  React.useEffect(() => {
    if (before && after) {
      try {
        const diffResult = historyManager.compare(before, after);
        setDiff(diffResult);
      } catch (error) {
        console.error('Failed to compare states:', error);
      }
    }
  }, [before, after, historyManager]);

  if (!before || !after) {
    return (
      <div className="history-diff-view empty">
        <p>Select two states to compare</p>
      </div>
    );
  }

  if (!diff) {
    return <div className="history-diff-view loading">Loading diff...</div>;
  }

  return (
    <div className="history-diff-view">
      <div className="diff-header">
        <div className="diff-state-info">
          <div className="diff-state before">
            <span className="state-label">Before:</span>
            <code className="state-id">{before.slice(0, 12)}...</code>
          </div>
          <div className="diff-separator">→</div>
          <div className="diff-state after">
            <span className="state-label">After:</span>
            <code className="state-id">{after.slice(0, 12)}...</code>
          </div>
        </div>
        <div className="diff-timestamp">
          {new Date(diff.timestamp).toLocaleString()}
        </div>
      </div>

      <div className="diff-content">
        {diff.added.length > 0 && (
          <div className="diff-section added">
            <h4>Added ({diff.added.length})</h4>
            <div className="diff-commands">
              {diff.added.map((cmd) => (
                <DiffCommandItem key={cmd.id} command={cmd} type="added" />
              ))}
            </div>
          </div>
        )}

        {diff.removed.length > 0 && (
          <div className="diff-section removed">
            <h4>Removed ({diff.removed.length})</h4>
            <div className="diff-commands">
              {diff.removed.map((cmd) => (
                <DiffCommandItem key={cmd.id} command={cmd} type="removed" />
              ))}
            </div>
          </div>
        )}

        {diff.modified.length > 0 && (
          <div className="diff-section modified">
            <h4>Modified ({diff.modified.length})</h4>
            <div className="diff-commands">
              {diff.modified.map((mod) => (
                <ModifiedCommandItem key={mod.command.id} modification={mod} />
              ))}
            </div>
          </div>
        )}

        {diff.added.length === 0 &&
          diff.removed.length === 0 &&
          diff.modified.length === 0 && (
            <div className="diff-empty">
              <p>No differences found between these states</p>
            </div>
          )}
      </div>
    </div>
  );
}

interface DiffCommandItemProps {
  command: any;
  type: 'added' | 'removed';
}

function DiffCommandItem({ command, type }: DiffCommandItemProps) {
  const icon = type === 'added' ? '+' : '-';
  const time = command.timestamp
    ? new Date(command.timestamp).toLocaleTimeString()
    : 'Unknown time';

  return (
    <div className={`diff-command-item ${type}`}>
      <div className="diff-command-icon">{icon}</div>
      <div className="diff-command-content">
        <div className="diff-command-description">{command.description}</div>
        <div className="diff-command-meta">
          <span className="command-id">{command.id?.slice(0, 12)}...</span>
          <span className="command-time">{time}</span>
        </div>
      </div>
    </div>
  );
}

interface ModifiedCommandItemProps {
  modification: any;
}

function ModifiedCommandItem({ modification }: ModifiedCommandItemProps) {
  const { command, previousState, newState } = modification;

  return (
    <div className="diff-command-item modified">
      <div className="diff-command-icon">~</div>
      <div className="diff-command-content">
        <div className="diff-command-description">{command.description}</div>

        {(previousState || newState) && (
          <div className="diff-command-changes">
            {previousState && (
              <div className="diff-change before">
                <span className="change-label">Before:</span>
                <code className="change-value">
                  {JSON.stringify(previousState, null, 2)}
                </code>
              </div>
            )}
            {newState && (
              <div className="diff-change after">
                <span className="change-label">After:</span>
                <code className="change-value">
                  {JSON.stringify(newState, null, 2)}
                </code>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
