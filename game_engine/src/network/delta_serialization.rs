//  增量序列化模块
//
//  实现网络数据的增量序列化协议，只传输变化的数据以减少网络带宽使用。
//
//  ## 设计原理
//
//  增量序列化通过比较当前状态和基准状态，只序列化变化的部分：
//
//  ```text
//  ┌─────────────────┐         ┌─────────────────┐
//  │   Current State │         │  Baseline State │
//  │                 │         │                 │
//  │  Entity A: pos │  Compare │  Entity A: pos  │
//  │  Entity B: pos │  ──────► │  Entity B: pos  │
//  │  Entity C: pos │         │  Entity C: pos   │
//  └─────────────────┘         └─────────────────┘
//          │                            │
//          └────────────┬───────────────┘
//                       ▼
//               ┌───────────────┐
//               │  Delta Data   │
//               │  (Only Changes)│
//               └───────────────┘
//  ```
//
//  ## 性能优化
//
//  - 减少网络带宽使用 50-80%（取决于变化率）
//  - 支持字段级别的增量更新
//  - 支持批量增量更新
//  - 自动基准状态管理
//
//  ## 使用示例
//
//  ```rust
//  use game_engine::network::{DeltaSerializer, EntityDelta};
//
//  // 创建增量序列化器
//  let mut serializer = DeltaSerializer::new();
//
//  // 设置基准状态
//  let baseline = vec![
//      EntityDelta { id: 1, position: Some([0.0, 0.0, 0.0]), ..Default::default() },
//      EntityDelta { id: 2, position: Some([1.0, 1.0, 1.0]), ..Default::default() },
//  ];
//  serializer.set_baseline(baseline);
//
//  // 计算增量
//  let current = vec![
//      EntityDelta { id: 1, position: Some([0.5, 0.0, 0.0]), ..Default::default() },
//      EntityDelta { id: 2, position: Some([1.0, 1.0, 1.0]), ..Default::default() },
//  ];
//  let delta = serializer.compute_delta(&current);
//
//  // 序列化增量（只包含变化的数据）
//  let serialized = serializer.serialize_delta(&delta)?;
//
//  // 反序列化并应用增量
//  let deserialized = serializer.deserialize_delta(&serialized)?;
//  serializer.apply_delta(&deserialized);
//  ```

use crate::network::NetworkError;
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 量化配置（增强功能）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuantizationConfig {
    /// 位置量化精度（单位：米）
    pub position_precision: f32,
    /// 旋转量化精度（单位：度）
    pub rotation_precision: f32,
    /// 速度量化精度（单位：米/秒）
    pub velocity_precision: f32,
    /// 缩放量化精度
    pub scale_precision: f32,
    /// 是否启用量化
    pub enabled: bool,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            position_precision: 0.01,  // 1cm精度
            rotation_precision: 0.1,    // 0.1度精度
            velocity_precision: 0.1,    // 0.1m/s精度
            scale_precision: 0.01,
            enabled: false, // 默认禁用，保持向后兼容
        }
    }
}

/// 量化器（增强功能）
pub struct Quantizer {
    config: QuantizationConfig,
}

impl Quantizer {
    /// 创建量化器
    pub fn new(config: QuantizationConfig) -> Self {
        Self { config }
    }

    /// 量化位置
    pub fn quantize_position(&self, pos: [f32; 3]) -> [i32; 3] {
        [
            (pos[0] / self.config.position_precision).round() as i32,
            (pos[1] / self.config.position_precision).round() as i32,
            (pos[2] / self.config.position_precision).round() as i32,
        ]
    }

    /// 反量化位置
    pub fn dequantize_position(&self, quantized: [i32; 3]) -> [f32; 3] {
        [
            quantized[0] as f32 * self.config.position_precision,
            quantized[1] as f32 * self.config.position_precision,
            quantized[2] as f32 * self.config.position_precision,
        ]
    }

    /// 量化旋转（四元数转换为欧拉角再量化）
    pub fn quantize_rotation(&self, rot: [f32; 4]) -> [i16; 3] {
        let (yaw, pitch, roll) = quaternion_to_euler(rot);
        [
            (yaw / self.config.rotation_precision).round() as i16,
            (pitch / self.config.rotation_precision).round() as i16,
            (roll / self.config.rotation_precision).round() as i16,
        ]
    }

