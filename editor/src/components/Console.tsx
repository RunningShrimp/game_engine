import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface ConsoleLog {
  level: string;
  message: string;
  timestamp: number;
}

export const Console: React.FC = () => {
  const [logs, setLogs] = useState<ConsoleLog[]>([]);
  const [filter, setFilter] = useState('');
  const [selectedLevel, setSelectedLevel] = useState<string>('all');
  const [isAutoScroll, setIsAutoScroll] = useState(true);
  const [isCollapsed, setIsCollapsed] = useState(false);
  const logsEndRef = React.useRef<HTMLDivElement>(null);

  useEffect(() => {
    refreshLogs();
    const interval = setInterval(refreshLogs, 1000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (isAutoScroll) {
      scrollToBottom();
    }
  }, [logs, isAutoScroll]);

  const scrollToBottom = () => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const refreshLogs = async () => {
    try {
      const data = await invoke<ConsoleLog[]>('get_console_logs', { limit: 100 });
      setLogs(data);
    } catch (error) {
      console.error('Failed to fetch console logs:', error);
    }
  };

  const clearLogs = () => {
    setLogs([]);
  };

  const getLevelIcon = (level: string): string => {
    switch (level.toLowerCase()) {
      case 'error': return '❌';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
      case 'debug': return '🔍';
      default: return '📝';
    }
  };

  const getLevelClass = (level: string): string => {
    return level.toLowerCase();
  };

  const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString();
  };

  const filteredLogs = logs.filter(log => {
    const matchesFilter = filter === '' ||
      log.message.toLowerCase().includes(filter.toLowerCase());
    const matchesLevel = selectedLevel === 'all' ||
      log.level.toLowerCase() === selectedLevel.toLowerCase();
    return matchesFilter && matchesLevel;
  });

  const logCounts = {
    error: logs.filter(l => l.level.toLowerCase() === 'error').length,
    warning: logs.filter(l => l.level.toLowerCase() === 'warning').length,
    info: logs.filter(l => l.level.toLowerCase() === 'info').length,
    debug: logs.filter(l => l.level.toLowerCase() === 'debug').length,
  };

  return (
    <div className={`console ${isCollapsed ? 'collapsed' : ''}`}>
      <div className="console-header">
        <div className="console-title" onClick={() => setIsCollapsed(!isCollapsed)}>
          <h3>Console</h3>
          <span className="toggle-icon">{isCollapsed ? '▲' : '▼'}</span>
        </div>
        <div className="console-actions">
          <select
            value={selectedLevel}
            onChange={(e) => setSelectedLevel(e.target.value)}
            className="log-level-filter"
          >
            <option value="all">All</option>
            <option value="error">Errors</option>
            <option value="warning">Warnings</option>
            <option value="info">Info</option>
            <option value="debug">Debug</option>
          </select>
          <button
            onClick={() => setIsAutoScroll(!isAutoScroll)}
            className={isAutoScroll ? 'active' : ''}
            title="Auto Scroll"
          >
            Auto
          </button>
          <button onClick={clearLogs} title="Clear Console">
            Clear
          </button>
        </div>
      </div>

      {!isCollapsed && (
        <>
          <div className="console-stats">
            <span className={`stat error ${logCounts.error > 0 ? 'has-logs' : ''}`}>
              {logCounts.error} Errors
            </span>
            <span className={`stat warning ${logCounts.warning > 0 ? 'has-logs' : ''}`}>
              {logCounts.warning} Warnings
            </span>
            <span className="stat info">{logCounts.info} Info</span>
            <span className="stat debug">{logCounts.debug} Debug</span>
          </div>

          <input
            type="text"
            placeholder="Filter logs..."
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="console-filter"
          />

          <div className="console-logs">
            {filteredLogs.map((log, index) => (
              <div key={index} className={`log-entry ${getLevelClass(log.level)}`}>
                <span className="log-icon">{getLevelIcon(log.level)}</span>
                <span className="log-time">{formatTime(log.timestamp)}</span>
                <span className="log-level">{log.level.toUpperCase()}</span>
                <span className="log-message">{log.message}</span>
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        </>
      )}
    </div>
  );
};
