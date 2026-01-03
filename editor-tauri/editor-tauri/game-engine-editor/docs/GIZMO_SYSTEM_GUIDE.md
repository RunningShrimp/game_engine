# 3D Gizmo System Implementation Guide

## Overview

This document describes the complete implementation of the 3D transformation Gizmo system for the Tauri Game Engine Editor.

## Architecture

### System Components

```
src/
├── gizmo/
│   ├── types.ts           # Type definitions
│   ├── GizmoRenderer.ts   # Rendering logic
│   ├── GizmoController.ts # Interaction controller
│   ├── SnapSystem.ts      # Grid snapping
│   └── index.ts           # Public exports
├── utils/
│   ├── math3d.ts          # 3D math utilities
│   └── raycast.ts         # Ray casting and picking
└── components/
    └── Viewport/
        └── Viewport.tsx   # Integrated viewport
```

## Features Implemented

### 1. Transform Modes

- **Translate (W)**: Move objects along X, Y, Z axes
- **Rotate (E)**: Rotate objects around X, Y, Z axes
- **Scale (R)**: Scale objects along X, Y, Z axes

### 2. Gizmo Types

#### Translate Gizmo
- **X Axis**: Red arrow (horizontal)
- **Y Axis**: Green arrow (vertical)
- **Z Axis**: Blue arrow (diagonal)
- **Screen Space**: White circle at center

#### Rotate Gizmo
- **X Axis**: Red arc
- **Y Axis**: Green arc
- **Z Axis**: Blue full circle
- **Screen Space**: White outer circle

#### Scale Gizmo
- **X Axis**: Red line with box
- **Y Axis**: Green line with box
- **Z Axis**: Blue line with box
- **Uniform**: White center box

### 3. Interaction Features

- **Hover Detection**: Visual feedback when hovering over gizmo elements
- **Click to Drag**: Click and drag to transform objects
- **Snap to Grid**: Optional grid snapping for precise positioning
- **World/Local Space**: Switch between coordinate systems
- **Visual Feedback**: Highlighting and cursor changes

### 4. Grid Snapping

Configurable snapping for:
- Translation (default: 1.0 unit)
- Rotation (default: 15 degrees)
- Scale (default: 0.1 units)

## API Reference

### GizmoController

Main controller for gizmo interactions.

```typescript
import { GizmoController } from './gizmo';

// Create controller
const controller = new GizmoController({
  snapEnabled: true,
  snapValue: 1.0
});

// Set transform mode
controller.setMode(TransformMode.Translate);

// Set coordinate space
controller.setSpace(Space.World);

// Handle mouse events
controller.handleMouseMove(mouseX, mouseY, position, camera, width, height);
controller.handleMouseDown(mouseX, mouseY, position, camera, width, height);
const result = controller.handleMouseUp();
```

### GizmoRenderer

Renders gizmo visualization using Canvas 2D.

```typescript
import { GizmoRenderer } from './gizmo';

const ctx = canvas.getContext('2d');
const renderer = new GizmoRenderer(ctx);

renderer.render(
  position,    // Target position
  state,       // Gizmo state
  camera,      // Camera object
  width,       // Viewport width
  height       // Viewport height
);
```

### SnapSystem

Manages grid snapping logic.

```typescript
import { SnapSystem } from './gizmo';

const snapSystem = new SnapSystem({
  enabled: true,
  translateValue: 1.0,
  rotateValue: 15.0,
  scaleValue: 0.1
});

// Snap values
const snapped = snapSystem.snapTranslation(vector3);
```

### Camera

Camera class for view and projection calculations.

```typescript
import { Camera } from './utils/raycast';

const camera = new Camera(
  position,  // Camera position
  target,    // Look-at target
  fov,       // Field of view
  aspect     // Aspect ratio
);

// Convert screen point to ray
const ray = camera.screenPointToRay(x, y, width, height);

// Convert world point to screen
const screen = camera.worldPointToScreen(worldPos, width, height);
```

## Usage Example

### Basic Setup

```typescript
import { Viewport } from './components/Viewport/Viewport';
import { TransformMode, Space } from './types/engine';

function App() {
  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);
  const [transformMode, setTransformMode] = useState(TransformMode.Translate);
  const [snapEnabled, setSnapEnabled] = useState(true);

  const handleEntityTransform = (entityId: string, transform: Partial<Vector3>) => {
    // Update entity transform
    console.log('Transformed', entityId, transform);
  };

  return (
    <Viewport
      entities={entities}
      selectedEntities={selectedEntities}
      transformMode={transformMode}
      space={Space.World}
      gridSize={10}
      snapEnabled={snapEnabled}
      snapValue={1.0}
      showGrid={true}
      showStats={true}
      onEntityTransform={handleEntityTransform}
    />
  );
}
```

### Custom Configuration

```typescript
// Customize gizmo appearance
const gizmoConfig = {
  size: 120,
  hoverSize: 140,
  snapEnabled: true,
  snapValue: 0.5,
  alpha: 0.9,
  hoverAlpha: 1.0,
  xAxisColor: '#FF4444',
  yAxisColor: '#44FF44',
  zAxisColor: '#4444FF',
  selectionColor: '#FFFF44',
  hoverColor: '#FFFFFF'
};

const controller = new GizmoController(gizmoConfig);
```

## Technical Implementation

### 1. Ray Casting

The system uses ray casting for 3D picking:

