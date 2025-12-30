// 行为树JSON序列化/反序列化
//
// 支持从JSON文件加载和保存行为树定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::behavior_tree::{Node, Sequence, Selector, Status, Action};

/// 行为树JSON表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorTreeJson {
    pub version: String,
    pub tree: String,
    #[serde(rename = "nodes")]
    pub node_definitions: Vec<NodeDefinition>,
}

/// 节点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub name: String,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Sequence,
    Selector,
    Parallel,
    /// 反转装饰器
    Inverter,
    /// 重复装饰器
    Repeat,
    /// 条件节点
    Condition,
    /// 行为节点
    Action,
}

/// 行为树序列化错误
#[derive(Debug)]
pub enum SerializationError {
    ParseError(String),
    InvalidNode(String),
    MissingChild(String),
    InvalidType(String),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SerializationError::InvalidNode(msg) => write!(f, "Invalid node: {}", msg),
            SerializationError::MissingChild(msg) => write!(f, "Missing child: {}", msg),
            SerializationError::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
        }
    }
}

impl std::error::Error for SerializationError {}

/// 行为树反序列化器
pub struct BehaviorTreeDeserializer;

impl BehaviorTreeDeserializer {
    /// 从JSON字符串反序列化行为树
    pub fn from_json(json: &str) -> Result<Box<dyn Node>, SerializationError> {
        let tree_json: BehaviorTreeJson = serde_json::from_str(json)
            .map_err(|e| SerializationError::ParseError(e.to_string()))?;

        // 构建节点索引
        let mut nodes: HashMap<String, NodeDefinition> = HashMap::new();
        for node_def in tree_json.node_definitions {
            nodes.insert(node_def.id.clone(), node_def);
        }

        // 查找根节点（没有父节点的节点）
        let all_children: std::collections::HashSet<String> = nodes.values()
            .flat_map(|n| n.children.iter().cloned())
            .collect();

        let root_id = nodes.keys()
            .find(|id| !all_children.contains(*id))
            .ok_or_else(|| SerializationError::InvalidNode("No root node found".to_string()))?;

        // 递归构建行为树
        Self::build_node(root_id, &nodes)
    }

    /// 递归构建节点
    fn build_node(
        node_id: &str,
        nodes: &HashMap<String, NodeDefinition>,
    ) -> Result<Box<dyn Node>, SerializationError> {
        let node_def = nodes.get(node_id)
            .ok_or_else(|| SerializationError::MissingChild(node_id.to_string()))?;

        match node_def.node_type {
            NodeType::Sequence => {
                let mut children = Vec::new();
                for child_id in &node_def.children {
                    children.push(Self::build_node(child_id, nodes)?);
                }
                Ok(Box::new(Sequence { children }))
            }
            NodeType::Selector => {
                let mut children = Vec::new();
                for child_id in &node_def.children {
                    children.push(Self::build_node(child_id, nodes)?);
                }
                Ok(Box::new(Selector { children }))
            }
            NodeType::Action => {
                // 创建通用Action节点
                Ok(Box::new(Action))
            }
            _ => {
                // TODO: 实现其他节点类型
                Err(SerializationError::InvalidType(
                    format!("Node type {:?} not yet implemented", node_def.node_type)
                ))
            }
        }
    }
}

/// 行为树序列化器
pub struct BehaviorTreeSerializer;

impl BehaviorTreeSerializer {
    /// 将行为树序列化为JSON字符串
    pub fn to_json(tree: &Box<dyn Node>, name: &str) -> String {
        let node_definitions = Self::extract_node_definitions(tree);

        let tree_json = BehaviorTreeJson {
            version: "1.0".to_string(),
            tree: name.to_string(),
            node_definitions,
        };

        serde_json::to_string_pretty(&tree_json).unwrap()
    }

    /// 提取节点定义（递归）
    fn extract_node_definitions(node: &Box<dyn Node>) -> Vec<NodeDefinition> {
        // TODO: 实现递归提取节点定义
        // 这需要Node trait提供内省能力
        vec![]
    }
}

// ============================================================================
// 示例JSON格式
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_simple_sequence() {
        let json = r#"
        {
            "version": "1.0",
            "tree": "test_tree",
            "nodes": [
                {
                    "id": "root",
                    "type": "sequence",
                    "name": "Root Sequence",
                    "children": ["action1", "action2"],
                    "config": {}
                },
                {
                    "id": "action1",
                    "type": "action",
                    "name": "Action 1",
                    "children": [],
                    "config": {}
                },
                {
                    "id": "action2",
                    "type": "action",
                    "name": "Action 2",
                    "children": [],
                    "config": {}
                }
            ]
        }
        "#;

        let result = BehaviorTreeDeserializer::from_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_selector() {
        let json = r#"
        {
            "version": "1.0",
            "tree": "test_tree",
            "nodes": [
                {
                    "id": "root",
                    "type": "selector",
                    "name": "Root Selector",
                    "children": ["action1", "action2"],
                    "config": {}
                },
                {
                    "id": "action1",
                    "type": "action",
                    "name": "Action 1",
                    "children": [],
                    "config": {}
                },
                {
                    "id": "action2",
                    "type": "action",
                    "name": "Action 2",
                    "children": [],
                    "config": {}
                }
            ]
        }
        "#;

        let result = BehaviorTreeDeserializer::from_json(json);
        assert!(result.is_ok());
    }
}
