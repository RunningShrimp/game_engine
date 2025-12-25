//! 决策树编辑器
//!
//! 提供决策树的可视化编辑和管理功能：
//! - 节点创建和编辑
//! - 树结构可视化
//! - 节点连接管理
//! - 决策树序列化/反序列化
//! - 决策树验证

use crate::ai::behavior_tree::{Node, Status};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 决策树节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionNodeType {
    /// 条件节点（叶子节点）
    Condition,
    /// 动作节点（叶子节点）
    Action,
    /// 选择器节点（内部节点）
    Selector,
    /// 序列节点（内部节点）
    Sequence,
    /// 装饰器节点
    Decorator,
}

/// 决策树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTreeNode {
    /// 节点ID
    pub id: u64,
    /// 节点类型
    pub node_type: DecisionNodeType,
    /// 节点名称
    pub name: String,
    /// 节点描述
    pub description: String,
    /// 节点位置（用于可视化）
    pub position: (f32, f32),
    /// 子节点ID列表
    pub children: Vec<u64>,
    /// 节点数据（类型特定的数据）
    pub data: DecisionNodeData,
}

/// 决策节点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionNodeData {
    /// 条件节点数据
    Condition {
        /// 条件表达式
        expression: String,
        /// 条件参数
        parameters: HashMap<String, String>,
    },
    /// 动作节点数据
    Action {
        /// 动作名称
        action_name: String,
        /// 动作参数
        parameters: HashMap<String, String>,
    },
    /// 选择器/序列节点数据
    Composite {
        /// 执行策略
        strategy: String,
    },
    /// 装饰器节点数据
    Decorator {
        /// 装饰器类型
        decorator_type: String,
        /// 装饰器参数
        parameters: HashMap<String, String>,
    },
}

/// 决策树
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    /// 树名称
    pub name: String,
    /// 根节点ID
    pub root_id: Option<u64>,
    /// 节点映射
    pub nodes: HashMap<u64, DecisionTreeNode>,
    /// 下一个节点ID
    pub next_node_id: u64,
}

impl DecisionTree {
    /// 创建新的决策树
    pub fn new(name: String) -> Self {
        Self {
            name,
            root_id: None,
            nodes: HashMap::new(),
            next_node_id: 1,
        }
    }

    /// 添加节点
    pub fn add_node(
        &mut self,
        node_type: DecisionNodeType,
        name: String,
        position: (f32, f32),
    ) -> u64 {
        let id = self.next_node_id;
        self.next_node_id += 1;

        let data = match node_type {
            DecisionNodeType::Condition => DecisionNodeData::Condition {
                expression: String::new(),
                parameters: HashMap::new(),
            },
            DecisionNodeType::Action => DecisionNodeData::Action {
                action_name: String::new(),
                parameters: HashMap::new(),
            },
            DecisionNodeType::Selector | DecisionNodeType::Sequence => {
                DecisionNodeData::Composite {
                    strategy: "default".to_string(),
                }
            }
            DecisionNodeType::Decorator => DecisionNodeData::Decorator {
                decorator_type: "inverter".to_string(),
                parameters: HashMap::new(),
            },
        };

        let node = DecisionTreeNode {
            id,
            node_type,
            name,
            description: String::new(),
            position,
            children: Vec::new(),
            data,
        };

        self.nodes.insert(id, node);

        // 如果没有根节点，设为根节点
        if self.root_id.is_none() {
            self.root_id = Some(id);
        }

        id
    }

    /// 移除节点
    pub fn remove_node(&mut self, node_id: u64) -> Result<(), DecisionTreeError> {
        if !self.nodes.contains_key(&node_id) {
            return Err(DecisionTreeError::NodeNotFound(node_id));
        }

        // 从父节点移除引用
        for node in self.nodes.values_mut() {
            node.children.retain(|&id| id != node_id);
        }

        // 如果是根节点，清除根节点
        if self.root_id == Some(node_id) {
            self.root_id = None;
        }

        self.nodes.remove(&node_id);
        Ok(())
    }

    /// 添加子节点
    pub fn add_child(&mut self, parent_id: u64, child_id: u64) -> Result<(), DecisionTreeError> {
        let parent = self.nodes.get_mut(&parent_id)
            .ok_or(DecisionTreeError::NodeNotFound(parent_id))?;

        // 检查节点类型是否支持子节点
        match parent.node_type {
            DecisionNodeType::Condition | DecisionNodeType::Action => {
                return Err(DecisionTreeError::InvalidOperation(
                    "Leaf nodes cannot have children".to_string(),
                ));
            }
            _ => {}
        }

        if !parent.children.contains(&child_id) {
            parent.children.push(child_id);
        }

        Ok(())
    }

    /// 移除子节点
    pub fn remove_child(&mut self, parent_id: u64, child_id: u64) -> Result<(), DecisionTreeError> {
        let parent = self.nodes.get_mut(&parent_id)
            .ok_or(DecisionTreeError::NodeNotFound(parent_id))?;

        parent.children.retain(|&id| id != child_id);
        Ok(())
    }

    /// 更新节点
    pub fn update_node(&mut self, node_id: u64, updates: NodeUpdates) -> Result<(), DecisionTreeError> {
        let node = self.nodes.get_mut(&node_id)
            .ok_or(DecisionTreeError::NodeNotFound(node_id))?;

        if let Some(name) = updates.name {
            node.name = name;
        }

        if let Some(description) = updates.description {
            node.description = description;
        }

        if let Some(position) = updates.position {
            node.position = position;
        }

        if let Some(data) = updates.data {
            node.data = data;
        }

        Ok(())
    }

