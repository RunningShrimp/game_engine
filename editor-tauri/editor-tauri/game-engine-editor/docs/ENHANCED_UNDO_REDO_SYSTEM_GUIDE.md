# Enhanced Undo/Redo System - Complete Guide

## Overview

The enhanced undo/redo system provides advanced history management capabilities including batch operations, transactions, macros, branching, bookmarks, search, and persistence.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Advanced Commands](#advanced-commands)
3. [History Persistence](#history-persistence)
4. [Bookmarks and Branches](#bookmarks-and-branches)
5. [Search and Filter](#search-and-filter)
6. [UI Components](#ui-components)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

## Basic Usage

### Setup

```typescript
import { HistoryManager } from './utils/HistoryManager';
import { HistoryPersistence } from './utils/HistoryPersistence';

// Create history manager with persistence
const persistence = new HistoryPersistence({
  enabled: true,
  autoSave: true,
  autoSaveInterval: 60000, // 1 minute
  storageType: 'localStorage',
  compress: true,
});

const historyManager = new HistoryManager(100, persistence);
```

### Basic Commands

```typescript
import { CreateEntityCommand, DeleteEntityCommand } from './types/commands';

// Execute a command
const createCmd = new CreateEntityCommand(
  entityId,
  entity,
  addEntity,
  removeEntity
);
await historyManager.executeCommand(createCmd);

// Undo
await historyManager.undo();

// Redo
await historyManager.redo();

// Check availability
if (historyManager.canUndo()) {
  await historyManager.undo();
}
```

### Subscribe to History Changes

```typescript
const unsubscribe = historyManager.subscribe((state) => {
  console.log('Can undo:', state.canUndo);
  console.log('Can redo:', state.canRedo);
  console.log('Bookmarks:', state.bookmarks.length);
  console.log('Branches:', state.branches.length);
});

// Unsubscribe when done
unsubscribe();
```

## Advanced Commands

### Batch Operations

Execute multiple commands together:

```typescript
import { BatchCommand } from './commands';

const commands = [
  new CreateEntityCommand(id1, entity1, add, remove),
  new CreateEntityCommand(id2, entity2, add, remove),
  new CreateEntityCommand(id3, entity3, add, remove),
];

// Sequential execution (default)
const batch = new BatchCommand(commands, { mode: 'sequential' });
await historyManager.executeCommand(batch);

// Or use the helper method
await historyManager.executeBatch(commands, 'sequential');

// Parallel execution
await historyManager.executeBatch(commands, 'parallel');
```

### Transactions

Execute commands with automatic rollback on failure:

```typescript
import { TransactionCommand } from './commands';

const commands = [
  new DeleteEntityCommand(entity1, remove, add),
  new DeleteEntityCommand(entity2, remove, add),
];

const transaction = new TransactionCommand(commands, {
  rollbackOnFailure: true,
  stopOnFirstError: true,
});

await historyManager.executeCommand(transaction);

// Or use the helper method
await historyManager.executeTransaction(commands, {
  rollbackOnFailure: true,
  stopOnFirstError: true,
});
```

### Macro Commands

Create reusable command sequences:

```typescript
import { MacroCommand } from './commands';

// Define a macro
const setupSceneMacro = new MacroCommand(
  'Setup Scene',
  [
    new CreateEntityCommand('light1', light, add, remove),
    new CreateEntityCommand('camera1', camera, add, remove),
    new CreateEntityCommand('ground1', ground, add, remove),
  ],
  {
    lightPosition: { x: 0, y: 10, z: 0 },
    cameraPosition: { x: 5, y: 5, z: 5 },
  }
);

await historyManager.executeMacro('Setup Scene', setupSceneMacro.commands);

// Execute with custom parameters
await historyManager.executeMacro('Setup Scene', commands, {
  lightPosition: { x: 0, y: 15, z: 0 },
});
```

### Conditional Commands

Execute commands based on conditions:

```typescript
import { ConditionalCommand } from './commands';

const condition = () => {
  return entityCount < 100;
};

const thenCommand = new CreateEntityCommand(id, entity, add, remove);
const elseCommand = new ShowWarningCommand('Maximum entities reached');

const conditional = new ConditionalCommand(
  condition,
  thenCommand,
  elseCommand
);

await historyManager.executeCommand(conditional);
```

### Delayed Commands

Execute commands after a delay:

```typescript
import { DelayedCommand } from './commands';

const command = new CreateEntityCommand(id, entity, add, remove);
const delayed = new DelayedCommand(command, 5000); // 5 seconds

await historyManager.executeCommand(delayed);

// Or schedule for specific time
import { scheduleCommandAt } from './commands';

const executeAt = new Date('2024-01-01T12:00:00');
const scheduled = scheduleCommandAt(command, executeAt);
await historyManager.executeCommand(scheduled);
```

## History Persistence

### Save and Load

```typescript
// Manual save
await historyManager.save();

// Manual load
await historyManager.load();

// Auto-save (configured in constructor)
const persistence = new HistoryPersistence({
  enabled: true,
  autoSave: true,
  autoSaveInterval: 60000,
});
```

### Export and Import

```typescript
// Export history as JSON
const blob = await historyManager.export('json');
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = `history_${Date.now()}.json`;
a.click();

// Import history
const input = document.createElement('input');
input.type = 'file';
input.accept = '.json';
input.onchange = async (e) => {
  const file = e.target.files[0];
  await historyManager.import(file);
};
input.click();
```

## Bookmarks and Branches

### Bookmarks

Create bookmarks to mark important points in history:

```typescript
// Create a bookmark
const bookmark = historyManager.createBookmark(
  'Initial Scene Setup',
  'Scene with basic lighting and camera',
  ['setup', 'initial']
);

// Get all bookmarks
const bookmarks = historyManager.getBookmarks();

// Jump to a bookmark
await historyManager.jumpToBookmark(bookmark.id);

// Delete a bookmark
historyManager.deleteBookmark(bookmark.id);
```

### Branches

Create parallel timelines:

```typescript
// Create a new branch
const experimentalBranch = historyManager.createBranch(
  'experimental',
  'main' // parent branch
);

// Switch to a branch
await historyManager.switchBranch(experimentalBranch.id);

// Get all branches
const branches = historyManager.getBranches();

// Delete a branch
historyManager.deleteBranch(branchId);
```

## Search and Filter

### Search

```typescript
// Search commands and bookmarks
const results = historyManager.search('entity');

console.log(`Found ${results.totalResults} results in ${results.searchTime}ms`);
console.log('Commands:', results.commands);
console.log('Bookmarks:', results.bookmarks);
```

### Filter

```typescript
import { HistoryFilterOptions } from './types/history';

const options: HistoryFilterOptions = {
  dateRange: {
    start: new Date('2024-01-01'),
    end: new Date('2024-12-31'),
  },
  commandTypes: ['CreateEntityCommand', 'DeleteEntityCommand'],
  tags: ['scene', 'entity'],
  textQuery: 'light',
};

const filtered = historyManager.filter(options);
```

### Compare States

```typescript
const diff = historyManager.compare(stateId1, stateId2);

console.log('Added commands:', diff.added);
console.log('Removed commands:', diff.removed);
console.log('Modified commands:', diff.modified);
```

## UI Components

### History Panel

```typescript
import { HistoryPanel } from './components/HistoryPanel';

function App() {
  const [showHistory, setShowHistory] = useState(false);

  return (
    <div>
      <button onClick={() => setShowHistory(!showHistory)}>
        Toggle History
      </button>

      {showHistory && (
        <HistoryPanel
          historyManager={historyManager}
          onClose={() => setShowHistory(false)}
        />
      )}
    </div>
  );
}
```

### Custom Components

```typescript
import {
  HistoryTimeline,
  HistoryBookmarkList,
  HistorySearch,
  HistoryStatistics,
} from './components/HistoryPanel';

// Timeline view
<HistoryTimeline
  events={historyManager.getTimelineEvents()}
  currentBranch={historyManager.getBranches().find(b => b.isActive)?.id || 'main'}
  onJumpTo={(stateId) => console.log('Jump to:', stateId)}
/>

// Bookmarks
<HistoryBookmarkList
  bookmarks={historyManager.getBookmarks()}
  selectedBookmark={null}
  onSelect={(bookmark) => console.log('Selected:', bookmark)}
  onJumpTo={async (bookmark) => {
    await historyManager.jumpToBookmark(bookmark.id);
  }}
/>

// Statistics
<HistoryStatistics
  statistics={historyManager.getStatistics()}
/>
```

## Best Practices

### 1. Command Design

- Keep commands focused and single-purpose
- Store all necessary state in the command
- Implement proper undo/redo logic
- Add descriptive names and metadata

```typescript
class MoveEntityCommand implements Command {
  id = `move_${Date.now()}`;
  timestamp = new Date();
  description = `Move entity ${entityId}`;

  metadata = {
    tags: ['transform', 'entity'],
    entityId,
  };

  constructor(
    private entityId: string,
    private from: Vector3,
    private to: Vector3,
    private updatePosition: (id: string, pos: Vector3) => void
  ) {}

  async execute() {
    this.updatePosition(this.entityId, this.to);
  }

  async undo() {
    this.updatePosition(this.entityId, this.from);
  }

  async redo() {
    this.execute();
  }
}
```

### 2. Batch Operations

- Group related commands into batches
- Use sequential mode for dependent commands
- Use parallel mode for independent commands

```typescript
// Good: Related operations batched together
const transformChanges = entities.map(entity =>
  new TransformEntityCommand(entity.id, oldTransform, newTransform, updateTransform)
);
await historyManager.executeBatch(transformChanges, 'parallel');

// Good: Dependent operations in sequence
const setupCommands = [
  new CreateSceneCommand(),
  new AddCameraCommand(),
  new AddLightCommand(),
];
await historyManager.executeBatch(setupCommands, 'sequential');
```

### 3. Error Handling

- Use transactions for operations that must succeed together
- Implement proper error recovery
- Log errors for debugging

```typescript
try {
  await historyManager.executeTransaction(commands, {
    rollbackOnFailure: true,
    stopOnFirstError: true,
  });
} catch (error) {
  console.error('Transaction failed, rolled back:', error);
  // Show error to user
}
```

### 4. Performance

- Enable auto-save with appropriate intervals
- Regularly optimize history
- Set reasonable history limits

```typescript
// Configure with performance in mind
const historyManager = new HistoryManager(500, persistence);

// Periodic optimization (e.g., every 10 minutes)
setInterval(async () => {
  await historyManager.optimize({
    mergeConsecutiveCommands: true,
    removeDuplicateCommands: true,
    keepBookmarks: true,
  });
}, 10 * 60 * 1000);
```

### 5. User Experience

- Create bookmarks before major changes
- Use descriptive names
- Provide visual feedback

```typescript
// Before major operation
const beforeBookmark = historyManager.createBookmark(
  'Before refactoring',
  'State before major scene refactoring'
);

// Perform operation
await performMajorRefactoring();

// After operation
const afterBookmark = historyManager.createBookmark(
  'After refactoring',
  'State after major scene refactoring'
);
```

## Examples

### Example 1: Scene Editor with Undo/Redo

```typescript
class SceneEditor {
  private history: HistoryManager;

  constructor() {
    this.history = new HistoryManager(100);
  }

  async addEntity(entity: Entity) {
    const command = new CreateEntityCommand(
      entity.id,
      entity,
      (e) => this.scene.add(e),
      (id) => this.scene.remove(id)
    );
    await this.history.executeCommand(command);
  }

  async moveEntity(entityId: string, newPosition: Vector3) {
    const entity = this.scene.get(entityId);
    const command = new TransformEntityCommand(
      entityId,
      entity.transform,
      { ...entity.transform, position: newPosition },
      (id, transform) => this.scene.updateTransform(id, transform)
    );
    await this.history.executeCommand(command);
  }

  undo() {
    return this.history.undo();
  }

  redo() {
    return this.history.redo();
  }
}
```

### Example 2: Batch Entity Operations

```typescript
async function duplicateSelectedEntities(selectedIds: string[]) {
  const commands: Command[] = [];

  for (const id of selectedIds) {
    const entity = scene.get(id);
    const duplicate = cloneEntity(entity);
    duplicate.id = generateUniqueId();

    commands.push(
      new CreateEntityCommand(
        duplicate.id,
        duplicate,
        (e) => scene.add(e),
        (id) => scene.remove(id)
      )
    );
  }

  // Execute all duplications in parallel
  await historyManager.executeBatch(commands, 'parallel');
}
```

### Example 3: Macro for Common Operations

```typescript
// Define common scene setup as a macro
const createRoomMacro = new MacroCommand(
  'Create Room',
  [
    new CreateEntityCommand('floor', floorEntity, add, remove),
    new CreateEntityCommand('walls', wallsEntity, add, remove),
    new CreateEntityCommand('ceiling', ceilingEntity, add, remove),
    new CreateEntityCommand('light', lightEntity, add, remove),
  ],
  {
    roomSize: { width: 10, height: 3, depth: 10 },
    lightIntensity: 1.0,
  }
);

// Use the macro
await historyManager.executeCommand(createRoomMacro);

// Create with custom parameters
const customRoom = createRoomMacro.withParameters({
  roomSize: { width: 15, height: 4, depth: 15 },
  lightIntensity: 1.5,
});
await historyManager.executeCommand(customRoom);
```

### Example 4: Saving Snapshots

```typescript
// Create snapshot before destructive operations
async function performDestructiveOperation() {
  // Create bookmark
  const snapshot = historyManager.createBookmark(
    `Pre-operation snapshot`,
    `Snapshot before ${operationName}`,
    ['snapshot', 'backup']
  );

  try {
    // Perform operation
    await operation();

    // Create success bookmark
    historyManager.createBookmark(
      `Post-operation success`,
      `Operation completed successfully`
    );
  } catch (error) {
    // Rollback to snapshot
    await historyManager.jumpToBookmark(snapshot.id);
    throw error;
  }
}
```

## Command Registry

### Registering Command Types

```typescript
import { getGlobalRegistry } from './utils/CommandRegistry';

const registry = getGlobalRegistry();

registry.register(
  'CreateEntity',
  (id, entity, add, remove) => new CreateEntityCommand(id, entity, add, remove),
  {
    type: 'CreateEntity',
    category: 'entity',
    description: 'Create a new entity',
    version: '1.0.0',
    tags: ['entity', 'create'],
  }
);

// Create command from registry
const command = registry.create('CreateEntity', id, entity, add, remove);
await historyManager.executeCommand(command);
```

### Tracking Statistics

```typescript
// Get statistics for a specific command type
const stats = registry.getStatistics('CreateEntity');
console.log('Total executions:', stats.totalExecutions);
console.log('Success rate:', stats.successfulExecutions / stats.totalExecutions);
console.log('Average time:', stats.averageExecutionTime);

// Get overall statistics
const allStats = registry.getAllStatistics();
console.log('Most used commands:', allStats);
```

## API Reference

### HistoryManager

#### Methods

- `executeCommand(command: Command): Promise<void>` - Execute a command
- `undo(): Promise<void>` - Undo last command
- `redo(): Promise<void>` - Redo last undone command
- `executeBatch(commands: Command[], mode): Promise<void>` - Execute batch
- `executeTransaction(commands: Command[], options?): Promise<void>` - Execute transaction
- `executeMacro(name: string, commands: Command[], params?): Promise<void>` - Execute macro
- `createBookmark(name, description?, tags?): HistoryBookmark` - Create bookmark
- `jumpToBookmark(id: string): Promise<void>` - Jump to bookmark
- `createBranch(name, parentId?): HistoryBranch` - Create branch
- `switchBranch(id: string): Promise<void>` - Switch branch
- `search(query: string): HistorySearchResult` - Search history
- `filter(options: HistoryFilterOptions): Command[]` - Filter commands
- `compare(stateId1, stateId2): HistoryDiff` - Compare states
- `getStatistics(): HistoryStatistics` - Get statistics
- `optimize(options?): Promise<void>` - Optimize history
- `save(): Promise<void>` - Save history
- `load(): Promise<void>` - Load history
- `export(format): Promise<Blob>` - Export history
- `import(blob): Promise<void>` - Import history

### Persistence Options

```typescript
interface HistoryPersistenceConfig {
  enabled: boolean;
  storageType: 'localStorage' | 'indexedDB';
  autoSave: boolean;
  autoSaveInterval: number; // milliseconds
  maxStorageSize: number; // bytes
  compress: boolean;
  encrypt: boolean;
}
```

## Troubleshooting

### Common Issues

**Issue**: History not persisting
```typescript
// Solution: Check persistence is enabled
const persistence = new HistoryPersistence({
  enabled: true, // Must be true
  autoSave: true,
});
```

**Issue**: Memory usage too high
```typescript
// Solution: Reduce history size and optimize
const history = new HistoryManager(50); // Reduce limit
await history.optimize({
  mergeConsecutiveCommands: true,
  removeDuplicateCommands: true,
});
```

**Issue**: Undo/redo not working
```typescript
// Solution: Ensure commands properly implement undo/redo
class MyCommand implements Command {
  async execute() { /* ... */ }
  async undo() { /* Must be implemented */ }
  async redo() { /* Must be implemented */ }
}
```

## Conclusion

The enhanced undo/redo system provides powerful tools for managing application state history. By following best practices and leveraging the advanced features, you can create robust applications with excellent user experience.

For more information, see the individual module documentation and type definitions.
