// History Statistics Component

import React from 'react';
import { HistoryStatistics as Stats } from '../../types/history';
import './HistoryStatistics.css';

interface HistoryStatisticsProps {
  statistics: Stats;
}

export function HistoryStatistics({ statistics }: HistoryStatisticsProps) {
  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  const formatDate = (date?: Date) => {
    if (!date) return 'Unknown';
    return new Date(date).toLocaleString();
  };

  return (
    <div className="history-statistics">
      <div className="statistics-overview">
        <div className="stat-card">
          <div className="stat-value">{statistics.totalCommands}</div>
          <div className="stat-label">Total Commands</div>
        </div>

        <div className="stat-card">
          <div className="stat-value">{statistics.undoStackDepth}</div>
          <div className="stat-label">Undo Depth</div>
        </div>

        <div className="stat-card">
          <div className="stat-value">{statistics.redoStackDepth}</div>
          <div className="stat-label">Redo Depth</div>
        </div>

        <div className="stat-card">
          <div className="stat-value">{formatBytes(statistics.memoryUsage)}</div>
          <div className="stat-label">Memory Usage</div>
        </div>
      </div>

      <div className="statistics-details">
        <div className="statistics-section">
          <h4>Bookmarks & Branches</h4>
          <div className="statistics-grid">
            <div className="stat-row">
              <span className="stat-key">Total Bookmarks:</span>
              <span className="stat-value">{statistics.totalBookmarks}</span>
            </div>
            <div className="stat-row">
              <span className="stat-key">Total Branches:</span>
              <span className="stat-value">{statistics.totalBranches}</span>
            </div>
          </div>
        </div>

        <div className="statistics-section">
          <h4>Time Range</h4>
          <div className="statistics-grid">
            <div className="stat-row">
              <span className="stat-key">Oldest Command:</span>
              <span className="stat-value">{formatDate(statistics.oldestCommand)}</span>
            </div>
            <div className="stat-row">
              <span className="stat-key">Newest Command:</span>
              <span className="stat-value">{formatDate(statistics.newestCommand)}</span>
            </div>
          </div>
        </div>

        {statistics.mostUsedCommandTypes.length > 0 && (
          <div className="statistics-section">
            <h4>Most Used Command Types</h4>
            <div className="command-types-list">
              {statistics.mostUsedCommandTypes.map((typeStat, index) => (
                <div key={typeStat.type} className="command-type-item">
                  <div className="type-rank">{index + 1}</div>
                  <div className="type-info">
                    <div className="type-name">{typeStat.type}</div>
                    <div className="type-bar-container">
                      <div
                        className="type-bar"
                        style={{ width: `${typeStat.percentage}%` }}
                      />
                    </div>
                  </div>
                  <div className="type-stats">
                    <span className="type-count">{typeStat.count}</span>
                    <span className="type-percentage">
                      {typeStat.percentage.toFixed(1)}%
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="statistics-section">
          <h4>Performance</h4>
          <div className="statistics-grid">
            <div className="stat-row">
              <span className="stat-key">Average Command Size:</span>
              <span className="stat-value">
                {formatBytes(statistics.averageCommandSize)}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