    /// 验证决策树
    pub fn validate(&self) -> Result<(), DecisionTreeError> {
        // 检查根节点
        if self.root_id.is_none() {
            return Err(DecisionTreeError::InvalidTree("No root node".to_string()));
        }

        let root_id = self.root_id.unwrap();
        if !self.nodes.contains_key(&root_id) {
            return Err(DecisionTreeError::InvalidTree("Root node not found".to_string()));
        }

        // 检查所有节点是否可达
        let mut visited = HashSet::new();
        self.visit_node(root_id, &mut visited)?;

        // 检查是否有孤立节点
        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                return Err(DecisionTreeError::InvalidTree(
                    format!("Isolated node: {}", node_id),
                ));
            }
        }

        Ok(())
    }

    /// 访问节点（递归）
    fn visit_node(&self, node_id: u64, visited: &mut HashSet<u64>) -> Result<(), DecisionTreeError> {
        if visited.contains(&node_id) {
            return Ok(()); // 已访问，可能是循环引用（需要更严格的检查）
        }

        visited.insert(node_id);

        let node = self.nodes.get(&node_id)
            .ok_or(DecisionTreeError::NodeNotFound(node_id))?;

        for &child_id in &node.children {
            self.visit_node(child_id, visited)?;
        }

        Ok(())
    }

    /// 获取节点
    pub fn get_node(&self, node_id: u64) -> Option<&DecisionTreeNode> {
        self.nodes.get(&node_id)
    }

    /// 获取所有节点
    pub fn get_all_nodes(&self) -> &HashMap<u64, DecisionTreeNode> {
        &self.nodes
    }
}

use std::collections::HashSet;

/// 节点更新
#[derive(Debug, Clone)]
pub struct NodeUpdates {
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: Option<(f32, f32)>,
    pub data: Option<DecisionNodeData>,
}

/// 决策树错误
#[derive(Debug, Clone)]
pub enum DecisionTreeError {
    NodeNotFound(u64),
    InvalidOperation(String),
    InvalidTree(String),
}

impl std::fmt::Display for DecisionTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionTreeError::NodeNotFound(id) => {
                write!(f, "Node not found: {}", id)
            }
            DecisionTreeError::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {}", msg)
            }
            DecisionTreeError::InvalidTree(msg) => {
                write!(f, "Invalid tree: {}", msg)
            }
        }
    }
}

impl std::error::Error for DecisionTreeError {}

/// 决策树编辑器
pub struct DecisionTreeEditor {
    /// 当前决策树
    current_tree: Option<DecisionTree>,
    /// 决策树库
    tree_library: HashMap<String, DecisionTree>,
}

impl DecisionTreeEditor {
    /// 创建新的决策树编辑器
    pub fn new() -> Self {
        Self {
            current_tree: None,
            tree_library: HashMap::new(),
        }
    }

    /// 创建新决策树
    pub fn create_tree(&mut self, name: String) -> &mut DecisionTree {
        let tree = DecisionTree::new(name.clone());
        self.tree_library.insert(name.clone(), tree);
        self.current_tree = Some(DecisionTree::new(name));
        self.current_tree.as_mut().unwrap()
    }

    /// 加载决策树
    pub fn load_tree(&mut self, name: &str) -> Result<&mut DecisionTree, DecisionTreeError> {
        let tree = self.tree_library.get(name)
            .ok_or(DecisionTreeError::InvalidTree(format!("Tree not found: {}", name)))?;
        self.current_tree = Some(tree.clone());
        Ok(self.current_tree.as_mut().unwrap())
    }

    /// 保存当前决策树
    pub fn save_current_tree(&mut self) -> Result<(), DecisionTreeError> {
        let tree = self.current_tree.take()
            .ok_or(DecisionTreeError::InvalidOperation("No current tree".to_string()))?;
        
        tree.validate()?;
        self.tree_library.insert(tree.name.clone(), tree);
        Ok(())
    }

    /// 获取当前决策树
    pub fn get_current_tree(&self) -> Option<&DecisionTree> {
        self.current_tree.as_ref()
    }

    /// 获取当前决策树（可变）
    pub fn get_current_tree_mut(&mut self) -> Option<&mut DecisionTree> {
        self.current_tree.as_mut()
    }

    /// 列出所有决策树
    pub fn list_trees(&self) -> Vec<&String> {
        self.tree_library.keys().collect()
    }
}

impl Default for DecisionTreeEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_tree_creation() {
        let mut tree = DecisionTree::new("Test Tree".to_string());
        let root_id = tree.add_node(
            DecisionNodeType::Selector,
            "Root".to_string(),
            (0.0, 0.0),
        );
        assert_eq!(tree.root_id, Some(root_id));
    }

    #[test]
    fn test_decision_tree_validation() {
        let mut tree = DecisionTree::new("Test Tree".to_string());
        let root_id = tree.add_node(
            DecisionNodeType::Selector,
            "Root".to_string(),
            (0.0, 0.0),
        );
        let child_id = tree.add_node(
            DecisionNodeType::Action,
            "Action".to_string(),
            (0.0, 100.0),
        );
        tree.add_child(root_id, child_id).unwrap();
        assert!(tree.validate().is_ok());
    }
}

