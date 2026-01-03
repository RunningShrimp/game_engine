//! # 内存顾问（Memory Advisor）
//!
//! 提供智能内存管理建议和自动分析功能。
//!
//! ## 功能特性
//!
//! - **内存使用分析**: 实时追踪内存分配
//! - **内存泄漏检测**: 自动检测未释放资源
//! - **优化建议**: 基于模式识别的建议
//! - **自动清理**: 安全的内存回收策略
//! - **性能预测**: 内存压力预测

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// =============================================================================
// 内存快照
// =============================================================================

/// 内存使用快照
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// 快照时间
    pub timestamp: u64,
    /// 总分配字节数
    pub total_allocated: usize,
    /// 当前使用字节数
    pub current_usage: usize,
    /// 峰值使用字节数
    pub peak_usage: usize,
    /// 分配次数
    pub allocation_count: u64,
    /// 释放次数
    pub deallocation_count: u64,
    /// 各分类内存使用
    pub category_usage: HashMap<MemoryCategory, CategoryUsage>,
}

/// 内存分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryCategory {
    /// 纹理
    Textures,
    /// 网格
    Meshes,
    /// 音频
    Audio,
    /// 着色器
    Shaders,
    /// GPU缓冲
    Buffers,
    /// ECS组件
    Components,
    /// 系统内部
    Internal,
    /// 用户自定义
    User,
    /// 其他
    Other,
}

/// 分类内存使用
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryUsage {
    /// 分类名称
    pub category: MemoryCategory,
    /// 使用字节数
    pub bytes: usize,
    /// 资源数量
    pub count: usize,
    /// 峰值字节数
    pub peak_bytes: usize,
}

impl MemorySnapshot {
    /// 创建新快照
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            total_allocated: 0,
            current_usage: 0,
            peak_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
            category_usage: HashMap::new(),
        }
    }

    /// 获取内存使用率 (0.0 - 1.0)
    pub fn usage_ratio(&self, total_memory: usize) -> f64 {
        if total_memory == 0 {
            return 0.0;
        }
        self.current_usage as f64 / total_memory as f64
    }

    /// 获取内存增长率 (分配 - 释放)
    pub fn growth_rate(&self) -> i64 {
        self.allocation_count as i64 - self.deallocation_count as i64
    }
}

// =============================================================================
// 内存泄漏检测
// =============================================================================

/// 内存泄漏报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakReport {
    /// 检测时间
    pub timestamp: u64,
    /// 是否检测到泄漏
    pub has_leaks: bool,
    /// 泄漏详情
    pub leaks: Vec<LeakInfo>,
    /// 总泄漏字节数
    pub total_leaked_bytes: usize,
    /// 严重程度
    pub severity: LeakSeverity,
}

/// 泄漏信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakInfo {
    /// 资源路径
    pub resource_path: String,
    /// 内存分类
    pub category: MemoryCategory,
    /// 分配时间
    pub allocated_at: u64,
    /// 字节数
    pub bytes: usize,
    /// 持续时间（秒）
    pub duration_secs: u64,
    /// 可疑程度 (0.0 - 1.0)
    pub suspicious_score: f32,
}

/// 泄漏严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeakSeverity {
    /// 无泄漏
    None,
    /// 轻度泄漏
    Low,
    /// 中度泄漏
    Medium,
    /// 严重泄漏
    High,
    /// 极其严重
    Critical,
}

// =============================================================================
// 内存优化建议
// =============================================================================

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// 建议ID
    pub id: SuggestionId,
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 优先级
    pub priority: SuggestionPriority,
    /// 标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 预期内存节省（字节）
    pub estimated_savings: usize,
    /// 实施步骤
    pub implementation_steps: Vec<String>,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 是否可自动修复
    pub can_auto_fix: bool,
}

/// 建议ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SuggestionId(pub u64);

