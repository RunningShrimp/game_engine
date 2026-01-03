/**
 * Alert System Component
 * Displays and manages performance alerts
 */

import React, { useState, useEffect } from 'react';
import type { PerformanceAlert } from '../../types/performance';
import { acknowledgeAlert, clearAlerts } from '../../api/performance';

interface AlertSystemProps {
  alerts: PerformanceAlert[];
  onAlertUpdate?: () => void;
}

export const AlertSystem: React.FC<AlertSystemProps> = ({ alerts, onAlertUpdate }) => {
  const [filter, setFilter] = useState<'all' | 'unacknowledged'>('all');
  const [showBanner, setShowBanner] = useState(true);

  // Auto-show banner for critical alerts
  useEffect(() => {
    const hasCriticalUnacknowledged = alerts.some(
      (a) => a.severity === 'critical' && !a.acknowledged
    );
    setShowBanner(hasCriticalUnacknowledged);
  }, [alerts]);

  const filteredAlerts = alerts.filter((alert) => {
    if (filter === 'unacknowledged') {
      return !alert.acknowledged;
    }
    return true;
  });

  const unacknowledgedCount = alerts.filter((a) => !a.acknowledged).length;
  const criticalCount = alerts.filter(
    (a) => a.severity === 'critical' && !a.acknowledged
  ).length;

  const handleAcknowledge = async (alertId: string) => {
    try {
      await acknowledgeAlert(alertId);
      onAlertUpdate?.();
    } catch (error) {
      console.error('Failed to acknowledge alert:', error);
    }
  };

  const handleClearAll = async () => {
    try {
      await clearAlerts();
      onAlertUpdate?.();
    } catch (error) {
      console.error('Failed to clear alerts:', error);
    }
  };

  const getSeverityIcon = (severity: string): string => {
    switch (severity) {
      case 'critical':
        return '🔴';
      case 'warning':
        return '🟡';
      case 'info':
        return '🔵';
      default:
        return '⚪';
    }
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'critical':
        return 'border-red-500 bg-red-500/10';
      case 'warning':
        return 'border-yellow-500 bg-yellow-500/10';
      case 'info':
        return 'border-blue-500 bg-blue-500/10';
      default:
        return 'border-slate-500 bg-slate-500/10';
    }
  };

  return (
    <>
      {/* Alert Banner */}
      {showBanner && criticalCount > 0 && (
        <div className="fixed top-0 left-0 right-0 bg-red-500 text-white px-4 py-2 z-50 animate-pulse">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-xl">🚨</span>
              <span className="font-semibold">
                {criticalCount} Critical Alert{criticalCount > 1 ? 's' : ''}!
              </span>
            </div>
            <button
              onClick={() => setShowBanner(false)}
              className="text-white hover:text-red-200 transition-colors"
            >
              ✕
            </button>
          </div>
        </div>
      )}

      {/* Alert Panel */}
      <div className="flex flex-col h-full">
        {/* Header */}
        <div className="px-4 py-3 border-b border-slate-700 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <h3 className="text-sm font-semibold text-slate-200">Alerts</h3>
            {unacknowledgedCount > 0 && (
              <span className="bg-red-500 text-white text-xs px-2 py-0.5 rounded-full">
                {unacknowledgedCount}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value as any)}
              className="bg-slate-700 text-slate-200 text-xs rounded px-2 py-1 border border-slate-600"
            >
              <option value="all">All</option>
              <option value="unacknowledged">Unacknowledged</option>
            </select>
            {alerts.length > 0 && (
              <button
                onClick={handleClearAll}
                className="text-xs text-slate-400 hover:text-red-400 transition-colors"
              >
                Clear All
              </button>
            )}
          </div>
        </div>

        {/* Alert List */}
        <div className="flex-1 overflow-y-auto">
          {filteredAlerts.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-slate-500 text-sm gap-2">
              <span className="text-4xl">✅</span>
              <span>No alerts</span>
            </div>
          ) : (
            <div className="divide-y divide-slate-700">
              {filteredAlerts.map((alert) => (
                <div
                  key={alert.id}
                  className={`px-4 py-3 border-l-4 ${getSeverityColor(
                    alert.severity
                  )} ${alert.acknowledged ? 'opacity-50' : ''}`}
                >
                  <div className="flex items-start gap-3">
                    <span className="text-xl flex-shrink-0">
                      {getSeverityIcon(alert.severity)}
                    </span>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-2">
                        <h4 className="text-sm font-medium text-slate-200">
                          {alert.message}
                        </h4>
                        {!alert.acknowledged && (
                          <button
                            onClick={() => handleAcknowledge(alert.id)}
                            className="text-xs text-blue-400 hover:text-blue-300 transition-colors flex-shrink-0"
                          >
                            Acknowledge
                          </button>
                        )}
                      </div>
                      <div className="mt-1 flex items-center gap-3 text-xs text-slate-400">
                        <span>
                          {new Date(alert.timestamp).toLocaleTimeString()}
                        </span>
                        <span>•</span>
                        <span className="capitalize">{alert.type}</span>
                        <span>•</span>
                        <span>
                          Value: {alert.value.toFixed(1)} / Threshold:{' '}
                          {alert.threshold.toFixed(1)}
                        </span>
                      </div>
                      {alert.acknowledged && (
                        <div className="mt-1 text-xs text-slate-500">
                          ✓ Acknowledged
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  );
};
