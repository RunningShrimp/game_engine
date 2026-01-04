# ResizablePanel Quick Start Guide

## 5-Minute Integration

### Step 1: Import the Component
```tsx
import { ResizablePanel } from '@/components/ResizablePanel';
import '@/components/ResizablePanel/ResizablePanel.css';
```

### Step 2: Basic Usage
```tsx
function MyLayout() {
  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      <ResizablePanel
        initialWidth={300}
        minWidth={200}
        maxWidth={600}
      >
        <YourSidebarContent />
      </ResizablePanel>

      <main style={{ flex: 1 }}>
        <YourMainContent />
      </main>
    </div>
  );
}
```

## Common Patterns

### Pattern 1: Persistent Panel Size
```tsx
const [width, setWidth] = useState(() => {
  const saved = localStorage.getItem('sidebar-width');
  return saved ? Number(saved) : 300;
});

<ResizablePanel
  initialWidth={width}
  onResize={(w) => {
    setWidth(w);
    localStorage.setItem('sidebar-width', String(w));
  }}
>
```

### Pattern 2: Debounced Resize (Performance)
```tsx
<ResizablePanel
  initialWidth={300}
  resizeDebounce={150} // Wait 150ms after drag stops
  onResize={(w) => {
    // Expensive operation runs less frequently
    saveToServer(w);
  }}
>
```

### Pattern 3: Two-Panel Layout
```tsx
const [leftWidth, setLeftWidth] = useState(300);
const [rightWidth, setRightWidth] = useState(250);

<>
  <ResizablePanel position="right" initialWidth={leftWidth} onResize={setLeftWidth}>
    <LeftPanel />
  </ResizablePanel>

  <main style={{ flex: 1 }}>
    <MainContent />
  </main>

  <ResizablePanel position="left" initialWidth={rightWidth} onResize={setRightWidth}>
    <RightPanel />
  </ResizablePanel>
</>
```

### Pattern 4: Controlled Component
```tsx
const [isResizing, setIsResizing] = useState(false);

<ResizablePanel
  initialWidth={width}
  onResizeStart={() => setIsResizing(true)}
  onResizeEnd={() => setIsResizing(false)}
  className={isResizing ? 'resizing' : ''}
>
  {isResizing && <ResizeIndicator />}
</ResizablePanel>
```

## Props Cheatsheet

| Prop | Type | Default | Purpose |
|------|------|---------|---------|
| `initialWidth` | number | 300 | Starting width |
| `minWidth` | number | 200 | Minimum width |
| `maxWidth` | number | 800 | Maximum width |
| `position` | 'left' \| 'right' | 'left' | Handle position |
| `onResize` | (width) => void | - | Size change callback |
| `resizeDebounce` | number | 0 | Delay (ms) before callback |

## Troubleshooting

**Problem**: Panel doesn't resize
- **Solution**: Ensure parent has `display: flex` and defined height

**Problem**: Text gets selected while dragging
- **Solution**: Component handles this automatically, check for conflicting CSS

**Problem**: Resize feels slow
- **Solution**: Use `resizeDebounce={100}` for expensive callbacks

**Problem**: Handle doesn't appear
- **Solution**: Check `position` prop and ensure CSS is imported

## Customization

### Change Handle Color
```css
.resizable-panel-handle:hover {
  background: rgba(59, 130, 246, 0.1); /* Blue */
}
```

### Adjust Grip Style
```css
.grip-line {
  width: 4px;  /* Thicker dots */
  height: 4px;
  background: #3b82f6;  /* Blue color */
}
```

### Smooth Width Transition
```css
.resizable-panel {
  transition: width 0.2s ease;
}
```

## Best Practices

1. **Always set minWidth**: Prevent panel from becoming unusable
2. **Use debounce for expensive operations**: Save to localStorage, API calls
3. **Persist width**: Use localStorage for better UX
4. **Set reasonable maxWidth**: Don't let panel consume entire screen
5. **Test on mobile**: Ensure touch targets are large enough

## Live Demo

Run the example component:
```tsx
import { ResizablePanelExample } from '@/components/ResizablePanel/ResizablePanelExample';

<ResizablePanelExample />
```

## Full Documentation
See [README.md](./README.md) for complete documentation.

## Need Help?
Check the example files:
- `ResizablePanelExample.tsx` - Working demo
- `ResizablePanel.css` - Styling reference
- `types.ts` - TypeScript definitions