    /// 反量化旋转
    pub fn dequantize_rotation(&self, quantized: [i16; 3]) -> [f32; 4] {
        let yaw = quantized[0] as f32 * self.config.rotation_precision;
        let pitch = quantized[1] as f32 * self.config.rotation_precision;
        let roll = quantized[2] as f32 * self.config.rotation_precision;
        euler_to_quaternion(yaw, pitch, roll)
    }

    /// 量化速度
    pub fn quantize_velocity(&self, vel: [f32; 3]) -> [i16; 3] {
        [
            (vel[0] / self.config.velocity_precision).round() as i16,
            (vel[1] / self.config.velocity_precision).round() as i16,
            (vel[2] / self.config.velocity_precision).round() as i16,
        ]
    }

    /// 反量化速度
    pub fn dequantize_velocity(&self, quantized: [i16; 3]) -> [f32; 3] {
        [
            quantized[0] as f32 * self.config.velocity_precision,
            quantized[1] as f32 * self.config.velocity_precision,
            quantized[2] as f32 * self.config.velocity_precision,
        ]
    }
}

/// 四元数转欧拉角（辅助函数）
fn quaternion_to_euler(quat: [f32; 4]) -> (f32, f32, f32) {
    let (w, x, y, z) = (quat[3], quat[0], quat[1], quat[2]);
    
    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr_cosp.atan2(cosr_cosp);

    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        (std::f32::consts::PI / 2.0).copysign(sinp)
    } else {
        sinp.asin()
    };

    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny_cosp.atan2(cosy_cosp);

    (yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees())
}

/// 欧拉角转四元数（辅助函数）
fn euler_to_quaternion(yaw: f32, pitch: f32, roll: f32) -> [f32; 4] {
    let (y, p, r) = (yaw.to_radians(), pitch.to_radians(), roll.to_radians());
    
    let cy = (y * 0.5).cos();
    let sy = (y * 0.5).sin();
    let cp = (p * 0.5).cos();
    let sp = (p * 0.5).sin();
    let cr = (r * 0.5).cos();
    let sr = (r * 0.5).sin();

    [
        sr * cp * cy - cr * sp * sy,
        cr * sp * cy + sr * cp * sy,
        cr * cp * sy - sr * sp * cy,
        cr * cp * cy + sr * sp * sy,
    ]
}

/// 实体增量数据
///
/// 只包含变化的数据字段，未变化的字段为None
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EntityDelta {
    /// 实体ID
    pub id: u64,
    /// 位置变化（如果变化）
    pub position: Option<[f32; 3]>,
    /// 旋转变化（如果变化）
    pub rotation: Option<[f32; 4]>,
    /// 缩放变化（如果变化）
    pub scale: Option<[f32; 3]>,
    /// 速度变化（如果变化）
    pub velocity: Option<[f32; 3]>,
    /// 自定义数据变化（如果变化）
    pub custom_data: Option<Vec<u8>>,
}

impl EntityDelta {
    /// 创建新的实体增量
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// 检查是否有任何变化
    pub fn has_changes(&self) -> bool {
        self.position.is_some()
            || self.rotation.is_some()
            || self.scale.is_some()
            || self.velocity.is_some()
            || self.custom_data.is_some()
    }

    /// 计算序列化后的大小（估算）
    pub fn estimated_size(&self) -> usize {
        let mut size = 8; // id (u64)
        if self.position.is_some() {
            size += 12; // 3 * f32
        }
        if self.rotation.is_some() {
            size += 16; // 4 * f32
        }
        if self.scale.is_some() {
            size += 12; // 3 * f32
        }
        if self.velocity.is_some() {
            size += 12; // 3 * f32
        }
        if let Some(ref custom) = self.custom_data {
            size += 4 + custom.len(); // length + data
        }
        size
    }
}

/// 增量数据包
///
/// 包含多个实体的增量数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaPacket {
    /// 序列号（单调递增）
    pub sequence: u64,
    /// 基准序列号（用于引用基准状态）
    pub baseline_sequence: u64,
    /// 实体增量列表
    pub deltas: Vec<EntityDelta>,
    /// 时间戳（毫秒）
    pub timestamp_ms: u64,
}

