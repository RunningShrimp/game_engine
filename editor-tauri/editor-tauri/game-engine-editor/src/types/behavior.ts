/**
 * Behavior Tree Type Definitions
 * Defines the structure of behavior trees for AI and game logic
 */

/**
 * Node Types in the Behavior Tree
 */
export enum NodeType {
  // Composite Nodes
  Sequence = 'sequence',           // Execute children in sequence
  Selector = 'selector',           // Execute children until one succeeds
  Parallel = 'parallel',           // Execute children in parallel

  // Decorator Nodes
  Inverter = 'inverter',           // Invert child's result
  Repeater = 'repeater',           // Repeat child N times
  Cooldown = 'cooldown',           // Add cooldown between executions
  AlwaysSucceed = 'always_succeed', // Always return success
  AlwaysFail = 'always_fail',      // Always return failure

  // Condition Nodes
  Condition = 'condition',         // Check condition
  Check = 'check',                 // Check state/value

  // Action Nodes
  Action = 'action',               // Execute action
  Wait = 'wait',                   // Wait for specified time
  Log = 'log',                     // Log message
}

/**
 * Node Execution Status
 */
export enum NodeStatus {
  Idle = 'idle',           // Not yet executed
  Running = 'running',     // Currently executing
  Success = 'success',     // Executed successfully
  Failure = 'failure',     // Execution failed
}

/**
 * Node Parameter Definition
 */
export interface NodeParameter {
  id: string;
  name: string;
  dataType: 'boolean' | 'number' | 'string' | 'object' | 'enum';
  value: any;
  enumValues?: string[];  // For enum type
  min?: number;           // For number type
  max?: number;           // For number type
  step?: number;          // For number type
}

/**
 * Behavior Node
 */
export interface BehaviorNode {
  id: string;
  type: NodeType;
  name: string;
  description?: string;
  position: { x: number; y: number };
  children: BehaviorNode[];
  parameters: NodeParameter[];
  // Debug information
  status?: NodeStatus;
  executionCount?: number;
  lastExecutionTime?: number;
}

/**
 * Blackboard Variable
 */
export interface BlackboardVariable {
  name: string;
  type: 'boolean' | 'number' | 'string' | 'object' | 'vector3';
  value: any;
  description?: string;
}

/**
 * Blackboard (Key-Value Storage)
 */
export interface Blackboard {
  variables: Record<string, BlackboardVariable>;
}

/**
 * Behavior Tree
 */
export interface BehaviorTree {
  id: string;
  name: string;
  description?: string;
  root: BehaviorNode | null;
  blackboard: Blackboard;
  createdAt: number;
  updatedAt: number;
}

/**
 * Execution Result
 */
export interface ExecutionResult {
  success: boolean;
  status: NodeStatus;
  executedNodes: string[];
  executionTime: number;
  blackboardUpdates: Record<string, any>;
}

/**
 * Execution State (for debugging)
 */
export interface ExecutionState {
  currentNodeId: string | null;
  nodeStates: Record<string, NodeStatus>;
  blackboard: Blackboard;
  breakpoints: Set<string>;
  isPaused: boolean;
}

/**
 * Node Category for palette organization
 */
export enum NodeCategory {
  Composite = 'composite',
  Decorator = 'decorator',
  Condition = 'condition',
  Action = 'action',
}

/**
 * Node Template (for creating new nodes)
 */
export interface NodeTemplate {
  type: NodeType;
  category: NodeCategory;
  name: string;
  description: string;
  icon: string;
  defaultParameters: NodeParameter[];
  maxChildren?: number;  // undefined = unlimited
  allowChildren: boolean;
}

/**
 * Tree Validation Result
 */
export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Tree Template
 */
export interface TreeTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  thumbnail?: string;
  tree: BehaviorTree;
}
