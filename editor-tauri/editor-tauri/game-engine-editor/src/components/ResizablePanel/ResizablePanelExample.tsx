import React, { useState } from 'react';
import { ResizablePanel } from './ResizablePanel';
import './ResizablePanelExample.css';

/**
 * Example component demonstrating the ResizablePanel usage
 */
export const ResizablePanelExample: React.FC = () => {
  const [leftPanelWidth, setLeftPanelWidth] = useState(300);
  const [rightPanelWidth, setRightPanelWidth] = useState(250);
  const [resizeCount, setResizeCount] = useState(0);

  const handleLeftResize = (width: number) => {
    setLeftPanelWidth(width);
    setResizeCount(prev => prev + 1);
  };

  const handleRightResize = (width: number) => {
    setRightPanelWidth(width);
    setResizeCount(prev => prev + 1);
  };

  return (
    <div className="panel-example-container">
      <div className="panel-example-header">
        <h1>Resizable Panel Demo</h1>
        <div className="panel-stats">
          <span>Left Panel: {leftPanelWidth}px</span>
          <span>Right Panel: {rightPanelWidth}px</span>
          <span>Total Resizes: {resizeCount}</span>
        </div>
      </div>

      <div className="panel-example-layout">
        {/* Left Panel */}
        <ResizablePanel
          position="right"
          initialWidth={leftPanelWidth}
          minWidth={200}
          maxWidth={500}
          onResize={handleLeftResize}
          onResizeStart={() => console.log('Left panel resize started')}
          onResizeEnd={() => console.log('Left panel resize ended')}
          className="left-example-panel"
        >
          <div className="panel-content">
            <h2>Left Panel</h2>
            <p>This panel can be resized by dragging the handle on the right edge.</p>
            <div className="panel-info">
              <h3>Features:</h3>
              <ul>
                <li>Position: Right handle</li>
                <li>Min Width: 200px</li>
                <li>Max Width: 500px</li>
                <li>Current: {leftPanelWidth}px</li>
              </ul>
            </div>
            <div className="panel-code">
              <code>
                {`<ResizablePanel
  position="right"
  initialWidth={${leftPanelWidth}}
  minWidth={200}
  maxWidth={500}
  onResize={handleLeftResize}
>`}
              </code>
            </div>
          </div>
        </ResizablePanel>

        {/* Main Content */}
        <div className="main-content">
          <h2>Main Content Area</h2>
          <p>
            This is the main content area that takes up the remaining space.
            The panels on the left and right can be resized independently.
          </p>
          <div className="content-sections">
            <section>
              <h3>How to Use</h3>
              <ol>
                <li>Hover over the panel edge to see the resize handle</li>
                <li>Click and drag the handle to resize the panel</li>
                <li>The panel will respect min/max width constraints</li>
                <li>Resize events are logged to the console</li>
              </ol>
            </section>
            <section>
              <h3>Features</h3>
              <ul>
                <li>Smooth drag experience with visual feedback</li>
                <li>Cursor changes during resize</li>
                <li>Handle highlights on hover</li>
                <li>Text selection prevented during drag</li>
                <li>Resize callbacks for state management</li>
                <li>Optional debouncing for performance</li>
              </ul>
            </section>
          </div>
        </div>

        {/* Right Panel */}
        <ResizablePanel
          position="left"
          initialWidth={rightPanelWidth}
          minWidth={150}
          maxWidth={400}
          onResize={handleRightResize}
          resizeDebounce={100}
          className="right-example-panel"
        >
          <div className="panel-content">
            <h2>Right Panel</h2>
            <p>This panel has a left-side handle with debounced resize events.</p>
            <div className="panel-info">
              <h3>Configuration:</h3>
              <ul>
                <li>Position: Left handle</li>
                <li>Min Width: 150px</li>
                <li>Max Width: 400px</li>
                <li>Debounce: 100ms</li>
                <li>Current: {rightPanelWidth}px</li>
              </ul>
            </div>
            <div className="panel-code">
              <code>
                {`<ResizablePanel
  position="left"
  initialWidth={${rightPanelWidth}}
  minWidth={150}
  maxWidth={400}
  onResize={handleRightResize}
  resizeDebounce={100}
>`}
              </code>
            </div>
          </div>
        </ResizablePanel>
      </div>
    </div>
  );
};
