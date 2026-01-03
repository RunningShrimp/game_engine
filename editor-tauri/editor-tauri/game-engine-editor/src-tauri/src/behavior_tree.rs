/**
 * Behavior Tree Backend Module
 * Provides Tauri commands for behavior tree management and execution
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Behavior tree node types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Sequence,
    Selector,
    Parallel,
    Inverter,
    Repeater,
    Cooldown,
    AlwaysSucceed,
    AlwaysFail,
    Condition,
    Check,
    Action,
    Wait,
    Log,
}

/// Node execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Idle,
    Running,
    Success,
    Failure,
}

/// Node parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeParameter {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub value: serde_json::Value,
}

/// Behavior tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub description: Option<String>,
    pub position: Position,
    pub children: Vec<BehaviorNode>,
    pub parameters: Vec<NodeParameter>,
    pub status: Option<NodeStatus>,
    pub execution_count: Option<u32>,
    pub last_execution_time: Option<f64>,
}

/// Position in 2D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Blackboard variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardVariable {
    pub name: String,
    pub variable_type: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
}

/// Blackboard (key-value storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blackboard {
    pub variables: HashMap<String, BlackboardVariable>,
}

/// Behavior tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorTree {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub root: Option<BehaviorNode>,
    pub blackboard: Blackboard,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub status: NodeStatus,
    pub executed_nodes: Vec<String>,
    pub execution_time: f64,
    pub blackboard_updates: HashMap<String, serde_json::Value>,
}

/// Execution state (for debugging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub current_node_id: Option<String>,
    pub node_states: HashMap<String, NodeStatus>,
    pub blackboard: Blackboard,
    pub breakpoints: Vec<String>,
    pub is_paused: bool,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Behavior tree manager state
pub struct BehaviorTreeManager {
    trees: HashMap<String, BehaviorTree>,
    execution_states: HashMap<String, ExecutionState>,
}

impl BehaviorTreeManager {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            execution_states: HashMap::new(),
        }
    }

    pub fn add_tree(&mut self, tree: BehaviorTree) {
        self.trees.insert(tree.id.clone(), tree);
    }

    pub fn get_tree(&self, id: &str) -> Option<&BehaviorTree> {
        self.trees.get(id)
    }

    pub fn list_trees(&self) -> Vec<BehaviorTree> {
        self.trees.values().cloned().collect()
    }

    pub fn delete_tree(&mut self, id: &str) -> Option<BehaviorTree> {
        self.trees.remove(id)
    }
}

/// Tauri commands for behavior tree management

/// Create a new behavior tree
#[tauri::command]
pub fn create_behavior_tree(
    name: String,
    description: Option<String>,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<BehaviorTree, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tree = BehaviorTree {
        id: format!("tree_{}", now),
        name,
        description,
        root: None,
        blackboard: Blackboard {
            variables: HashMap::new(),
        },
        created_at: now,
        updated_at: now,
    };

    let mut manager = state.lock().unwrap();
    manager.add_tree(tree.clone());

    Ok(tree)
}

/// Save behavior tree
#[tauri::command]
pub fn save_behavior_tree(
    tree: BehaviorTree,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();
    manager.add_tree(tree);
    Ok(())
}

/// Load behavior tree
#[tauri::command]
pub fn load_behavior_tree(
    id: String,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<BehaviorTree, String> {
    let manager = state.lock().unwrap();
    manager
        .get_tree(&id)
        .cloned()
        .ok_or_else(|| "Tree not found".to_string())
}

/// List all behavior trees
#[tauri::command]
pub fn list_behavior_trees(
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<Vec<BehaviorTree>, String> {
    let manager = state.lock().unwrap();
    Ok(manager.list_trees())
}

/// Delete behavior tree
#[tauri::command]
pub fn delete_behavior_tree(
    id: String,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();
    manager
        .delete_tree(&id)
        .ok_or_else(|| "Tree not found".to_string())?;
    Ok(())
}

/// Validate behavior tree
#[tauri::command]
pub fn validate_behavior_tree(tree: BehaviorTree) -> Result<ValidationResult, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check if tree has a root node
    if tree.root.is_none() {
        errors.push("Behavior tree has no root node".to_string());
    }

    // Validate nodes recursively
    if let Some(root) = &tree.root {
        validate_node(root, &mut errors, &mut warnings, vec!["Root".to_string()]);
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

/// Helper function to validate nodes recursively
fn validate_node(
    node: &BehaviorNode,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    path: Vec<String>,
) {
    let current_path = path.join(" > ");

    // Check composite nodes have children
    match node.node_type {
        NodeType::Sequence | NodeType::Selector | NodeType::Parallel => {
            if node.children.is_empty() {
                warnings.push(format!(
                    "Composite node '{}' at {} has no children",
                    node.name, current_path
                ));
            }
        }
        NodeType::Inverter | NodeType::Repeater | NodeType::Cooldown => {
            if node.children.len() != 1 {
                errors.push(format!(
                    "Decorator node '{}' at {} must have exactly 1 child, found {}",
                    node.name,
                    current_path,
                    node.children.len()
                ));
            }
        }
        _ => {
            if !node.children.is_empty() {
                warnings.push(format!(
                    "Leaf node '{}' at {} has children, which will be ignored",
                    node.name, current_path
                ));
            }
        }
    }

    // Recursively validate children
    for (i, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(format!("{}[{}]", node.name, i));
        validate_node(child, errors, warnings, child_path);
    }
}

/// Execute behavior tree (simulation)
#[tauri::command]
pub fn execute_behavior_tree(
    tree_id: String,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<ExecutionResult, String> {
    let manager = state.lock().unwrap();
    let tree = manager
        .get_tree(&tree_id)
        .ok_or_else(|| "Tree not found".to_string())?;

    // Simulate execution
    let start = std::time::Instant::now();

    let executed_nodes = if let Some(root) = &tree.root {
        collect_node_ids(root)
    } else {
        Vec::new()
    };

    let execution_time = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ExecutionResult {
        success: true,
        status: NodeStatus::Success,
        executed_nodes,
        execution_time,
        blackboard_updates: HashMap::new(),
    })
}

/// Debug behavior tree step by step
#[tauri::command]
pub fn debug_behavior_step(
    tree_id: String,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<ExecutionState, String> {
    let manager = state.lock().unwrap();
    let tree = manager
        .get_tree(&tree_id)
        .ok_or_else(|| "Tree not found".to_string())?;

    // Get or create execution state
    let exec_state = manager
        .execution_states
        .get(&tree_id)
        .cloned()
        .unwrap_or_else(|| ExecutionState {
            current_node_id: tree.root.as_ref().map(|n| n.id.clone()),
            node_states: HashMap::new(),
            blackboard: tree.blackboard.clone(),
            breakpoints: Vec::new(),
            is_paused: false,
        });

    Ok(exec_state)
}

/// Set breakpoint
#[tauri::command]
pub fn set_breakpoint(
    tree_id: String,
    node_id: String,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();

    let exec_state = manager
        .execution_states
        .entry(tree_id.clone())
        .or_insert_with(|| ExecutionState {
            current_node_id: None,
            node_states: HashMap::new(),
            blackboard: Blackboard {
                variables: HashMap::new(),
            },
            breakpoints: Vec::new(),
            is_paused: false,
        });

    if !exec_state.breakpoints.contains(&node_id) {
        exec_state.breakpoints.push(node_id);
    }

    Ok(())
}

/// Clear breakpoint
#[tauri::command]
pub fn clear_breakpoint(
    tree_id: String,
    node_id: String,
    state: tauri::State<Mutex<BehaviorTreeManager>>,
) -> Result<(), String> {
    let mut manager = state.lock().unwrap();

    if let Some(exec_state) = manager.execution_states.get_mut(&tree_id) {
        exec_state.breakpoints.retain(|id| id != &node_id);
    }

    Ok(())
}

/// Helper function to collect all node IDs recursively
fn collect_node_ids(node: &BehaviorNode) -> Vec<String> {
    let mut ids = vec![node.id.clone()];
    for child in &node.children {
        ids.extend(collect_node_ids(child));
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_behavior_tree() {
        let tree = BehaviorTree {
            id: "test_tree".to_string(),
            name: "Test Tree".to_string(),
            description: Some("A test tree".to_string()),
            root: None,
            blackboard: Blackboard {
                variables: HashMap::new(),
            },
            created_at: 0,
            updated_at: 0,
        };

        assert_eq!(tree.id, "test_tree");
        assert_eq!(tree.name, "Test Tree");
    }

    #[test]
    fn test_validate_empty_tree() {
        let tree = BehaviorTree {
            id: "test_tree".to_string(),
            name: "Test Tree".to_string(),
            description: None,
            root: None,
            blackboard: Blackboard {
                variables: HashMap::new(),
            },
            created_at: 0,
            updated_at: 0,
        };

        let result = validate_behavior_tree(tree).unwrap();
        assert!(!result.valid);
        assert!(result.errors.len() > 0);
    }
}