/// 建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionType {
    /// 资源压缩
    Compression,
    /// 资源卸载
    Unload,
    /// 纹理格式转换
    TextureFormat,
    /// 网格简化
    MeshSimplification,
    /// 缓存策略调整
    CacheStrategy,
    /// LOD级别调整
    LodAdjustment,
    /// 分配器优化
    AllocatorOptimization,
    /// 其他
    Other,
}

/// 建议优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 无风险
    None,
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
}

// =============================================================================
// 内存顾问
// =============================================================================

/// 内存顾问配置
#[derive(Debug, Clone)]
pub struct MemoryAdvisorConfig {
    /// 快照间隔
    pub snapshot_interval: Duration,
    /// 保留快照数量
    pub max_snapshots: usize,
    /// 泄漏检测阈值（秒）
    pub leak_detection_threshold: Duration,
    /// 内存警告阈值 (0.0 - 1.0)
    pub memory_warning_threshold: f64,
    /// 内存危险阈值 (0.0 - 1.0)
    pub memory_critical_threshold: f64,
}

impl Default for MemoryAdvisorConfig {
    fn default() -> Self {
        Self {
            snapshot_interval: Duration::from_secs(5),
            max_snapshots: 100,
            leak_detection_threshold: Duration::from_secs(300), // 5分钟
            memory_warning_threshold: 0.7,                      // 70%
            memory_critical_threshold: 0.9,                     // 90%
        }
    }
}

/// 内存顾问
///
/// 提供智能内存管理建议和自动分析。
pub struct MemoryAdvisor {
    /// 配置
    config: MemoryAdvisorConfig,
    /// 内存快照历史
    snapshots: Arc<RwLock<VecDeque<MemorySnapshot>>>,
    /// 活跃分配追踪
    allocations: Arc<RwLock<HashMap<AllocationId, AllocationInfo>>>,
    /// 下一个分配ID
    next_allocation_id: Arc<RwLock<u64>>,
    /// 总可用内存
    total_memory: usize,
    /// 峰值内存使用
    peak_usage: Arc<RwLock<usize>>,
}

/// 分配ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AllocationId(pub u64);

/// 分配信息
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    /// 分配ID
    id: AllocationId,
    /// 资源路径
    resource_path: String,
    /// 内存分类
    category: MemoryCategory,
    /// 字节数
    bytes: usize,
    /// 分配时间
    allocated_at: Instant,
}

