//! 增强的增量序列化模块
//!
//! 提供优化的增量序列化功能：
//! - 字段级别的增量压缩
//! - 量化压缩（减少浮点数精度）
//! - 差分编码
//! - 批量优化序列化

use crate::network::delta_serialization::{DeltaPacket, DeltaSerializer, EntityDelta};
use crate::network::NetworkError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 量化配置
#[derive(Debug, Clone, Copy)]
pub struct QuantizationConfig {
    /// 位置量化精度（单位：米）
    pub position_precision: f32,
    /// 旋转量化精度（单位：度）
    pub rotation_precision: f32,
    /// 速度量化精度（单位：米/秒）
    pub velocity_precision: f32,
    /// 缩放量化精度
    pub scale_precision: f32,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            position_precision: 0.01,  // 1cm精度
            rotation_precision: 0.1,    // 0.1度精度
            velocity_precision: 0.1,    // 0.1m/s精度
            scale_precision: 0.01,
        }
    }
}

/// 量化器
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
        // 简化的四元数到欧拉角转换（仅用于量化）
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

/// 四元数转欧拉角（简化实现）
fn quaternion_to_euler(quat: [f32; 4]) -> (f32, f32, f32) {
    let (w, x, y, z) = (quat[3], quat[0], quat[1], quat[2]);
    
    // Roll (x-axis rotation)
    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr_cosp.atan2(cosr_cosp);

    // Pitch (y-axis rotation)
    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        (std::f32::consts::PI / 2.0).copysign(sinp)
    } else {
        sinp.asin()
    };

    // Yaw (z-axis rotation)
    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny_cosp.atan2(cosy_cosp);

    (yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees())
}

/// 欧拉角转四元数
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

/// 量化的实体增量
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 增强的增量序列化器
pub struct EnhancedDeltaSerializer {
    base_serializer: DeltaSerializer,
    quantizer: Quantizer,
    /// 上次发送的量化状态（用于差分编码）
    last_quantized_state: HashMap<u64, QuantizedEntityDelta>,
}

impl EnhancedDeltaSerializer {
    /// 创建增强的增量序列化器
    pub fn new(config: QuantizationConfig) -> Self {
        Self {
            base_serializer: DeltaSerializer::new(),
            quantizer: Quantizer::new(config),
            last_quantized_state: HashMap::new(),
        }
    }

    /// 量化实体增量
    pub fn quantize_delta(&self, delta: &EntityDelta) -> QuantizedEntityDelta {
        QuantizedEntityDelta {
            id: delta.id,
            position: delta.position.map(|p| self.quantizer.quantize_position(p)),
            rotation: delta.rotation.map(|r| self.quantizer.quantize_rotation(r)),
            velocity: delta.velocity.map(|v| self.quantizer.quantize_velocity(v)),
            scale: delta.scale,
            custom_data: delta.custom_data.clone(),
        }
    }

    /// 反量化实体增量
    pub fn dequantize_delta(&self, quantized: &QuantizedEntityDelta) -> EntityDelta {
        EntityDelta {
            id: quantized.id,
            position: quantized.position.map(|p| self.quantizer.dequantize_position(p)),
            rotation: quantized.rotation.map(|r| self.quantizer.dequantize_rotation(r)),
            velocity: quantized.velocity.map(|v| self.quantizer.dequantize_velocity(v)),
            scale: quantized.scale,
            custom_data: quantized.custom_data.clone(),
        }
    }

    /// 计算差分编码的增量（只发送变化的部分）
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

    /// 处理实体增量列表（量化+差分编码）
    pub fn process_deltas(&mut self, deltas: &[EntityDelta]) -> Vec<QuantizedEntityDelta> {
        deltas
            .iter()
            .map(|delta| {
                let quantized = self.quantize_delta(delta);
                self.compute_differential_delta(&quantized)
            })
            .collect()
    }

    /// 序列化量化的增量数据包
    pub fn serialize_quantized(
        &self,
        quantized_deltas: &[QuantizedEntityDelta],
        sequence: u64,
        baseline_sequence: u64,
    ) -> Result<Vec<u8>, NetworkError> {
        let packet = QuantizedDeltaPacket {
            sequence,
            baseline_sequence,
            deltas: quantized_deltas.to_vec(),
            timestamp_ms: crate::core::utils::current_timestamp_ms() as u64,
        };

        bincode::serialize(&packet).map_err(|e| {
            NetworkError::SerializationError(format!("Quantized serialization failed: {}", e))
        })
    }

    /// 反序列化量化的增量数据包
    pub fn deserialize_quantized(&self, data: &[u8]) -> Result<Vec<EntityDelta>, NetworkError> {
        let packet: QuantizedDeltaPacket = bincode::deserialize(data).map_err(|e| {
            NetworkError::SerializationError(format!("Quantized deserialization failed: {}", e))
        })?;

        Ok(packet
            .deltas
            .iter()
            .map(|q| self.dequantize_delta(q))
            .collect())
    }
}

/// 量化的增量数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedDeltaPacket {
    /// 序列号
    pub sequence: u64,
    /// 基准序列号
    pub baseline_sequence: u64,
    /// 量化的实体增量列表
    pub deltas: Vec<QuantizedEntityDelta>,
    /// 时间戳
    pub timestamp_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization() {
        let config = QuantizationConfig::default();
        let quantizer = Quantizer::new(config);

        let pos = [1.234, 2.567, 3.890];
        let quantized = quantizer.quantize_position(pos);
        let dequantized = quantizer.dequantize_position(quantized);

        // 检查量化误差在精度范围内
        assert!((pos[0] - dequantized[0]).abs() < config.position_precision);
        assert!((pos[1] - dequantized[1]).abs() < config.position_precision);
        assert!((pos[2] - dequantized[2]).abs() < config.position_precision);
    }

    #[test]
    fn test_differential_encoding() {
        let config = QuantizationConfig::default();
        let mut serializer = EnhancedDeltaSerializer::new(config);

        let delta1 = EntityDelta {
            id: 1,
            position: Some([1.0, 2.0, 3.0]),
            rotation: Some([0.0, 0.0, 0.0, 1.0]),
            ..Default::default()
        };

        let delta2 = EntityDelta {
            id: 1,
            position: Some([1.0, 2.0, 3.0]), // 未变化
            rotation: Some([0.1, 0.0, 0.0, 1.0]), // 变化
            ..Default::default()
        };

        let quantized1 = serializer.quantize_delta(&delta1);
        let quantized2 = serializer.quantize_delta(&delta2);

        let diff1 = serializer.compute_differential_delta(&quantized1);
        let diff2 = serializer.compute_differential_delta(&quantized2);

        // 第一次应该包含所有数据
        assert!(diff1.position.is_some());
        assert!(diff1.rotation.is_some());

        // 第二次应该只包含变化的数据
        assert!(diff2.position.is_none()); // 位置未变化
        assert!(diff2.rotation.is_some()); // 旋转变化
    }
}

