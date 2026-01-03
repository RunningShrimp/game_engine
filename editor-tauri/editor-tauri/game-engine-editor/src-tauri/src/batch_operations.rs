// Batch Operations for Game Engine Editor
// Provides efficient batch processing for entity operations

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use tauri::State;

/// Result of a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkEditResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<FailedOperation>,
    pub skipped: Vec<String>,
    pub total_affected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedOperation {
    pub id: String,
    pub error: String,
}

/// Options for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationOptions {
    pub confirm_threshold: Option<usize>,
    pub undo_name: Option<String>,
}

/// Rename pattern for batch renaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePattern {
    pub mode: String, // "prefix", "suffix", "replace", "number"
    pub value: String,
    pub start_number: Option<usize>,
    pub padding: Option<usize>,
}

/// Material batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialBatchOperation {
    pub mode: String, // "apply", "replace", "modify"
    pub material_id: Option<String>,
    pub old_material_id: Option<String>,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// Component batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentBatchOperation {
    pub mode: String, // "add", "remove", "modify", "toggle"
    pub component_type: String,
    pub properties: Option<HashMap<String, serde_json::Value>>,
    pub enabled: Option<bool>,
}

/// Alignment options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentOptions {
    pub axis: String, // "x", "y", "z", "all"
    pub mode: String, // "min", "max", "center", "grid", "distribute"
    pub target: Option<String>,
    pub spacing: Option<f32>,
}

/// Distribution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionOptions {
    pub axis: String, // "x", "y", "z"
    pub mode: String, // "equal", "custom"
    pub spacing: Option<f32>,
    pub bounds: Option<DistributionBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionBounds {
    pub start: f32,
    pub end: f32,
}

/// Entity data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub transform: Transform,
    pub visible: bool,
    pub locked: bool,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub scale: [f32; 3],
}

/// Batch Operations Manager
pub struct BatchOperationsManager {
    entities: HashMap<String, Entity>,
}