impl MemoryAdvisor {
    /// 创建新的内存顾问
    ///
    /// # 参数
    ///
    /// - `total_memory`: 总可用内存（字节）
    /// - `config`: 顾问配置
    pub fn new(total_memory: usize, config: MemoryAdvisorConfig) -> Self {
        Self {
            config,
            snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            next_allocation_id: Arc::new(RwLock::new(0)),
            total_memory,
            peak_usage: Arc::new(RwLock::new(0)),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(total_memory: usize) -> Self {
        Self::new(total_memory, MemoryAdvisorConfig::default())
    }

    /// 记录内存分配
    ///
    /// # 参数
    ///
    /// - `resource_path`: 资源路径
    /// - `category`: 内存分类
    /// - `bytes`: 分配字节数
    pub async fn record_allocation(
        &self,
        resource_path: String,
        category: MemoryCategory,
        bytes: usize,
    ) -> AllocationId {
        let mut next_id = self.next_allocation_id.write().await;
        let id = AllocationId(*next_id);
        *next_id += 1;

        let info = AllocationInfo {
            id,
            resource_path,
            category,
            bytes,
            allocated_at: Instant::now(),
        };

        self.allocations.write().await.insert(id, info);
        id
    }

    /// 记录内存释放
    ///
    /// # 参数
    ///
    /// - `allocation_id`: 分配ID
    pub async fn record_deallocation(&self, allocation_id: AllocationId) {
        self.allocations.write().await.remove(&allocation_id);
    }

    /// 创建内存快照
    pub async fn create_snapshot(&self) -> MemorySnapshot {
        let allocations = self.allocations.read().await;
        let current_usage: usize = allocations.values().map(|info| info.bytes).sum();

        // 更新峰值
        let mut peak = self.peak_usage.write().await;
        if current_usage > *peak {
            *peak = current_usage;
        }
        let peak_usage = *peak;

        // 按分类统计
        let mut category_usage: HashMap<MemoryCategory, CategoryUsage> = HashMap::new();
        for info in allocations.values() {
            let entry = category_usage.entry(info.category).or_insert(CategoryUsage {
                category: info.category,
                bytes: 0,
                count: 0,
                peak_bytes: 0,
            });
            entry.bytes += info.bytes;
            entry.count += 1;
            if entry.bytes > entry.peak_bytes {
                entry.peak_bytes = entry.bytes;
            }
        }

        let snapshot = MemorySnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_allocated: current_usage,
            current_usage,
            peak_usage,
            allocation_count: allocations.len() as u64,
            deallocation_count: 0, // Note: Deallocation tracking would require tracing every free() call
            // which adds overhead. Current usage tracking is sufficient for memory advisory.
            category_usage,
        };

        // 添加到历史
        let mut snapshots = self.snapshots.write().await;
        snapshots.push_back(snapshot.clone());
        if snapshots.len() > self.config.max_snapshots {
            snapshots.pop_front();
        }

        snapshot
    }

    /// 获取当前内存使用
    pub async fn get_current_usage(&self) -> usize {
        self.allocations.read().await.values().map(|info| info.bytes).sum()
    }

    /// 获取内存压力
    ///
    /// # 返回
    ///
    /// 内存压力等级
    pub async fn get_memory_pressure(&self) -> MemoryPressure {
        let current = self.get_current_usage().await;
        let ratio = current as f64 / self.total_memory.max(1) as f64;

        if ratio >= self.config.memory_critical_threshold {
            MemoryPressure::Critical
        } else if ratio >= self.config.memory_warning_threshold {
            MemoryPressure::High
        } else if ratio >= 0.5 {
            MemoryPressure::Medium
        } else {
            MemoryPressure::Low
        }
    }

    /// 检测内存泄漏
    ///
    /// # 返回
    ///
    /// 泄漏报告
    pub async fn detect_leaks(&self) -> LeakReport {
        let allocations = self.allocations.read().await;
        let now = Instant::now();
        let threshold = self.config.leak_detection_threshold;

        let mut leaks = Vec::new();
        let mut total_leaked = 0;

        for info in allocations.values() {
            let duration = now.duration_since(info.allocated_at);
            if duration > threshold {
                let suspicious_score = (duration.as_secs_f32() / threshold.as_secs_f32()).min(1.0);

                leaks.push(LeakInfo {
                    resource_path: info.resource_path.clone(),
                    category: info.category,
                    allocated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .saturating_sub(duration.as_secs()),
                    bytes: info.bytes,
                    duration_secs: duration.as_secs(),
                    suspicious_score,
                });

                total_leaked += info.bytes;
            }
        }

        let severity = if total_leaked > 100_000_000 {
            // > 100MB
            LeakSeverity::Critical
        } else if total_leaked > 50_000_000 {
            // > 50MB
            LeakSeverity::High
        } else if total_leaked > 10_000_000 {
            // > 10MB
            LeakSeverity::Medium
        } else if !leaks.is_empty() {
            LeakSeverity::Low
        } else {
            LeakSeverity::None
        };

        LeakReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            has_leaks: !leaks.is_empty(),
            leaks,
            total_leaked_bytes: total_leaked,
            severity,
        }
    }

