# Game Engine Editor API Documentation

This document describes all Tauri commands exposed to the frontend and their usage.

## Table of Contents

- [Engine Management](#engine-management)
- [Entity Operations](#entity-operations)
- [Component Editing](#component-editing)
- [Scene Control](#scene-control)
- [Interaction](#interaction)
- [Asset Management](#asset-management)
- [Scene Persistence](#scene-persistence)
- [Logging](#logging)

## Engine Management

### create_engine

Initialize the game engine instance.

**Command**: `create_engine`

**Parameters**: None

**Returns**: `Promise<string>`

**Example**:
```typescript
const result = await invoke<string>('create_engine');
console.log(result); // "Engine created: <uuid>"
```

**Errors**:
- Throws if engine initialization fails

---

## Entity Operations

### get_entities

Get the hierarchical list of all entities in the scene.

**Command**: `get_entities`

**Parameters**: None

**Returns**: `Promise<EntityData[]>`

**EntityData Interface**:
```typescript
interface EntityData {
  id: number;
  name: string;
  children: EntityData[];
}
```

**Example**:
```typescript
const entities = await invoke<EntityData[]>('get_entities');
entities.forEach(entity => {
  console.log(`Entity: ${entity.name} (${entity.id})`);
});
```

---

### create_entity

Create a new entity in the scene.

**Command**: `create_entity`

**Parameters**:
- `name: string` - Name of the new entity
- `parent_id: number | null` - Parent entity ID (null for root-level)

**Returns**: `Promise<number>` - ID of the created entity

**Example**:
```typescript
const entityId = await invoke<number>('create_entity', {
  name: 'MyEntity',
  parentId: null
});
console.log(`Created entity with ID: ${entityId}`);
```

---

### delete_entity

Delete an entity from the scene.

**Command**: `delete_entity`

**Parameters**:
- `entity_id: number` - ID of the entity to delete

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('delete_entity', { entityId: 123 });
```

---

## Component Editing

### get_entity_components

Get all components attached to an entity.

**Command**: `get_entity_components`

**Parameters**:
- `entity_id: number` - Entity ID

**Returns**: `Promise<ComponentData[]>`

**ComponentData Interface**:
```typescript
type ComponentData =
  | { type: 'Transform'; position: [number, number, number]; rotation: [number, number, number]; scale: [number, number, number] }
  | { type: 'Mesh'; mesh_path: string }
  | { type: 'Material'; material_path: string }
  | { type: 'Light'; light_type: string; color: [number, number, number]; intensity: number }
  | { type: 'Camera'; fov: number; near: number; far: number }
  | { type: 'RigidBody'; body_type: string; mass: number }
  | { type: 'Collider'; collider_type: string; size: [number, number, number] };
```

**Example**:
```typescript
const components = await invoke<ComponentData[]>('get_entity_components', {
  entityId: 123
});

components.forEach(comp => {
  if (comp.type === 'Transform') {
    console.log('Position:', comp.position);
  }
});
```

---

### update_component

Update a component's data.

**Command**: `update_component`

**Parameters**:
- `entity_id: number` - Entity ID
- `component: ComponentData` - Component data to update

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('update_component', {
  entityId: 123,
  component: {
    type: 'Transform',
    position: [1, 2, 3],
    rotation: [0, 0, 0],
    scale: [1, 1, 1]
  }
});
```

---

### update_transform

Convenience method to update an entity's transform component.

**Command**: `update_transform`

**Parameters**:
- `entity_id: number` - Entity ID
- `position: [number, number, number]` - X, Y, Z position
- `rotation: [number, number, number]` - X, Y, Z rotation (radians)
- `scale: [number, number, number]` - X, Y, Z scale

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('update_transform', {
  entityId: 123,
  position: [10, 0, 5],
  rotation: [0, Math.PI / 2, 0],
  scale: [2, 2, 2]
});
```

---

## Scene Control

### play_scene

Start the scene simulation.

**Command**: `play_scene`

**Parameters**: None

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('play_scene');
```

---

### pause_scene

Pause the scene simulation.

**Command**: `pause_scene`

**Parameters**: None

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('pause_scene');
```

---

### stop_scene

Stop the scene simulation and reset to initial state.

**Command**: `stop_scene`

**Parameters**: None

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('stop_scene');
```

---

## Interaction

### raycast

Perform a raycast from the camera to select entities.

**Command**: `raycast`

**Parameters**:
- `x: number` - Normalized X coordinate (0-1)
- `y: number` - Normalized Y coordinate (0-1)

**Returns**: `Promise<number | null>` - Entity ID or null if no hit

**Example**:
```typescript
const canvas = canvasRef.current;
const rect = canvas.getBoundingClientRect();
const x = (event.clientX - rect.left) / canvas.width;
const y = (event.clientY - rect.top) / canvas.height;

const entityId = await invoke<number | null>('raycast', { x, y });
if (entityId !== null) {
  console.log(`Hit entity: ${entityId}`);
}
```

---

## Asset Management

### get_assets

Get the list of assets in the project.

**Command**: `get_assets`

**Parameters**:
- `asset_type: string | null` - Filter by asset type (null for all)

**Asset Types**: `"model"`, `"texture"`, `"material"`, `"audio"`, `"scene"`

**Returns**: `Promise<AssetData[]>`

**AssetData Interface**:
```typescript
interface AssetData {
  path: string;
  name: string;
  asset_type: string;
  thumbnail?: string;
}
```

**Example**:
```typescript
// Get all assets
const allAssets = await invoke<AssetData[]>('get_assets', { assetType: null });

// Get only models
const models = await invoke<AssetData[]>('get_assets', { assetType: 'model' });
```

---

### import_asset

Import an asset file into the project.

**Command**: `import_asset`

**Parameters**:
- `source_path: string` - Path to the source file
- `asset_type: string` - Type of asset

**Returns**: `Promise<string>` - Path to the imported asset

**Example**:
```typescript
import { open } from '@tauri-apps/api/dialog';

const selected = await open({
  multiple: false,
  filters: [
    {
      name: '3D Model',
      extensions: ['glb', 'gltf']
    }
  ]
});

if (selected && typeof selected === 'string') {
  const assetPath = await invoke<string>('import_asset', {
    sourcePath: selected,
    assetType: 'model'
  });
  console.log(`Imported: ${assetPath}`);
}
```

---

## Scene Persistence

### save_scene

Save the current scene to a file.

**Command**: `save_scene`

**Parameters**:
- `scene_path: string` - Path where to save the scene

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('save_scene', {
  scenePath: '/scenes/my_scene.scene'
});
```

---

### load_scene

Load a scene from a file.

**Command**: `load_scene`

**Parameters**:
- `scene_path: string` - Path to the scene file

**Returns**: `Promise<void>`

**Example**:
```typescript
await invoke('load_scene', {
  scenePath: '/scenes/my_scene.scene'
});
```

---

## Logging

### get_console_logs

Get console log entries.

**Command**: `get_console_logs`

**Parameters**:
- `limit: number | null` - Maximum number of logs to retrieve (null for all)

**Returns**: `Promise<ConsoleLog[]>`

**ConsoleLog Interface**:
```typescript
interface ConsoleLog {
  level: string;    // 'error', 'warning', 'info', 'debug'
  message: string;
  timestamp: number; // Unix timestamp
}
```

**Example**:
```typescript
// Get last 100 logs
const logs = await invoke<ConsoleLog[]>('get_console_logs', { limit: 100 });

logs.forEach(log => {
  const time = new Date(log.timestamp * 1000).toLocaleTimeString();
  console.log(`[${time}] ${log.level.toUpperCase()}: ${log.message}`);
});
```

---

## Error Handling

All commands can throw errors. Always wrap invocations in try-catch:

```typescript
try {
  await invoke('some_command', { /* params */ });
} catch (error) {
  console.error('Command failed:', error);
  // Show error to user
}
```

---

## TypeScript Type Definitions

Create a `types.ts` file to reuse types:

```typescript
// types.ts
export interface EntityData {
  id: number;
  name: string;
  children: EntityData[];
}

export type ComponentData =
  | { type: 'Transform'; position: [number, number, number]; rotation: [number, number, number]; scale: [number, number, number] }
  | { type: 'Mesh'; mesh_path: string }
  | { type: 'Material'; material_path: string }
  | { type: 'Light'; light_type: string; color: [number, number, number]; intensity: number }
  | { type: 'Camera'; fov: number; near: number; far: number }
  | { type: 'RigidBody'; body_type: string; mass: number }
  | { type: 'Collider'; collider_type: string; size: [number, number, number] };

export interface AssetData {
  path: string;
  name: string;
  asset_type: string;
  thumbnail?: string;
}

export interface ConsoleLog {
  level: string;
  message: string;
  timestamp: number;
}
```

---

## Advanced Usage

### Batch Operations

For performance, batch multiple operations:

```typescript
// Instead of multiple individual calls
await invoke('update_transform', { entityId: 1, position: [0, 0, 0], rotation: [0, 0, 0], scale: [1, 1, 1] });
await invoke('update_transform', { entityId: 2, position: [1, 0, 0], rotation: [0, 0, 0], scale: [1, 1, 1] });

// Consider implementing batch commands in the backend
// await invoke('update_transforms_batch', { updates: [...] });
```

### Event Listening

(TODO: Implement event system for real-time updates)

```typescript
// Future: Listen to engine events
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('entity-updated', (event) => {
  console.log('Entity updated:', event.payload);
});

// Cleanup
unlisten();
```

---

## Best Practices

1. **Type Safety**: Always use TypeScript types
2. **Error Handling**: Wrap all `invoke()` calls in try-catch
3. **Debouncing**: Debounce user inputs (e.g., transform edits)
4. **Loading States**: Show loading indicators during long operations
5. **Caching**: Cache entity/component data to avoid redundant calls

---

## Version History

### v0.1.0 (2024-12-31)
- Initial API
- Basic CRUD operations for entities and components
- Scene playback controls
- Asset browsing
- Console logging

---

**For more information, see [README.md](./README.md)**
