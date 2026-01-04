# ResizablePanel Component System

A flexible and performant resizable panel component system for React applications.

## Features

- **Bidirectional Support**: Works on both left and right positions
- **Size Constraints**: Supports minWidth and maxWidth limits
- **Smooth Dragging**: Optimized drag experience with visual feedback
- **Visual Feedback**: Cursor changes, handle highlighting, and grip indicators
- **Resize Callbacks**: onResize, onResizeStart, and onResizeEnd events
- **Performance**: useCallback optimization and optional resize debouncing
- **Text Selection Prevention**: Prevents unwanted text selection during drag
- **Dark Theme Support**: Automatically adapts to system color scheme
- **Responsive Design**: Mobile-friendly with larger touch targets

## Installation

The component is located at:
```
src/components/ResizablePanel/
```

## Basic Usage

```tsx
import { ResizablePanel } from './components/ResizablePanel';

function App() {
  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      <ResizablePanel
        position="left"
        initialWidth={300}
        minWidth={200}
        maxWidth={600}
      >
        <div>
          <h2>Left Panel</h2>
          <p>This panel can be resized by dragging the handle on the right edge.</p>
        </div>
      </ResizablePanel>

      <div style={{ flex: 1, padding: '20px' }}>
        <h1>Main Content</h1>
      </div>
    </div>
  );
}
```

## Props

### ResizablePanelProps

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `children` | `React.ReactNode` | - | Content to display in the panel |
| `position` | `'left' \| 'right'` | `'left'` | Position of the resize handle |
| `initialWidth` | `number` | `300` | Initial width of the panel in pixels |
| `minWidth` | `number` | `200` | Minimum width in pixels |
| `maxWidth` | `number` | `800` | Maximum width in pixels |
| `className` | `string` | `''` | Additional CSS classes |
| `onResize` | `(width: number) => void` | - | Callback when panel is resized |
| `onResizeStart` | `() => void` | - | Callback when resize starts |
| `onResizeEnd` | `() => void` | - | Callback when resize ends |
| `resizeDebounce` | `number` | `0` | Debounce delay in milliseconds (0 = no debounce) |
| `style` | `CSSProperties` | - | Additional inline styles |

## Advanced Examples

### With Resize Callbacks

```tsx
const [panelWidth, setPanelWidth] = useState(300);

<ResizablePanel
  initialWidth={panelWidth}
  minWidth={200}
  maxWidth={600}
  onResize={(width) => {
    console.log('Panel resized to:', width);
    setPanelWidth(width);
  }}
  onResizeStart={() => console.log('Resize started')}
  onResizeEnd={() => console.log('Resize ended')}
>
  {/* Panel content */}
</ResizablePanel>
```

### With Debounced Resize

```tsx
<ResizablePanel
  initialWidth={300}
  resizeDebounce={150} // Wait 150ms after resize stops before calling onResize
  onResize={(width) => {
    // This will only be called 150ms after the user stops dragging
    savePanelWidthToStorage(width);
  }}
>
  {/* Panel content */}
</ResizablePanel>
```

### Two-Panel Layout

```tsx
function TwoPanelLayout() {
  const [leftPanelWidth, setLeftPanelWidth] = useState(300);
  const [rightPanelWidth, setRightPanelWidth] = useState(300);

  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      {/* Left Panel */}
      <ResizablePanel
        position="right"
        initialWidth={leftPanelWidth}
        minWidth={200}
        maxWidth={500}
        onResize={setLeftPanelWidth}
      >
        <LeftPanelContent />
      </ResizablePanel>

      {/* Main Content */}
      <div style={{ flex: 1, overflow: 'auto' }}>
        <MainContent />
      </div>

      {/* Right Panel */}
      <ResizablePanel
        position="left"
        initialWidth={rightPanelWidth}
        minWidth={200}
        maxWidth={500}
        onResize={setRightPanelWidth}
      >
        <RightPanelContent />
      </ResizablePanel>
    </div>
  );
}
```

### With Custom Styling

```tsx
<ResizablePanel
  className="custom-panel"
  initialWidth={350}
  style={{
    backgroundColor: '#f5f5f5',
    borderRight: '1px solid #ddd'
  }}
>
  {/* Panel content */}
</ResizablePanel>
```

```css
.custom-panel {
  border-radius: 8px;
  overflow: hidden;
}

.custom-panel .resizable-panel-handle {
  background: #e0e0e0;
}
```

### Persisting Panel Size

```tsx
function PersistentPanel() {
  const [width, setWidth] = useState(() => {
    // Load saved width from localStorage
    const saved = localStorage.getItem('panel-width');
    return saved ? parseInt(saved, 10) : 300;
  });

  const handleResize = (newWidth: number) => {
    setWidth(newWidth);
    // Save to localStorage
    localStorage.setItem('panel-width', newWidth.toString());
  };

  return (
    <ResizablePanel
      initialWidth={width}
      minWidth={200}
      maxWidth={600}
      onResize={handleResize}
      resizeDebounce={100}
    >
      {/* Panel content */}
    </ResizablePanel>
  );
}
```

## Performance Considerations

1. **Debouncing**: Use `resizeDebounce` prop to reduce the frequency of resize callbacks, especially if performing expensive operations like re-rendering large trees or making API calls.

2. **Event Cleanup**: The component automatically cleans up event listeners when dragging stops or the component unmounts.

3. **useCallback**: All event handlers are memoized using `useCallback` to prevent unnecessary re-renders.

4. **Debounce Cleanup**: Debounce timers are properly cleaned up on component unmount to prevent memory leaks.

## Accessibility

- The resize handle has `focus-visible` styles for keyboard navigation support
- Cursor changes provide clear visual feedback
- Text selection is prevented during drag to improve UX

## Browser Support

- Chrome/Edge: Full support
- Firefox: Full support
- Safari: Full support
- Mobile browsers: Full support (with larger touch targets)

## CSS Customization

The component uses CSS classes that can be customized:

```css
/* Panel container */
.resizable-panel

/* Handle container */
.resizable-panel-handle
.resizable-panel-handle-left
.resizable-panel-handle-right

/* Handle states */
.resizable-panel-handle:hover
.resizable-panel-handle.dragging

/* Grip indicator */
.handle-grip
.grip-line
```

## File Structure

```
ResizablePanel/
├── ResizablePanel.tsx      # Main panel component
├── PanelHandle.tsx          # Resize handle component
├── ResizablePanel.css       # Component styles
├── index.ts                 # Exports
└── README.md               # Documentation
```

## License

Part of the Game Engine Editor project.