impl DeltaPacket {
    /// 创建新的增量数据包
    pub fn new(sequence: u64, baseline_sequence: u64) -> Self {
        Self {
            sequence,
            baseline_sequence,
            deltas: Vec::new(),
            timestamp_ms: crate::core::utils::current_timestamp_ms(),
        }
    }

    /// 添加实体增量
    pub fn add_delta(&mut self, delta: EntityDelta) {
        if delta.has_changes() {
            self.deltas.push(delta);
        }
    }

    /// 计算序列化后的大小（估算）
    pub fn estimated_size(&self) -> usize {
        let mut size = 8 + 8 + 8 + 8; // sequence + baseline_sequence + timestamp + deltas length
        for delta in &self.deltas {
            size += delta.estimated_size();
        }
        size
    }
}

/// 增量序列化器
///
/// 管理基准状态和增量计算
pub struct DeltaSerializer {
    /// 基准状态（实体ID -> 完整状态）
    baseline: HashMap<u64, EntityDelta>,
    /// 当前序列号
    current_sequence: u64,
    /// 基准序列号
    baseline_sequence: u64,
    /// 变化阈值（用于浮点数比较）
    change_threshold: f32,
    /// 量化配置（增强功能）
    quantization: Option<QuantizationConfig>,
    /// 量化器（仅在启用量化时使用）
    quantizer: Option<Quantizer>,
    /// 上次发送的量化状态（用于差分编码，增强功能）
    last_quantized_state: HashMap<u64, QuantizedEntityDelta>,
}

/// 量化的实体增量（增强功能）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizedEntityDelta {
    /// 实体ID
    pub id: u64,
    /// 量化位置（如果变化）
    pub position: Option<[i32; 3]>,
    /// 量化旋转（如果变化）
    pub rotation: Option<[i16; 3]>,
    /// 量化速度（如果变化）
    pub velocity: Option<[i16; 3]>,
    /// 缩放（如果变化，保持f32精度）
    pub scale: Option<[f32; 3]>,
    /// 自定义数据（如果变化）
    pub custom_data: Option<Vec<u8>>,
}

impl QuantizedEntityDelta {
    /// 估算序列化大小
    pub fn estimated_size(&self) -> usize {
        let mut size = 8; // id (u64)
        if self.position.is_some() {
            size += 12; // 3 * i32
        }
        if self.rotation.is_some() {
            size += 6; // 3 * i16
        }
        if self.velocity.is_some() {
            size += 6; // 3 * i16
        }
        if self.scale.is_some() {
            size += 12; // 3 * f32
        }
        if let Some(ref custom) = self.custom_data {
            size += 4 + custom.len();
        }
        size
    }
}

