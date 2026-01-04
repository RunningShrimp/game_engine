# Toolbar Component

A comprehensive, atomically-designed toolbar component for the game engine editor.

## Architecture

The Toolbar component follows the Atomic Design methodology, composed of reusable sub-components:

```
Toolbar (Organism)
├── HistoryControls (Organism)
│   └── ToolbarButton (Molecule)
│       └── Icon (Atom)
├── ClipboardControls (Organism)
│   └── ToolbarButton (Molecule)
│       └── Icon (Atom)
├── TransformControls (Organism)
│   └── Custom Buttons
│       └── Icon (Atom)
├── SpaceControls (Organism)
│   └── Custom Buttons
└── PlaybackControls (Organism)
    └── Custom Buttons
        └── Icon (Atom)
```

## Components

### Main Component

- **Toolbar**: Main container that orchestrates all toolbar sections

### Sub-Components (Organisms)

1. **HistoryControls**
   - Undo/Redo functionality
   - Disabled state handling
   - Location: `./HistoryControls/index.tsx`

2. **ClipboardControls**
   - Copy/Paste functionality
   - Disabled state when no clipboard data
   - Location: `./ClipboardControls/index.tsx`

3. **TransformControls**
   - Transform mode selection (Translate, Rotate, Scale)
   - Active state highlighting
   - Location: `./TransformControls/index.tsx`

4. **SpaceControls**
   - World/Local space toggle
   - Grid snap toggle
   - Location: `./SpaceControls/index.tsx`

5. **PlaybackControls**
   - Play/Pause/Stop functionality
   - State-based enabling/disabling
   - Location: `./PlaybackControls/index.tsx`

### Supporting Components (Molecules)

- **ToolbarGroup**: Groups buttons with optional divider
- **ToolbarButton**: Reusable button with icon support
- Location: `../../molecules/`

### Atoms

- **Icon**: SVG icon wrapper
- **Divider**: Visual separator
- Location: `../../atoms/`

## Usage

### Basic Usage

```tsx
import { Toolbar } from '@/components/organisms';

function App() {
  return (
    <Toolbar
      transformMode={TransformMode.Translate}
      space={Space.World}
      isPlaying={false}
      isPaused={false}
      snapEnabled={true}
      canUndo={true}
      canRedo={false}
      copiedEntity={null}
      onTransformModeChange={(mode) => console.log(mode)}
      onSpaceChange={(space) => console.log(space)}
      onPlay={() => console.log('play')}
      onPause={() => console.log('pause')}
      onStop={() => console.log('stop')}
      onSnapToggle={() => console.log('snap toggle')}
      onUndo={() => console.log('undo')}
      onRedo={() => console.log('redo')}
      onCopy={() => console.log('copy')}
      onPaste={() => console.log('paste')}
    />
  );
}
```

### Using Individual Sub-Components

```tsx
import { HistoryControls, PlaybackControls } from '@/components/organisms';

function CustomToolbar() {
  return (
    <div className="flex gap-4">
      <HistoryControls
        canUndo={true}
        canRedo={false}
        onUndo={handleUndo}
        onRedo={handleRedo}
      />
      <PlaybackControls
        isPlaying={false}
        isPaused={false}
        onPlay={handlePlay}
        onPause={handlePause}
        onStop={handleStop}
      />
    </div>
  );
}
```

## Props

### Toolbar Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `transformMode` | `TransformMode` | Yes | Current transform mode |
| `space` | `Space` | Yes | Current coordinate space |
| `isPlaying` | `boolean` | Yes | Whether game is playing |
| `isPaused` | `boolean` | Yes | Whether game is paused |
| `snapEnabled` | `boolean` | Yes | Whether grid snap is enabled |
| `canUndo` | `boolean` | Yes | Whether undo is available |
| `canRedo` | `boolean` | Yes | Whether redo is available |
| `copiedEntity` | `Entity \| null` | Yes | Currently copied entity |
| `onTransformModeChange` | `(mode: TransformMode) => void` | Yes | Transform mode change handler |
| `onSpaceChange` | `(space: Space) => void` | Yes | Space change handler |
| `onPlay` | `() => void` | Yes | Play handler |
| `onPause` | `() => void` | Yes | Pause handler |
| `onStop` | `() => void` | Yes | Stop handler |
| `onSnapToggle` | `() => void` | Yes | Snap toggle handler |
| `onUndo` | `() => void` | Yes | Undo handler |
| `onRedo` | `() => void` | Yes | Redo handler |
| `onCopy` | `() => void` | Yes | Copy handler |
| `onPaste` | `() => void` | Yes | Paste handler |
| `className` | `string` | No | Additional CSS classes |

## Styling

The toolbar uses Tailwind CSS utility classes:

- Base: `bg-slate-800 border-b border-slate-700`
- Layout: `flex items-center justify-between`
- Spacing: `px-4 py-2`

Custom styles can be passed via the `className` prop.

## Keyboard Shortcuts

The toolbar supports the following keyboard shortcuts (handled by parent):

- **Ctrl+Z**: Undo
- **Ctrl+Shift+Z**: Redo
- **Ctrl+C**: Copy
- **Ctrl+V**: Paste
- **W**: Translate mode
- **E**: Rotate mode
- **R**: Scale mode
- **Ctrl+P**: Play/Pause

## Accessibility

- All buttons include `title` attributes for tooltips
- Disabled buttons are properly marked with `disabled` attribute
- Icons use appropriate ARIA labels
- Keyboard navigation support via native button elements

## Examples

### Custom Styled Toolbar

```tsx
<Toolbar
  {...props}
  className="bg-slate-900 border-slate-600"
/>
```

### With Custom Section Ordering

You can compose the toolbar differently by importing individual components:

```tsx
import {
  PlaybackControls,
  TransformControls,
  SpaceControls
} from '@/components/organisms';

function ReversedToolbar() {
  return (
    <div className="flex justify-between">
      <PlaybackControls {...playbackProps} />
      <TransformControls {...transformProps} />
      <SpaceControls {...spaceProps} />
    </div>
  );
}
```

## Testing

Each sub-component can be tested independently:

```tsx
import { render, screen } from '@testing-library/react';
import { HistoryControls } from '@/components/organisms';

describe('HistoryControls', () => {
  it('should disable undo when canUndo is false', () => {
    render(
      <HistoryControls
        canUndo={false}
        canRedo={true}
        onUndo={jest.fn()}
        onRedo={jest.fn()}
      />
    );
    const undoButton = screen.getByTitle('Undo (Ctrl+Z)');
    expect(undoButton).toBeDisabled();
  });
});
```

## Future Enhancements

- [ ] Add customizable button layouts
- [ ] Support for toolbar customization/drag-and-drop
- [ ] Add more transform modes (e.g., Bounds)
- [ ] Tooltips for all buttons
- [ ] Keyboard shortcut display in tooltips
- [ ] Animation mode controls
- [ ] Bookmark/favorite locations

## Related Components

- `StatusBar`: Display editor state information
- `PropertyInspector`: Show entity properties
- `EntityTree`: Display scene hierarchy