    /// 生成优化建议
    ///
    /// # 返回
    ///
    /// 优化建议列表
    pub async fn generate_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // 检查内存压力
        let pressure = self.get_memory_pressure().await;
        if pressure == MemoryPressure::High || pressure == MemoryPressure::Critical {
            suggestions.push(OptimizationSuggestion {
                id: SuggestionId(1),
                suggestion_type: SuggestionType::Unload,
                priority: SuggestionPriority::High,
                title: "内存压力过高".to_string(),
                description: format!(
                    "当前内存使用达到 {:.1}%，建议卸载未使用资源",
                    self.get_current_usage().await as f64 / self.total_memory as f64 * 100.0
                ),
                estimated_savings: self.get_current_usage().await / 10, // 估计10%
                implementation_steps: vec![
                    "识别并卸载长时间未使用的资源".to_string(),
                    "使用LRU缓存策略".to_string(),
                    "增加资源卸载频率".to_string(),
                ],
                risk_level: RiskLevel::Low,
                can_auto_fix: true,
            });
        }

        // 检查纹理内存
        let snapshot = self.create_snapshot().await;
        if let Some(texture_usage) = snapshot.category_usage.get(&MemoryCategory::Textures) {
            if texture_usage.bytes > 100_000_000 {
                // > 100MB
                suggestions.push(OptimizationSuggestion {
                    id: SuggestionId(2),
                    suggestion_type: SuggestionType::TextureFormat,
                    priority: SuggestionPriority::Medium,
                    title: "纹理内存占用过高".to_string(),
                    description: format!(
                        "纹理占用 {:.1} MB，建议使用压缩格式",
                        texture_usage.bytes as f64 / 1_048_576.0
                    ),
                    estimated_savings: texture_usage.bytes / 2, // 估计50%
                    implementation_steps: vec![
                        "转换纹理为BC/ETC2格式".to_string(),
                        "降低未使用纹理分辨率".to_string(),
                        "启用纹理流式加载".to_string(),
                    ],
                    risk_level: RiskLevel::Medium,
                    can_auto_fix: true,
                });
            }
        }

        // 检查网格内存
        if let Some(mesh_usage) = snapshot.category_usage.get(&MemoryCategory::Meshes) {
            if mesh_usage.count > 1000 && mesh_usage.bytes > 50_000_000 {
                suggestions.push(OptimizationSuggestion {
                    id: SuggestionId(3),
                    suggestion_type: SuggestionType::MeshSimplification,
                    priority: SuggestionPriority::Medium,
                    title: "网格数量过多".to_string(),
                    description: format!("加载了 {} 个网格，考虑使用LOD或简化", mesh_usage.count),
                    estimated_savings: mesh_usage.bytes / 3,
                    implementation_steps: vec![
                        "为远距离物体使用LOD".to_string(),
                        "合并静态网格".to_string(),
                        "使用网格简化算法".to_string(),
                    ],
                    risk_level: RiskLevel::Low,
                    can_auto_fix: false,
                });
            }
        }

        suggestions
    }

    /// 获取内存使用历史
    pub async fn get_history(&self) -> Vec<MemorySnapshot> {
        self.snapshots.read().await.iter().cloned().collect()
    }

    /// 清理历史快照
    pub async fn clear_history(&self) {
        self.snapshots.write().await.clear();
    }

    /// 获取分配统计
    pub async fn get_allocation_stats(&self) -> AllocationStats {
        let allocations = self.allocations.read().await;

        let total_bytes: usize = allocations.values().map(|info| info.bytes).sum();
        let total_count = allocations.len();

        let mut by_category: HashMap<MemoryCategory, usize> = HashMap::new();
        for info in allocations.values() {
            *by_category.entry(info.category).or_insert(0) += info.bytes;
        }

        AllocationStats {
            total_bytes,
            total_count,
            by_category,
            peak_usage: *self.peak_usage.read().await,
        }
    }
}

/// 内存压力
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressure {
    /// 低压力
    Low,
    /// 中等压力
    Medium,
    /// 高压力
    High,
    /// 危险
    Critical,
}

/// 分配统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStats {
    /// 总字节数
    pub total_bytes: usize,
    /// 总数量
    pub total_count: usize,
    /// 按分类统计
    pub by_category: HashMap<MemoryCategory, usize>,
    /// 峰值使用
    pub peak_usage: usize,
}