impl DeltaSerializer {
    /// 创建新的增量序列化器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带阈值的增量序列化器
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            baseline: HashMap::new(),
            current_sequence: 0,
            baseline_sequence: 0,
            change_threshold: threshold,
            quantization: None,
            quantizer: None,
            last_quantized_state: HashMap::new(),
        }
    }

    /// 创建带量化配置的增量序列化器（增强功能）
    pub fn with_quantization(config: QuantizationConfig) -> Self {
        let quantizer = if config.enabled {
            Some(Quantizer::new(config))
        } else {
            None
        };
        Self {
            baseline: HashMap::new(),
            current_sequence: 0,
            baseline_sequence: 0,
            change_threshold: 0.001,
            quantization: Some(config),
            quantizer,
            last_quantized_state: HashMap::new(),
        }
    }

    /// 启用量化（增强功能）
    pub fn enable_quantization(&mut self, config: QuantizationConfig) {
        self.quantization = Some(config);
        if config.enabled {
            self.quantizer = Some(Quantizer::new(config));
        }
    }

    /// 禁用量化
    pub fn disable_quantization(&mut self) {
        self.quantization = None;
        self.quantizer = None;
        self.last_quantized_state.clear();
    }

    /// 设置基准状态
    pub fn set_baseline(&mut self, entities: Vec<EntityDelta>) {
        self.baseline.clear();
        for entity in entities {
            self.baseline.insert(entity.id, entity);
        }
        self.baseline_sequence = self.current_sequence;
    }

    /// 更新基准状态（合并增量）
    pub fn update_baseline(&mut self, deltas: &[EntityDelta]) {
        for delta in deltas {
            if let Some(baseline) = self.baseline.get_mut(&delta.id) {
                // 合并增量到基准状态
                if let Some(pos) = delta.position {
                    baseline.position = Some(pos);
                }
                if let Some(rot) = delta.rotation {
                    baseline.rotation = Some(rot);
                }
                if let Some(scale) = delta.scale {
                    baseline.scale = Some(scale);
                }
                if let Some(vel) = delta.velocity {
                    baseline.velocity = Some(vel);
                }
                if let Some(ref custom) = delta.custom_data {
                    baseline.custom_data = Some(custom.clone());
                }
            } else {
                // 新实体，直接添加
                self.baseline.insert(delta.id, delta.clone());
            }
        }
        self.baseline_sequence = self.current_sequence;
    }

    /// 计算增量（比较当前状态和基准状态）
    pub fn compute_delta(&mut self, current: &[EntityDelta]) -> DeltaPacket {
        self.current_sequence += 1;
        let mut packet = DeltaPacket::new(self.current_sequence, self.baseline_sequence);

        for entity in current {
            let mut delta = EntityDelta::new(entity.id);
            let baseline = self.baseline.get(&entity.id);

            // 比较位置
            if let Some(current_pos) = entity.position {
                let changed = if let Some(baseline) = baseline {
                    if let Some(baseline_pos) = baseline.position {
                        // 计算距离变化
                        let dx = current_pos[0] - baseline_pos[0];
                        let dy = current_pos[1] - baseline_pos[1];
                        let dz = current_pos[2] - baseline_pos[2];
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        dist_sq > self.change_threshold * self.change_threshold
                    } else {
                        true
                    }
                } else {
                    true
                };
                if changed {
                    delta.position = Some(current_pos);
                }
            }

            // 比较旋转
            if let Some(current_rot) = entity.rotation {
                let changed = if let Some(baseline) = baseline {
                    if let Some(baseline_rot) = baseline.rotation {
                        // 计算四元数差异
                        let dot = current_rot[0] * baseline_rot[0]
                            + current_rot[1] * baseline_rot[1]
                            + current_rot[2] * baseline_rot[2]
                            + current_rot[3] * baseline_rot[3];
                        let angle = (dot.abs().min(1.0)).acos() * 2.0;
                        angle > self.change_threshold
                    } else {
                        true
                    }
                } else {
                    true
                };
                if changed {
                    delta.rotation = Some(current_rot);
                }
            }

            // 比较缩放
            if let Some(current_scale) = entity.scale {
                let changed = if let Some(baseline) = baseline {
                    if let Some(baseline_scale) = baseline.scale {
                        let dx = current_scale[0] - baseline_scale[0];
                        let dy = current_scale[1] - baseline_scale[1];
                        let dz = current_scale[2] - baseline_scale[2];
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        dist_sq > self.change_threshold * self.change_threshold
                    } else {
                        true
                    }
                } else {
                    true
                };
                if changed {
                    delta.scale = Some(current_scale);
                }
            }

            // 比较速度
            if let Some(current_vel) = entity.velocity {
                let changed = if let Some(baseline) = baseline {
                    if let Some(baseline_vel) = baseline.velocity {
                        let dx = current_vel[0] - baseline_vel[0];
                        let dy = current_vel[1] - baseline_vel[1];
                        let dz = current_vel[2] - baseline_vel[2];
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        dist_sq > self.change_threshold * self.change_threshold
                    } else {
                        true
                    }
                } else {
                    true
                };
                if changed {
                    delta.velocity = Some(current_vel);
                }
            }

            // 比较自定义数据（字节级比较）
            if let Some(ref current_custom) = entity.custom_data {
                let changed = if let Some(baseline) = baseline {
                    if let Some(ref baseline_custom) = baseline.custom_data {
                        current_custom != baseline_custom
                    } else {
                        true
                    }
                } else {
                    true
                };
                if changed {
                    delta.custom_data = Some(current_custom.clone());
                }
            }

            packet.add_delta(delta);
        }

        packet
    }

    /// 序列化增量数据包
    pub fn serialize_delta(&self, packet: &DeltaPacket) -> Result<Vec<u8>, NetworkError> {
        bincode::serialize(&packet).map_err(|e| {
            NetworkError::SerializationError(format!("Delta serialization failed: {}", e))
        })
    }

    /// 反序列化增量数据包
    pub fn deserialize_delta(&self, data: &[u8]) -> Result<DeltaPacket, NetworkError> {
        bincode::deserialize::<DeltaPacket>(data).map_err(|e| {
            NetworkError::SerializationError(format!("Delta deserialization failed: {}", e))
        })
    }

    /// 应用增量到基准状态
    pub fn apply_delta(&mut self, packet: &DeltaPacket) {
        self.update_baseline(&packet.deltas);
    }

    /// 获取当前序列号
    pub fn current_sequence(&self) -> u64 {
        self.current_sequence
    }

    /// 获取基准序列号
    pub fn baseline_sequence(&self) -> u64 {
        self.baseline_sequence
    }

    /// 获取基准状态中的实体数量
    pub fn baseline_entity_count(&self) -> usize {
        self.baseline.len()
    }

    /// 清除基准状态
    pub fn clear_baseline(&mut self) {
        self.baseline.clear();
        self.baseline_sequence = self.current_sequence;
    }
}

