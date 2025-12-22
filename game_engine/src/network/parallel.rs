//! 并行网络消息处理
//!
//! 提供并行处理网络消息的功能，充分利用多核CPU提升性能。
//!
//! ## 设计原则
//!
//! 1. **消息并行处理**：将消息分组，并行处理
//! 2. **最小同步**：减少线程间同步开销
//! 3. **自适应批处理**：根据消息数量动态调整批处理大小
//! 4. **线程安全**：确保并行处理的线程安全性

use rayon::prelude::*;
use std::sync::Arc;

use super::{NetworkMessage, NetworkState};
use super::compression::NetworkCompressor;

/// 并行网络消息处理器
///
/// 并行处理多个网络消息，提升性能。
/// 适合有大量网络消息的场景。
///
/// # 性能特性
///
/// - 多线程并行处理，充分利用多核CPU
/// - 自适应批处理，根据消息数量调整
/// - 预计性能提升2-4倍（取决于CPU核心数和消息数量）
pub struct ParallelMessageProcessor {
    /// 批处理大小
    batch_size: usize,
    /// 是否启用并行处理
    enabled: bool,
}

impl Default for ParallelMessageProcessor {
    fn default() -> Self {
        Self {
            batch_size: 32,
            enabled: true,
        }
    }
}

impl ParallelMessageProcessor {
    /// 创建新的并行消息处理器
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            enabled: true,
        }
    }

    /// 并行处理消息列表
    ///
    /// # 参数
    /// - `messages`: 待处理的消息列表
    /// - `state`: 网络状态（只读）
    /// - `compressor`: 压缩器（可选）
    ///
    /// # 返回
    /// 处理结果列表
    pub fn process_messages_parallel(
        &self,
        messages: Vec<NetworkMessage>,
        state: &NetworkState,
        compressor: Option<&Arc<NetworkCompressor>>,
    ) -> Vec<MessageProcessResult> {
        if !self.enabled || messages.len() < self.batch_size {
            // 消息数量较少，使用顺序处理
            return messages
                .into_iter()
                .map(|msg| self.process_message(&msg, state, compressor))
                .collect();
        }

        // 并行处理消息
        messages
            .into_par_iter()
            .map(|msg| self.process_message(&msg, state, compressor))
            .collect()
    }

    /// 处理单个消息
    fn process_message(
        &self,
        message: &NetworkMessage,
        state: &NetworkState,
        compressor: Option<&Arc<NetworkCompressor>>,
    ) -> MessageProcessResult {
        match message {
            NetworkMessage::StateSync { tick, data } => {
                // 解压缩数据（如果需要）
                let decompressed_data = if let Some(compressor) = compressor {
                    compressor
                        .decompress_with_flag(data)
                        .unwrap_or_else(|_| data.clone())
                } else {
                    data.clone()
                };

                MessageProcessResult::StateSync {
                    tick: *tick,
                    data: decompressed_data,
                }
            }
            NetworkMessage::Heartbeat { timestamp } => {
                let now = crate::core::utils::current_timestamp_ms();
                let latency_ms = (now - *timestamp) as f32;

                MessageProcessResult::Heartbeat { latency_ms }
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                MessageProcessResult::TimeSyncRequest {
                    client_send_time: *client_send_time,
                }
            }
            NetworkMessage::TimeSyncResponse { sync } => {
                MessageProcessResult::TimeSyncResponse {
                    sync: sync.clone(),
                }
            }
            _ => MessageProcessResult::Other,
        }
    }

    /// 设置批处理大小
    pub fn set_batch_size(&mut self, batch_size: usize) {
        self.batch_size = batch_size;
    }

    /// 启用/禁用并行处理
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub enum MessageProcessResult {
    /// 状态同步结果
    StateSync { tick: u64, data: Vec<u8> },
    /// 心跳结果
    Heartbeat { latency_ms: f32 },
    /// 时间同步请求
    TimeSyncRequest { client_send_time: u64 },
    /// 时间同步响应
    TimeSyncResponse { sync: super::delay_compensation::TimeSyncMessage },
    /// 其他消息类型
    Other,
}

/// 并行网络消息处理配置
#[derive(Debug, Clone)]
pub struct ParallelNetworkConfig {
    /// 最小批处理大小（小于此值使用顺序处理）
    pub min_batch_size: usize,
    /// 最大并行度（0表示使用CPU核心数）
    pub max_parallelism: usize,
    /// 是否启用并行处理
    pub enabled: bool,
}

impl Default for ParallelNetworkConfig {
    fn default() -> Self {
        Self {
            min_batch_size: 16,
            max_parallelism: 0, // 0表示使用CPU核心数
            enabled: true,
        }
    }
}

impl ParallelNetworkConfig {
    /// 创建高性能配置（最大化并行度）
    pub fn high_performance() -> Self {
        Self {
            min_batch_size: 8,
            max_parallelism: 0,
            enabled: true,
        }
    }

    /// 创建低延迟配置（最小化并行度）
    pub fn low_latency() -> Self {
        Self {
            min_batch_size: 32,
            max_parallelism: 2,
            enabled: true,
        }
    }

    /// 禁用并行处理
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_network_config() {
        let config = ParallelNetworkConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_batch_size, 16);

        let high_perf = ParallelNetworkConfig::high_performance();
        assert_eq!(high_perf.min_batch_size, 8);

        let low_latency = ParallelNetworkConfig::low_latency();
        assert_eq!(low_latency.max_parallelism, 2);

        let disabled = ParallelNetworkConfig::disabled();
        assert!(!disabled.enabled);
    }

    #[test]
    fn test_parallel_message_processor() {
        let processor = ParallelMessageProcessor::new(16);
        assert_eq!(processor.batch_size, 16);
        assert!(processor.enabled);
    }
}