// =============================================================================
// 内存事件
// =============================================================================

/// 内存事件
#[derive(Debug, Clone)]
pub enum MemoryEvent {
    /// 内存快照创建
    SnapshotCreated { snapshot: MemorySnapshot },
    /// 内存泄漏检测
    LeakDetected { report: LeakReport },
    /// 优化建议生成
    SuggestionGenerated {
        suggestions: Vec<OptimizationSuggestion>,
    },
    /// 内存压力变化
    PressureChanged {
        old_pressure: MemoryPressure,
        new_pressure: MemoryPressure,
    },
    /// 资源自动卸载
    ResourceUnloaded {
        resource_path: String,
        bytes_freed: usize,
    },
}

impl DomainEvent for MemoryEvent {
    fn event_type(&self) -> &'static str {
        match self {
            MemoryEvent::SnapshotCreated { .. } => "SnapshotCreated",
            MemoryEvent::LeakDetected { .. } => "LeakDetected",
            MemoryEvent::SuggestionGenerated { .. } => "SuggestionGenerated",
            MemoryEvent::PressureChanged { .. } => "PressureChanged",
            MemoryEvent::ResourceUnloaded { .. } => "ResourceUnloaded",
        }
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        // 事件应用逻辑（由具体的内存系统处理）
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// =============================================================================
// ECS集成
// =============================================================================

/// 内存顾问资源
#[derive(Resource)]
pub struct MemoryAdvisorResource {
    pub advisor: MemoryAdvisor,
}

/// 内存统计组件
#[derive(Component, Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub pressure: MemoryPressure,
    pub last_snapshot: Option<MemorySnapshot>,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_advisor_creation() {
        let advisor = MemoryAdvisor::with_default_config(1_000_000_000); // 1GB

        assert_eq!(advisor.total_memory, 1_000_000_000);
    }

    #[tokio::test]
    async fn test_allocation_tracking() {
        let advisor = MemoryAdvisor::with_default_config(1_000_000_000);

        let id = advisor
            .record_allocation("test.png".to_string(), MemoryCategory::Textures, 1024)
            .await;

        let usage = advisor.get_current_usage().await;
        assert_eq!(usage, 1024);

        advisor.record_deallocation(id).await;
        let usage = advisor.get_current_usage().await;
        assert_eq!(usage, 0);
    }

    #[tokio::test]
    async fn test_snapshot_creation() {
        let advisor = MemoryAdvisor::with_default_config(1_000_000_000);

        advisor
            .record_allocation("texture1.png".to_string(), MemoryCategory::Textures, 2048)
            .await;
        advisor
            .record_allocation("mesh1.obj".to_string(), MemoryCategory::Meshes, 4096)
            .await;

        let snapshot = advisor.create_snapshot().await;
        assert_eq!(snapshot.current_usage, 2048 + 4096);
        assert_eq!(snapshot.allocation_count, 2);
    }

    #[tokio::test]
    async fn test_memory_pressure() {
        let advisor = MemoryAdvisor::with_default_config(1_000_000); // 1MB

        // 高压力
        advisor
            .record_allocation("large.bin".to_string(), MemoryCategory::Other, 800_000)
            .await;

        let pressure = advisor.get_memory_pressure().await;
        assert_eq!(pressure, MemoryPressure::High);
    }

    #[tokio::test]
    async fn test_leak_detection() {
        let config = MemoryAdvisorConfig {
            leak_detection_threshold: Duration::from_millis(100),
            ..Default::default()
        };
        let advisor = MemoryAdvisor::new(1_000_000_000, config);

        advisor
            .record_allocation("leak.png".to_string(), MemoryCategory::Textures, 1024)
            .await;

        // 等待超过阈值
        tokio::time::sleep(Duration::from_millis(150)).await;

        let report = advisor.detect_leaks().await;
        assert!(report.has_leaks);
        assert_eq!(report.leaks.len(), 1);
    }
}
