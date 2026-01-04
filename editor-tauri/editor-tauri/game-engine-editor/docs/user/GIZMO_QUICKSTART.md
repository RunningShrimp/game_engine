# 3D Gizmo System - Quick Start Guide

## 5-Minute Setup

The 3D Gizmo system is already integrated into your Viewport component. Here's how to use it:

## Basic Usage

### 1. Import the Viewport Component

```typescript
import { Viewport } from './components/Viewport/Viewport';
import { TransformMode, Space } from './types/engine';
```

### 2. Set Up State

```typescript
const [selectedEntities, setSelectedEntities] = useState<string[]>([]);
const [transformMode, setTransformMode] = useState(TransformMode.Translate);
const [snapEnabled, setSnapEnabled] = useState(true);
```

### 3. Render the Viewport

```typescript
<Viewport
  entities={entities}
  selectedEntities={selectedEntities}
  transformMode={transformMode}
  space={Space.World}
  snapEnabled={snapEnabled}
  snapValue={1.0}
  showGrid={true}
  showStats={true}
  onEntityTransform={(entityId, transform) => {
    // Update your entity here
    console.log('Entity transformed:', entityId, transform);
  }}
/>
```

## Controls

### Mouse Controls
- **Click & Drag Axis**: Transform along that axis
- **Hover**: Highlight gizmo elements
- **Release**: Apply transformation

### Keyboard Shortcuts
- **W**: Switch to Translate mode
- **E**: Switch to Rotate mode
- **R**: Switch to Scale mode

### Visual Indicators
- **Red Arrow**: X axis
- **Green Arrow**: Y axis
- **Blue Arrow**: Z axis
- **White Circle**: Screen-space transformation

## Features

### Transform Modes

#### Translate (W)
Move objects in 3D space
- Click and drag arrows to move along axes
- Click center circle for screen-space movement

#### Rotate (E)
Rotate objects around axes
- Click and drag arcs to rotate
- Click outer circle for screen-space rotation

#### Scale (R)
Resize objects
- Click and drag axes to scale along axes
- Click center box for uniform scaling

### Grid Snapping

Enable snapping for precise positioning:

```typescript
<Viewport
  snapEnabled={true}
  snapValue={1.0}  // Snap to 1 unit increments
  // ... other props
/>
```

Snap values:
- **Translation**: 1.0 unit (default)
- **Rotation**: 15 degrees (default)
- **Scale**: 0.1 units (default)

## Integration Example

```typescript
import React, { useState } from 'react';
import { Viewport } from './components/Viewport/Viewport';
import { TransformMode, Space } from './types/engine';

function Editor() {
  const [entities, setEntities] = useState([
    {
      id: 'cube-1',
      name: 'Cube',
      transform: {
        position: { x: 0, y: 0, z: 0 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 }
      },
      components: [],
      children: [],
      visible: true,
      locked: false
    }
  ]);

  const [selectedId, setSelectedId] = useState('cube-1');
  const [mode, setMode] = useState(TransformMode.Translate);

  return (
    <div style={{ width: '100%', height: '100vh' }}>
      {/* Toolbar */}
      <div>
        <button onClick={() => setMode(TransformMode.Translate)}>
          Translate (W)
        </button>
        <button onClick={() => setMode(TransformMode.Rotate)}>
          Rotate (E)
        </button>
        <button onClick={() => setMode(TransformMode.Scale)}>
          Scale (R)
        </button>
      </div>

      {/* Viewport */}
      <Viewport
        entities={entities}
        selectedEntities={[selectedId]}
        transformMode={mode}
        space={Space.World}
        snapEnabled={true}
        snapValue={1.0}
        onEntityTransform={(id, transform) => {
          setEntities(prev =>
            prev.map(entity =>
              entity.id === id
                ? {
                    ...entity,
                    transform: {
                      ...entity.transform,
                      position: {
                        ...entity.transform.position,
                        ...transform
                      }
                    }
                  }
                : entity
            )
          );
        }}
      />
    </div>
  );
}
```

## Common Patterns

### Toggle Snap
```typescript
const [snap, setSnap] = useState(true);

<button onClick={() => setSnap(!snap)}>
  {snap ? 'Snap: ON' : 'Snap: OFF'}
</button>
```

### Change Snap Value
```typescript
const [snapValue, setSnapValue] = useState(1.0);

<select
  value={snapValue}
  onChange={(e) => setSnapValue(parseFloat(e.target.value))}
>
  <option value={0.1}>0.1</option>
  <option value={0.5}>0.5</option>
  <option value={1.0}>1.0</option>
  <option value={2.0}>2.0</option>
</select>
```

### Mode Switching
```typescript
const handleKeyDown = (e: KeyboardEvent) => {
  switch (e.key.toLowerCase()) {
    case 'w': setMode(TransformMode.Translate); break;
    case 'e': setMode(TransformMode.Rotate); break;
    case 'r': setMode(TransformMode.Scale); break;
  }
};

useEffect(() => {
  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
}, []);
```

## Customization

### Change Gizmo Colors
```typescript
import { GizmoController } from './gizmo';

const controller = new GizmoController({
  xAxisColor: '#FF0000',
  yAxisColor: '#00FF00',
  zAxisColor: '#0000FF',
  hoverColor: '#FFFFFF',
  selectionColor: '#FFFF00'
});
```

### Adjust Gizmo Size
```typescript
const controller = new GizmoController({
  size: 100,        // Normal size
  hoverSize: 120    // Size when hovering
});
```

## Troubleshooting

### Gizmo Not Showing
- Ensure an entity is selected
- Check that entities array is not empty
- Verify selectedEntities contains valid IDs

### Transform Not Working
- Check onEntityTransform callback is provided
- Verify entityId matches an entity in the array
- Ensure transform object has correct structure

### Performance Issues
- Reduce number of entities
- Disable stats display: `showStats={false}`
- Disable grid overlay: `showGrid={false}`

## Next Steps

1. Read the full guide: `docs/GIZMO_SYSTEM_GUIDE.md`
2. Check examples: `examples/GIZMO_USAGE_EXAMPLE.tsx`
3. Explore API: Inline documentation in source files

## Support

For issues or questions:
- Check documentation: `docs/GIZMO_SYSTEM_GUIDE.md`
- Review implementation: `src/gizmo/`
- Examine examples: `examples/GIZMO_USAGE_EXAMPLE.tsx`

---

**Version**: 1.0.0
**Last Updated**: January 2, 2026
