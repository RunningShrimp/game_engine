//! Code Completion Provider
//!
//! Provides intelligent code completion suggestions based on
//! the current context and available symbols.

use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::*;
use tracing::debug;

use crate::lsp::api_index::ApiIndex;
use crate::lsp::symbol_info::CompletionItemData;

/// Completion provider for code suggestions
pub struct CompletionProvider {
    /// API index for symbol lookup
    api_index: Option<Arc<Mutex<ApiIndex>>>,

    /// Predefined completion items for common patterns
    predefined_items: Vec<CompletionItem>,
}

impl CompletionProvider {
    /// Create a new CompletionProvider
    pub fn new() -> Self {
        debug!("Creating CompletionProvider");

        let predefined_items = Self::create_predefined_items();

        Self {
            api_index: None,
            predefined_items,
        }
    }

    /// Set the API index
    pub fn set_api_index(&mut self, index: Arc<Mutex<ApiIndex>>) {
        self.api_index = Some(index);
    }

    /// Create predefined completion items for common patterns
    fn create_predefined_items() -> Vec<CompletionItem> {
        vec![
            // Entity creation
            CompletionItem {
                label: "Entity::new()".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create a new entity".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Creates a new Entity with a unique ID.\n\n# Example\n\n```rust\nlet entity = Entity::new();\n```".to_string(),
                })),
                ..Default::default()
            },
            // Transform creation
            CompletionItem {
                label: "Transform::default()".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Create default transform".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Creates a Transform at origin with no rotation and unit scale.".to_string(),
                })),
                ..Default::default()
            },
            // Common imports
            CompletionItem {
                label: "use game_engine::".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Import from game engine".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Import modules from the game engine crate.".to_string(),
                })),
                ..Default::default()
            },
            // Game loop
            CompletionItem {
                label: "fn update(&mut self, dt: f32)".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Game update function".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Standard game update function.\n\n# Parameters\n\n- `dt`: Delta time in seconds".to_string(),
                })),
                insert_text: Some("fn update(&mut self, dt: f32) {\n    $0\n}".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Provide completion items for the given position
    pub async fn provide_completion(
        &self,
        uri: &Url,
        position: Position,
    ) -> anyhow::Result<Vec<CompletionItem>> {
        debug!("Providing completion for {:?} at {:?}", uri, position);

        let mut items = Vec::new();

        // Add predefined items
        items.extend(self.predefined_items.clone());

        // 添加上下文感知的补全
        if let Some(index) = &self.api_index {
            let index_guard = index.lock().await;

            // 从文档获取基本上下文（简化实现）
            let line_text = params
                .text_document_position
                .text_document
                .uri
                .path()
                .and_then(|p| p.to_str())
                .unwrap_or("");

            // 根据文件路径提供上下文相关的补全
            let is_engine_file = line_text.contains("game_engine") || line_text.contains("engine");

            if is_engine_file {
                // 引擎特定的补全

            // Core types
            items.push(CompletionItem {
                label: "Entity".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Game entity".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Represents a game entity with a unique ID.".to_string(),
                })),
                ..Default::default()
            });

            items.push(CompletionItem {
                label: "Transform".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Transform component".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Transform component for position, rotation, and scale.".to_string(),
                })),
                ..Default::default()
            });

            items.push(CompletionItem {
                label: "Vec3".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("3D vector".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "3-dimensional vector for positions, directions, etc.".to_string(),
                })),
                ..Default::default()
            });

            items.push(CompletionItem {
                label: "Quat".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Quaternion".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Quaternion for representing rotations.".to_string(),
                })),
                ..Default::default()
            });

            // Component methods
            items.push(CompletionItem {
                label: "add_component".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Add component to entity".to_string()),
                insert_text: Some("add_component::<${1:T}>($0)".to_string()),
                ..Default::default()
            });

            items.push(CompletionItem {
                label: "get_component".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Get component from entity".to_string()),
                insert_text: Some("get_component::<${1:T}>()".to_string()),
                ..Default::default()
            });
        }

        Ok(items)
    }

    /// Resolve a completion item (called when an item is selected)
    pub async fn resolve_completion(
        &self,
        item: CompletionItem,
    ) -> anyhow::Result<CompletionItem> {
        debug!("Resolving completion item: {}", item.label);
        // For now, just return the item as-is
        // In the future, we could add more details here
        Ok(item)
    }

    /// Get completions for struct fields
    pub async fn get_field_completions(&self, type_name: &str) -> Vec<CompletionItem> {
        debug!("Getting field completions for type: {}", type_name);

        // API索引类型字段查找（简化版本）
        // For now, return common fields for known types

        match type_name {
            "Transform" => vec![
                CompletionItem {
                    label: "position".to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("Vec3".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "rotation".to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("Quat".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "scale".to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("Vec3".to_string()),
                    ..Default::default()
                },
            ],
            "Vec3" => vec![
                CompletionItem {
                    label: "x".to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("f32".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "y".to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("f32".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "z".to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("f32".to_string()),
                    ..Default::default()
                },
            ],
            _ => vec![],
        }
    }

    /// Get completions for methods
    pub async fn get_method_completions(&self, type_name: &str) -> Vec<CompletionItem> {
        debug!("Getting method completions for type: {}", type_name);

        // API索引类型方法查找（简化版本）
        // For now, return common methods for known types

        match type_name {
            "Entity" => vec![
                CompletionItem {
                    label: "id()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("EntityId".to_string()),
                    documentation: Some(Documentation::String("Get the entity's unique ID".to_string())),
                    ..Default::default()
                },
                CompletionItem {
                    label: "is_valid()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("bool".to_string()),
                    documentation: Some(Documentation::String("Check if the entity is still valid".to_string())),
                    ..Default::default()
                },
            ],
            "Transform" => vec![
                CompletionItem {
                    label: "translate()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("fn(&mut self, offset: Vec3)".to_string()),
                    documentation: Some(Documentation::String("Translate the transform".to_string())),
                    ..Default::default()
                },
                CompletionItem {
                    label: "rotate()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("fn(&mut self, rotation: Quat)".to_string()),
                    documentation: Some(Documentation::String("Rotate the transform".to_string())),
                    ..Default::default()
                },
                CompletionItem {
                    label: "scale()".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("fn(&mut self, factor: Vec3)".to_string()),
                    documentation: Some(Documentation::String("Scale the transform".to_string())),
                    ..Default::default()
                },
            ],
            _ => vec![],
        }
    }
}