impl Default for DeltaSerializer {
    fn default() -> Self {
        Self {
            baseline: HashMap::new(),
            current_sequence: 0,
            baseline_sequence: 0,
            change_threshold: 0.001, // 默认阈值：1mm
            quantization: None,
            quantizer: None,
            last_quantized_state: HashMap::new(),
        }
    }
}

impl DeltaSerializer {
    /// 量化实体增量（增强功能）
    pub fn quantize_delta(&self, delta: &EntityDelta) -> Option<QuantizedEntityDelta> {
        let quantizer = self.quantizer.as_ref()?;
        Some(QuantizedEntityDelta {
            id: delta.id,
            position: delta.position.map(|p| quantizer.quantize_position(p)),
            rotation: delta.rotation.map(|r| quantizer.quantize_rotation(r)),
            velocity: delta.velocity.map(|v| quantizer.quantize_velocity(v)),
            scale: delta.scale,
            custom_data: delta.custom_data.clone(),
        })
    }

    /// 反量化实体增量（增强功能）
    pub fn dequantize_delta(&self, quantized: &QuantizedEntityDelta) -> Option<EntityDelta> {
        let quantizer = self.quantizer.as_ref()?;
        Some(EntityDelta {
            id: quantized.id,
            position: quantized.position.map(|p| quantizer.dequantize_position(p)),
            rotation: quantized.rotation.map(|r| quantizer.dequantize_rotation(r)),
            velocity: quantized.velocity.map(|v| quantizer.dequantize_velocity(v)),
            scale: quantized.scale,
            custom_data: quantized.custom_data.clone(),
        })
    }

    /// 计算差分编码的增量（增强功能）
    pub fn compute_differential_delta(
        &mut self,
        current: &QuantizedEntityDelta,
    ) -> QuantizedEntityDelta {
        let mut diff = QuantizedEntityDelta {
            id: current.id,
            position: None,
            rotation: None,
            velocity: None,
            scale: None,
            custom_data: None,
        };

        if let Some(last) = self.last_quantized_state.get(&current.id) {
            // 只包含变化的部分
            if current.position != last.position {
                diff.position = current.position;
            }
            if current.rotation != last.rotation {
                diff.rotation = current.rotation;
            }
            if current.velocity != last.velocity {
                diff.velocity = current.velocity;
            }
            if current.scale != last.scale {
                diff.scale = current.scale;
            }
            if current.custom_data != last.custom_data {
                diff.custom_data = current.custom_data.clone();
            }
        } else {
            // 新实体，包含所有数据
            diff.position = current.position;
            diff.rotation = current.rotation;
            diff.velocity = current.velocity;
            diff.scale = current.scale;
            diff.custom_data = current.custom_data.clone();
        }

        // 更新最后状态
        self.last_quantized_state.insert(current.id, current.clone());

        diff
    }

    /// 处理实体增量列表（量化+差分编码，增强功能）
    pub fn process_deltas_quantized(&mut self, deltas: &[EntityDelta]) -> Vec<QuantizedEntityDelta> {
        if self.quantizer.is_none() {
            return Vec::new();
        }

        deltas
            .iter()
            .filter_map(|delta| {
                self.quantize_delta(delta).map(|quantized| {
                    self.compute_differential_delta(&quantized)
                })
            })
            .collect()
    }
}

