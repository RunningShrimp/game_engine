/**
 * 播放头组件
 */

import React from 'react';
import './Playhead.css';

export interface PlayheadProps {
  currentTime: number;
  zoom: number;
}

export const Playhead: React.FC<PlayheadProps> = ({ currentTime, zoom }) => {
  const left = currentTime * zoom;

  return (
    <div
      className="playhead"
      style={{
        left: `${left}px`,
      }}
    >
      <div className="playhead-handle" />
    </div>
  );
};

export default Playhead;