impl BatchOperationsManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    /// Set entities for operations
    pub fn set_entities(&mut self, entities: HashMap<String, Entity>) {
        self.entities = entities;
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Batch delete entities
    pub fn batch_delete(
        &mut self,
        ids: &[String],
        _options: Option<BatchOperationOptions>,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        for id in ids {
            if let Some(entity) = self.entities.get(id) {
                if entity.locked {
                    result.skipped.push(id.clone());
                } else {
                    self.entities.remove(id);
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch rename entities
    pub fn batch_rename(&mut self, ids: &[String], pattern: &RenamePattern) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        let start_number = pattern.start_number.unwrap_or(1);
        let padding = pattern.padding.unwrap_or(3);

        for (index, id) in ids.iter().enumerate() {
            if let Some(entity) = self.entities.get_mut(id) {
                let new_name = match pattern.mode.as_str() {
                    "prefix" => format!("{}_{}", pattern.value, entity.name),
                    "suffix" => format!("{}_{}", entity.name, pattern.value),
                    "replace" => pattern.value.clone(),
                    "number" => {
                        let number = start_number + index;
                        format!("{}_{:0width$}", pattern.value, number, width = padding)
                    }
                    _ => entity.name.clone(),
                };

                entity.name = new_name;
                result.succeeded.push(id.clone());
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch move entities
    pub fn batch_move(
        &mut self,
        ids: &[String],
        offset: [f32; 3],
        _options: Option<BatchOperationOptions>,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        for id in ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.locked {
                    result.skipped.push(id.clone());
                } else {
                    entity.transform.position[0] += offset[0];
                    entity.transform.position[1] += offset[1];
                    entity.transform.position[2] += offset[2];
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch rotate entities
    pub fn batch_rotate(
        &mut self,
        ids: &[String],
        rotation: [f32; 3], // Euler angles in radians
        _space: &str,       // "world" or "local"
        _options: Option<BatchOperationOptions>,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        for id in ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.locked {
                    result.skipped.push(id.clone());
                } else {
                    // Simplified rotation - in real implementation use proper quaternion math
                    apply_rotation(&mut entity.transform.rotation, rotation);
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch scale entities
    pub fn batch_scale(
        &mut self,
        ids: &[String],
        scale: [f32; 3],
        _options: Option<BatchOperationOptions>,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        for id in ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.locked {
                    result.skipped.push(id.clone());
                } else {
                    entity.transform.scale[0] *= scale[0];
                    entity.transform.scale[1] *= scale[1];
                    entity.transform.scale[2] *= scale[2];
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch toggle visibility
    pub fn batch_toggle_visibility(
        &mut self,
        ids: &[String],
        visible: bool,
        _options: Option<BatchOperationOptions>,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        for id in ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.visible == visible {
                    result.skipped.push(id.clone());
                } else {
                    entity.visible = visible;
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch toggle locked
    pub fn batch_toggle_locked(
        &mut self,
        ids: &[String],
        locked: bool,
        _options: Option<BatchOperationOptions>,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        for id in ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.locked == locked {
                    result.skipped.push(id.clone());
                } else {
                    entity.locked = locked;
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Batch apply material
    pub fn batch_apply_material(
        &mut self,
        _ids: &[String],
        _operation: &MaterialBatchOperation,
    ) -> BulkEditResult {
        // Placeholder - implement based on your material system
        BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        }
    }

    /// Batch component operation
    pub fn batch_component_operation(
        &mut self,
        _ids: &[String],
        _operation: &ComponentBatchOperation,
    ) -> BulkEditResult {
        // Placeholder - implement based on your component system
        BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        }
    }

    /// Align entities
    pub fn align_entities(&mut self, ids: &[String], options: &AlignmentOptions) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        if ids.is_empty() {
            return result;
        }

        // Get target value from first entity or specified target
        let target_id = options.target.as_ref().unwrap_or(&ids[0]);
        let target_value = if let Some(target_entity) = self.entities.get(target_id) {
            match options.axis.as_str() {
                "x" => target_entity.transform.position[0],
                "y" => target_entity.transform.position[1],
                "z" => target_entity.transform.position[2],
                _ => return result,
            }
        } else {
            return result;
        };

        for id in ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.locked {
                    result.skipped.push(id.clone());
                } else {
                    match options.axis.as_str() {
                        "x" => entity.transform.position[0] = target_value,
                        "y" => entity.transform.position[1] = target_value,
                        "z" => entity.transform.position[2] = target_value,
                        _ => {}
                    }
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Distribute entities
    pub fn distribute_entities(
        &mut self,
        ids: &[String],
        options: &DistributionOptions,
    ) -> BulkEditResult {
        let mut result = BulkEditResult {
            succeeded: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            total_affected: 0,
        };

        if ids.len() < 2 {
            return result;
        }

        // Sort entities by axis
        let axis_index = match options.axis.as_str() {
            "x" => 0,
            "y" => 1,
            "z" => 2,
            _ => return result,
        };

        let mut sorted_ids: Vec<&String> = ids.iter().collect();
        sorted_ids.sort_by(|a, b| {
            let pos_a = self.entities.get(*a).map(|e| e.transform.position[axis_index]);
            let pos_b = self.entities.get(*b).map(|e| e.transform.position[axis_index]);
            pos_a.partial_cmp(&pos_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let first_pos = self
            .entities
            .get(sorted_ids[0])
            .map(|e| e.transform.position[axis_index])
            .unwrap_or(0.0);

        let last_pos = self
            .entities
            .get(sorted_ids[sorted_ids.len() - 1])
            .map(|e| e.transform.position[axis_index])
            .unwrap_or(0.0);

        let spacing = if let Some(bounds) = &options.bounds {
            (bounds.end - bounds.start) / (sorted_ids.len() - 1) as f32
        } else {
            (last_pos - first_pos) / (sorted_ids.len() - 1) as f32
        };

        let spacing = options.spacing.unwrap_or(spacing);

        for (index, id) in sorted_ids.iter().enumerate() {
            if let Some(entity) = self.entities.get_mut(id) {
                if entity.locked {
                    result.skipped.push(id.clone());
                } else {
                    entity.transform.position[axis_index] = first_pos + spacing * index as f32;
                    result.succeeded.push(id.clone());
                }
            } else {
                result.failed.push(FailedOperation {
                    id: id.clone(),
                    error: "Entity not found".to_string(),
                });
            }
        }

        result.total_affected = result.succeeded.len();
        result
    }

    /// Get all entities
    pub fn get_all_entities(&self) -> &HashMap<String, Entity> {
        &self.entities
    }
}

impl Default for BatchOperationsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply rotation to quaternion (simplified)
fn apply_rotation(quaternion: &mut [f32; 4], euler: [f32; 3]) {
    // Simplified rotation application
    // In a real implementation, use proper quaternion math
    let cos_x = (euler[0] / 2.0).cos();
    let sin_x = (euler[0] / 2.0).sin();
    let cos_y = (euler[1] / 2.0).cos();
    let sin_y = (euler[1] / 2.0).sin();
    let cos_z = (euler[2] / 2.0).cos();
    let sin_z = (euler[2] / 2.0).sin();

    // Create rotation quaternion from Euler angles
    let rot_x = [sin_x, 0.0, 0.0, cos_x];
    let rot_y = [0.0, sin_y, 0.0, cos_y];
    let rot_z = [0.0, 0.0, sin_z, cos_z];

    // Multiply quaternions (simplified)
    let temp = multiply_quaternions(quaternion, &rot_x);
    let temp2 = multiply_quaternions(&temp, &rot_y);
    *quaternion = multiply_quaternions(&temp2, &rot_z);
}

/// Multiply two quaternions
fn multiply_quaternions(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_delete() {
        let mut manager = BatchOperationsManager::new();
        let mut entities = HashMap::new();

        entities.insert(
            "1".to_string(),
            Entity {
                id: "1".to_string(),
                name: "Entity1".to_string(),
                transform: Transform {
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                visible: true,
                locked: false,
                enabled: Some(true),
            },
        );

        manager.set_entities(entities);
        let result = manager.batch_delete(&["1".to_string()], None);

        assert_eq!(result.succeeded.len(), 1);
        assert_eq!(result.total_affected, 1);
    }

    #[test]
    fn test_batch_rename() {
        let mut manager = BatchOperationsManager::new();
        let mut entities = HashMap::new();

        entities.insert(
            "1".to_string(),
            Entity {
                id: "1".to_string(),
                name: "Entity1".to_string(),
                transform: Transform {
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                visible: true,
                locked: false,
                enabled: Some(true),
            },
        );

        manager.set_entities(entities);

        let pattern = RenamePattern {
            mode: "prefix".to_string(),
            value: "Test".to_string(),
            start_number: None,
            padding: None,
        };

        let result = manager.batch_rename(&["1".to_string()], &pattern);

        assert_eq!(result.succeeded.len(), 1);
        let entity = manager.get_entity("1").unwrap();
        assert_eq!(entity.name, "Test_Entity1");
    }
}

// ==================== Tauri Commands ====================

/// Batch delete entities
#[tauri::command]
pub async fn batch_delete(
    ids: Vec<String>,
    options: Option<BatchOperationOptions>,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_delete(&ids, options))
}

/// Batch rename entities
#[tauri::command]
pub async fn batch_rename(
    ids: Vec<String>,
    pattern: RenamePattern,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_rename(&ids, &pattern))
}

/// Batch move entities
#[tauri::command]
pub async fn batch_move(
    ids: Vec<String>,
    offset: [f32; 3],
    options: Option<BatchOperationOptions>,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_move(&ids, offset, options))
}

/// Batch rotate entities
#[tauri::command]
pub async fn batch_rotate(
    ids: Vec<String>,
    rotation: [f32; 3],
    space: String,
    options: Option<BatchOperationOptions>,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_rotate(&ids, rotation, &space, options))
}

/// Batch scale entities
#[tauri::command]
pub async fn batch_scale(
    ids: Vec<String>,
    scale: [f32; 3],
    options: Option<BatchOperationOptions>,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_scale(&ids, scale, options))
}

/// Batch toggle visibility
#[tauri::command]
pub async fn batch_toggle_visibility(
    ids: Vec<String>,
    visible: bool,
    options: Option<BatchOperationOptions>,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_toggle_visibility(&ids, visible, options))
}

/// Batch toggle locked
#[tauri::command]
pub async fn batch_toggle_locked(
    ids: Vec<String>,
    locked: bool,
    options: Option<BatchOperationOptions>,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_toggle_locked(&ids, locked, options))
}

/// Batch apply material
#[tauri::command]
pub async fn batch_apply_material(
    ids: Vec<String>,
    operation: MaterialBatchOperation,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_apply_material(&ids, &operation))
}

/// Batch component operation
#[tauri::command]
pub async fn batch_component_operation(
    ids: Vec<String>,
    operation: ComponentBatchOperation,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.batch_component_operation(&ids, &operation))
}

/// Align entities
#[tauri::command]
pub async fn align_entities(
    ids: Vec<String>,
    options: AlignmentOptions,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.align_entities(&ids, &options))
}

/// Distribute entities
#[tauri::command]
pub async fn distribute_entities(
    ids: Vec<String>,
    options: DistributionOptions,
    manager: State<'_, Mutex<BatchOperationsManager>>,
) -> Result<BulkEditResult, String> {
    let mut manager = manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.distribute_entities(&ids, &options))
}
