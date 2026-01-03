// History Timeline Component

import React from 'react';
import { TimelineEvent } from '../../types/history';
import './HistoryTimeline.css';

interface HistoryTimelineProps {
  events: TimelineEvent[];
  currentBranch: string;
  onJumpTo: (stateId: string) => void;
  onSelectState?: (stateId: string) => void;
}

export function HistoryTimeline({
  events,
  currentBranch,
  onJumpTo,
  onSelectState,
}: HistoryTimelineProps) {
  const groupedEvents = groupEventsByDate(events);

  return (
    <div className="history-timeline">
      <div className="timeline-container">
        {Object.entries(groupedEvents).map(([date, dateEvents]) => (
          <div key={date} className="timeline-date-group">
            <div className="timeline-date-header">{date}</div>
            <div className="timeline-events">
              {dateEvents.map((event, index) => (
                <TimelineEventItem
                  key={`${event.id}_${index}`}
                  event={event}
                  onJumpTo={onJumpTo}
                  onSelectState={onSelectState}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

interface TimelineEventItemProps {
  event: TimelineEvent;
  onJumpTo: (stateId: string) => void;
  onSelectState?: (stateId: string) => void;
}

function TimelineEventItem({ event, onJumpTo, onSelectState }: TimelineEventItemProps) {
  const handleClick = () => {
    if (event.type === 'command' && onSelectState) {
      onSelectState(event.id);
    }
  };

  const handleDoubleClick = () => {
    if (event.type === 'command') {
      onJumpTo(event.id);
    }
  };

  const getEventIcon = () => {
    switch (event.type) {
      case 'command':
        return '⚙️';
      case 'bookmark':
        return '🔖';
      case 'branch':
        return '🌿';
      default:
        return '•';
    }
  };

  const getEventColor = () => {
    switch (event.type) {
      case 'command':
        return '#4a9eff';
      case 'bookmark':
        return '#ff9e4a';
      case 'branch':
        return '#4aff9e';
      default:
        return '#888';
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const getEventDescription = () => {
    if (event.type === 'command') {
      const cmd = event.data as any;
      return cmd.description || 'Unknown Command';
    } else if (event.type === 'bookmark') {
      const bookmark = event.data as any;
      return `Bookmark: ${bookmark.name}`;
    } else if (event.type === 'branch') {
      const branch = event.data as any;
      return `Branch: ${branch.name}`;
    }
    return 'Unknown Event';
  };

  return (
    <div
      className="timeline-event-item"
      style={{ borderLeftColor: getEventColor() }}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      title={getEventDescription()}
    >
      <div className="event-icon">{getEventIcon()}</div>
      <div className="event-content">
        <div className="event-time">{formatTime(event.timestamp)}</div>
        <div className="event-description">{getEventDescription()}</div>
        {event.metadata && (
          <div className="event-metadata">
            {Object.entries(event.metadata).map(([key, value]) => (
              <span key={key} className="metadata-tag">
                {key}: {String(value)}
              </span>
            ))}
          </div>
        )}
      </div>
      {event.type === 'command' && onSelectState && (
        <button className="event-select-btn" onClick={(e) => {
          e.stopPropagation();
          handleClick();
        }}>
          Select
        </button>
      )}
    </div>
  );
}

function groupEventsByDate(events: TimelineEvent[]): Record<string, TimelineEvent[]> {
  const grouped: Record<string, TimelineEvent[]> = {};

  events.forEach((event) => {
    const date = event.timestamp.toLocaleDateString();
    if (!grouped[date]) {
      grouped[date] = [];
    }
    grouped[date].push(event);
  });

  // Sort events within each date by time
  Object.keys(grouped).forEach((date) => {
    grouped[date].sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
  });

  return grouped;
}