```typescript
// Create ray from mouse position
const ray = camera.screenPointToRay(mouseX, mouseY, width, height);

// Intersect with geometric primitives
const hit = BoundingBoxHelper.intersectRay(ray, boundingBox);
const sphereHit = SphereHelper.intersectRay(ray, sphere);
```

### 2. Hit Testing

Hit testing is performed in screen space for better performance:

```typescript
private hitTestTranslate(
  mouseX: number,
  mouseY: number,
  screenPos: { x: number; y: number },
  size: number,
  threshold: number
): GizmoAxis | null {
  // Calculate distance to each axis
  const xDist = this.pointToLineDistance(mouseX, mouseY, ...);
  const yDist = this.pointToLineDistance(mouseX, mouseY, ...);
  const zDist = this.pointToLineDistance(mouseX, mouseY, ...);

  // Return axis if within threshold
  if (xDist < threshold) return GizmoAxis.X;
  // ... etc
}
```

### 3. Transform Calculation

Transforms are calculated from mouse delta:

```typescript
private handleDrag(
  mouseX: number,
  mouseY: number,
  targetPosition: Vector3,
  camera: Camera,
  viewportWidth: number,
  viewportHeight: number
): GizmoAxis | null {
  const mouseDelta = {
    x: mouseX - this.dragStartMouse.x,
    y: mouseY - this.dragStartMouse.y
  };

  // Calculate delta based on active axis
  let delta = Vector3.zero;
  switch (this.state.activeAxis) {
    case GizmoAxis.X:
      delta = new Vector3(mouseDelta.x * 0.01, 0, 0);
      break;
    // ... other axes
  }

  // Apply snapping
  delta = this.snapSystem.snapDelta(delta, 'translate');

  this.state.delta = delta;
  this.state.currentPosition = this.state.startPosition.add(delta);

  return this.state.activeAxis;
}
```

### 4. Rendering Loop

The gizmo is rendered in a 60FPS loop:

```typescript
const render = (currentTime: number) => {
  // Clear canvas
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Get selected entity
  const selectedEntity = entities.find(e =>
    selectedEntities.includes(e.id)
  );

  // Render gizmo if entity is selected
  if (selectedEntity) {
    const position = new Vector3(
      selectedEntity.transform.position.x,
      selectedEntity.transform.position.y,
      selectedEntity.transform.position.z
    );

    const state = gizmoController.getState();
    gizmoRenderer.render(position, state, camera, width, height);
  }

  requestAnimationFrame(render);
};
```

## Performance Optimizations

1. **Canvas 2D Rendering**: Efficient 2D drawing instead of full 3D rendering
2. **Screen-Space Hit Testing**: Faster than 3D ray casting for simple gizmos
3. **RequestAnimationFrame**: Smooth 60FPS rendering
4. **Lazy Initialization**: Gizmo only created when needed
5. **State Caching**: Reuse calculated values between frames

## Future Enhancements

### Planned Features

1. **WebGPU Rendering**: True 3D gizmo rendering
2. **Multi-Selection**: Transform multiple objects
3. **Pivot Points**: Custom transformation origins
4. **Visual Indicators**: Snap indicators and measurement tools
5. **Undo/Redo**: Transform history
6. **Keyboard Shortcuts**: Quick mode switching
7. **Custom Gizmos**: Pluggable gizmo system

### Performance Improvements

1. **Instanced Rendering**: Batch multiple gizmos
2. **Level of Detail**: Simpler gizmos at distance
3. **Culling**: Don't render off-screen gizmos
4. **GPU Acceleration**: Shader-based rendering

## Troubleshooting

### Gizmo Not Appearing

- Check that an entity is selected
- Verify canvas context is initialized
- Check camera position and target

### Hit Detection Not Working

- Verify mouse coordinates are correct
- Check hit threshold values
- Ensure canvas size is properly set

### Snapping Not Applied

- Verify `snapEnabled` is true
- Check `snapValue` is greater than 0
- Ensure snap system is properly initialized

## Integration Checklist

- [x] Type definitions
- [x] 3D math utilities
- [x] Ray casting system
- [x] Gizmo renderer
- [x] Gizmo controller
- [x] Snap system
- [x] Viewport integration
- [x] Mouse event handling
- [x] Visual feedback
- [x] Performance monitoring

## Testing

Manual testing checklist:

1. **Translation**
   - [ ] X axis movement
   - [ ] Y axis movement
   - [ ] Z axis movement
   - [ ] Screen space movement
   - [ ] Snap to grid

2. **Rotation**
   - [ ] X axis rotation
   - [ ] Y axis rotation
   - [ ] Z axis rotation
   - [ ] Screen space rotation
   - [ ] Angle snapping

3. **Scaling**
   - [ ] X axis scaling
   - [ ] Y axis scaling
   - [ ] Z axis scaling
   - [ ] Uniform scaling
   - [ ] Scale snapping

4. **Interaction**
   - [ ] Hover detection
   - [ ] Click to select
   - [ ] Drag to transform
   - [ ] Release to apply
   - [ ] Cursor feedback

## License

This implementation is part of the Game Engine Editor project.

## Author

Generated for the Tauri Game Engine Editor project.

## Version

1.0.0 - Initial implementation
- Basic gizmo rendering
- Transform modes (Translate, Rotate, Scale)
- Grid snapping
- Mouse interaction
- Viewport integration