/// 批量增量序列化器
///
/// 优化批量实体的增量序列化
pub struct BatchDeltaSerializer {
    serializer: DeltaSerializer,
    batch_size: usize,
}

impl BatchDeltaSerializer {
    /// 创建批量增量序列化器
    pub fn new(batch_size: usize) -> Self {
        Self {
            serializer: DeltaSerializer::new(),
            batch_size,
        }
    }

    /// 批量计算增量
    pub fn compute_batch_delta(&mut self, entities: &[EntityDelta]) -> Vec<DeltaPacket> {
        let mut packets = Vec::new();

        // 分批处理
        for chunk in entities.chunks(self.batch_size) {
            let packet = self.serializer.compute_delta(chunk);
            if !packet.deltas.is_empty() {
                packets.push(packet);
            }
        }

        packets
    }

    /// 批量序列化
    pub fn serialize_batch(&self, packets: &[DeltaPacket]) -> Result<Vec<u8>, NetworkError> {
        bincode::serialize(&packets).map_err(|e| {
            NetworkError::SerializationError(format!("Batch serialization failed: {}", e))
        })
    }

    /// 批量反序列化
    pub fn deserialize_batch(&self, data: &[u8]) -> Result<Vec<DeltaPacket>, NetworkError> {
        bincode::deserialize::<Vec<DeltaPacket>>(data).map_err(|e| {
            NetworkError::SerializationError(format!("Batch deserialization failed: {}", e))
        })
    }

    /// 批量应用增量
    pub fn apply_batch(&mut self, packets: &[DeltaPacket]) {
        for packet in packets {
            self.serializer.apply_delta(packet);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_computation() {
        let mut serializer = DeltaSerializer::new();

        // 设置基准状态
        let baseline = vec![
            EntityDelta {
                id: 1,
                position: Some([0.0, 0.0, 0.0]),
                rotation: Some([0.0, 0.0, 0.0, 1.0]),
                ..Default::default()
            },
            EntityDelta {
                id: 2,
                position: Some([1.0, 1.0, 1.0]),
                ..Default::default()
            },
        ];
        serializer.set_baseline(baseline);

        // 计算增量（只有实体1的位置变化）
        let current = vec![
            EntityDelta {
                id: 1,
                position: Some([0.5, 0.0, 0.0]),      // 变化
                rotation: Some([0.0, 0.0, 0.0, 1.0]), // 未变化
                ..Default::default()
            },
            EntityDelta {
                id: 2,
                position: Some([1.0, 1.0, 1.0]), // 未变化
                ..Default::default()
            },
        ];

        let delta = serializer.compute_delta(&current);

        // 验证增量只包含变化的数据
        assert_eq!(delta.deltas.len(), 1);
        assert_eq!(delta.deltas[0].id, 1);
        assert!(delta.deltas[0].position.is_some());
        assert!(delta.deltas[0].rotation.is_none()); // 未变化，应该为None
    }

    #[test]
    fn test_delta_serialization() {
        let serializer = DeltaSerializer::new();
        let mut packet = DeltaPacket::new(1, 0);
        packet.add_delta(EntityDelta {
            id: 1,
            position: Some([1.0, 2.0, 3.0]),
            ..Default::default()
        });

        // 序列化
        let serialized = serializer.serialize_delta(&packet).unwrap();

        // 反序列化
        let deserialized = serializer.deserialize_delta(&serialized).unwrap();

        assert_eq!(deserialized.sequence, packet.sequence);
        assert_eq!(deserialized.deltas.len(), 1);
        assert_eq!(deserialized.deltas[0].id, 1);
        assert_eq!(deserialized.deltas[0].position, Some([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_batch_delta() {
        let mut batch_serializer = BatchDeltaSerializer::new(10);

        let entities = (0..25)
            .map(|i| EntityDelta {
                id: i,
                position: Some([i as f32, 0.0, 0.0]),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let packets = batch_serializer.compute_batch_delta(&entities);

        // 应该分成3批（10, 10, 5）
        assert_eq!(packets.len(), 3);
    }
}
