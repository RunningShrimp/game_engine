# Behavior Tree Editor

## Overview

The Behavior Tree Editor is a complete visual editor for creating and managing AI behavior trees in your game. It provides an intuitive node-based interface for designing complex AI logic.

## Features

### Core Functionality

- **Visual Node Editor**: Drag-and-drop interface for creating behavior trees
- **Node Types**:
  - **Composite Nodes**: Sequence, Selector, Parallel
  - **Decorator Nodes**: Inverter, Repeater, Cooldown, Always Succeed, Always Fail
  - **Condition Nodes**: Condition, Check
  - **Action Nodes**: Action, Wait, Log

- **Blackboard System**: Key-value storage for sharing data between nodes
- **Real-time Debugging**: Step-by-step execution with breakpoints
- **Tree Management**: Save, load, and validate behavior trees

### Components

```
src/components/BehaviorEditor/
├── BehaviorEditor.tsx         # Main editor component
├── NodeCanvas.tsx             # Infinite canvas with pan/zoom
├── BehaviorNode.tsx           # Individual node rendering
├── ConnectionLine.tsx         # Bezier curve connections
├── NodePalette.tsx            # Node creation panel
├── PropertiesPanel.tsx        # Node property editor
├── BlackboardEditor.tsx       # Blackboard variable manager
├── DebugPanel.tsx             # Runtime debugging tools
├── BehaviorTreeManager.tsx    # Save/load interface
└── Toolbar.tsx                # Top toolbar
```

## Usage

### Basic Workflow

1. **Create a Tree**:
   - Click "New" in the toolbar or press `Ctrl+N`
   - A default root node (Sequence) is created

2. **Add Nodes**:
   - Open the Node Palette (left panel)
   - Drag nodes from the palette onto the canvas
   - Or select a composite node and click a node type to add as child

3. **Configure Nodes**:
   - Select a node to view its properties
   - Edit node name, description, and parameters in the Properties Panel (right side)

4. **Setup Blackboard**:
   - Open Blackboard Editor (bottom-right button)
   - Add variables to store data accessible by all nodes
   - Supported types: boolean, number, string, object, vector3

5. **Debug**:
   - Click the "Edit/Debug" toggle to enter debug mode
   - Use Run, Pause, Step buttons to control execution
   - Set breakpoints by clicking on nodes
   - Monitor blackboard variables in real-time

### Keyboard Shortcuts

- `Ctrl+N`: New behavior tree
- `Ctrl+S`: Save tree
- `Ctrl+O`: Open tree manager
- `Delete/Backspace`: Remove selected nodes
- `Ctrl+C`: Copy selected nodes
- `Ctrl+V`: Paste nodes
- `Escape`: Clear selection
- `Ctrl+Scroll`: Zoom canvas
- `Space + Drag`: Pan canvas

### Node Types Reference

#### Composite Nodes

**Sequence (➡️)**
- Executes children in order
- Succeeds if all children succeed
- Fails if any child fails

**Selector (⭕)**
- Tries children in order
- Succeeds if any child succeeds
- Fails if all children fail

**Parallel (⫴)**
- Executes all children simultaneously
- Success mode: "all" or "any"

#### Decorator Nodes

**Inverter (🔄)**
- Inverts child's result (success → failure, failure → success)
- Must have exactly 1 child

**Repeater (🔁)**
- Repeats child N times
- Parameter: repeat count (1-100)
- Must have exactly 1 child

**Cooldown (⏱️)**
- Adds cooldown between executions
- Parameter: duration in seconds (0.1-60)
- Must have exactly 1 child

**Always Succeed (✅)**
- Always returns success regardless of child result
- Must have exactly 1 child

**Always Fail (❌)**
- Always returns failure regardless of child result
- Must have exactly 1 child

#### Condition Nodes

**Condition (❓)**
- Checks blackboard variable
- Parameters:
  - Variable: variable name
  - Operator: equals, not_equals, greater, less, contains
  - Value: comparison value

**Check (✓)**
- Evaluates custom expression
- Parameter: expression string

#### Action Nodes

**Action (⚡)**
- Executes custom game logic
- Parameters:
  - Action Name: identifier
  - Parameters: JSON object

**Wait (⏸️)**
- Pauses execution for specified time
- Parameter: duration in seconds (0.1-60)

**Log (📝)**
- Outputs message to console
- Parameters:
  - Message: text to log
  - Level: debug, info, warning, error

## Examples

### Example 1: Simple Patrol AI

```
[Root: Sequence]
  └─[Sequence: Patrol Loop]
     ├─[Condition: Has Target?]
     │  └─[Selector: Approach or Ignore]
     │     ├─[Action: Move to Target]
     │     └─[Action: Random Wander]
     └─[Action: Look Around]
        └─[Wait: 2s]
```

### Example 2: Combat AI

```
[Root: Selector]
  └─[Selector: Attack or Flee]
     ├─[Sequence: Attack]
     │  ├─[Condition: Can Attack?]
     │  ├─[Action: Attack]
     │  └─[Cooldown: 1s]
     └─[Sequence: Flee]
        ├─[Condition: Health Low?]
        └─[Action: Run Away]
```

## Tauri Backend Commands

The behavior tree editor integrates with Tauri backend commands:

```rust
// Tree management
create_behavior_tree(name: String, description: Option<String>) -> BehaviorTree
save_behavior_tree(tree: BehaviorTree)
load_behavior_tree(id: String) -> BehaviorTree
list_behavior_trees() -> Vec<BehaviorTree>
delete_behavior_tree(id: String)

// Validation
validate_behavior_tree(tree: BehaviorTree) -> ValidationResult

// Execution
execute_behavior_tree(tree_id: String) -> ExecutionResult
debug_behavior_step(tree_id: String) -> ExecutionState

// Debugging
set_breakpoint(tree_id: String, node_id: String)
clear_breakpoint(tree_id: String, node_id: String)
```

## Data Structures

### BehaviorTree

```typescript
interface BehaviorTree {
  id: string;
  name: string;
  description?: string;
  root: BehaviorNode | null;
  blackboard: Blackboard;
  createdAt: number;
  updatedAt: number;
}
```

### BehaviorNode

```typescript
interface BehaviorNode {
  id: string;
  type: NodeType;
  name: string;
  description?: string;
  position: { x: number; y: number };
  children: BehaviorNode[];
  parameters: NodeParameter[];
  status?: NodeStatus;
  executionCount?: number;
  lastExecutionTime?: number;
}
```

### Blackboard

```typescript
interface Blackboard {
  variables: Record<string, BlackboardVariable>;
}

interface BlackboardVariable {
  name: string;
  type: 'boolean' | 'number' | 'string' | 'object' | 'vector3';
  value: any;
  description?: string;
}
```

## File Locations

- **TypeScript Types**: `src/types/behavior.ts`
- **Components**: `src/components/BehaviorEditor/`
- **Tauri Backend**: `src-tauri/src/behavior_tree.rs`

## Integration

To integrate the behavior tree editor into your application:

```tsx
import { BehaviorEditor } from './components/BehaviorEditor';

function App() {
  return (
    <div className="app">
      <BehaviorEditor
        onTreeChange={(tree) => {
          console.log('Tree updated:', tree);
        }}
      />
    </div>
  );
}
```

## Performance Tips

1. **Limit Tree Depth**: Keep behavior trees shallow (< 10 levels) for optimal performance
2. **Cache Blackboard Lookups**: Store frequently accessed variables locally
3. **Use Cooldowns**: Prevent rapid re-execution of expensive actions
4. **Optimize Conditions**: Place cheap conditions before expensive ones

## Troubleshooting

### Nodes not connecting
- Ensure parent node is a composite or decorator type
- Check that parent has capacity for more children

### Debug mode not working
- Verify tree has been validated
- Check console for errors in node execution

### Blackboard variables not updating
- Ensure variable names match exactly (case-sensitive)
- Check variable types are compatible

## Future Enhancements

- [ ] Visual scripting for action nodes
- [ ] Behavior tree templates library
- [ ] Hot-reload during gameplay
- [ ] Performance profiling
- [ ] Node search and filtering
- [ ] Undo/redo functionality
- [ ] Multi-tree editing
- [ ] Export/import JSON
- [ ] Node grouping/comments
- [ ] Behavior tree visualization (execution graph)

## License

MIT License - See LICENSE file for details
