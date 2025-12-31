//! # LLM成本追踪系统
//!
//! 提供LLM API调用的成本追踪、预算控制和报告功能。
//!
//! ## 功能特性
//!
//! - **实时追踪** - 追踪每次API调用的成本
//! - **预算控制** - 设置每日/每月预算限制
//! - **成本预警** - 接近预算时发出警告
//! - **详细报告** - 按模型、NPC、时间段生成报告
//! - **导出功能** - 导出成本数据为CSV/JSON
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::cost_tracking::{CostTracker, BudgetConfig};
//!
//! let tracker = CostTracker::new(BudgetConfig {
//!     daily_budget_usd: 10.0,
//!     monthly_budget_usd: 100.0,
//!     ..Default::default()
//! });
//!
//! // 记录API调用
//! tracker.record_call("gpt-4", 1000, 500, "npc_merchant");
//!
//! // 检查预算
//! if tracker.is_over_budget() {
//!     log::warn!("LLM budget exceeded!");
//! }
//! ```

use super::llm_cache::CostEstimator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 预算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// 每日预算（美元）
    pub daily_budget_usd: f64,
    /// 每月预算（美元）
    pub monthly_budget_usd: f64,
    /// 警告阈值（预算的百分比，0.0-1.0）
    pub warning_threshold: f32,
    /// 是否在超出预算时停止调用
    pub block_on_exceed: bool,
    /// 是否启用预算控制
    pub enable_budget_control: bool,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            daily_budget_usd: 10.0,
            monthly_budget_usd: 100.0,
            warning_threshold: 0.8, // 80%时警告
            block_on_exceed: false,
            enable_budget_control: true,
        }
    }
}

/// API调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APICallRecord {
    /// 时间戳
    pub timestamp: u64,
    /// 模型名称
    pub model: String,
    /// NPC ID
    pub npc_id: String,
    /// 输入token数
    pub input_tokens: usize,
    /// 输出token数
    pub output_tokens: usize,
    /// 总token数
    pub total_tokens: usize,
    /// 成本（美元）
    pub cost_usd: f64,
    /// 调用时长（毫秒）
    pub duration_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

impl APICallRecord {
    /// 创建新的调用记录
    pub fn new(
        model: &str,
        npc_id: &str,
        input_tokens: usize,
        output_tokens: usize,
        cost_usd: f64,
        duration_ms: u64,
        success: bool,
    ) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            model: model.to_string(),
            npc_id: npc_id.to_string(),
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd,
            duration_ms,
            success,
            error: None,
        }
    }
}

/// 成本统计（按时间段）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostStatistics {
    /// 总调用次数
    pub total_calls: u64,
    /// 成功调用次数
    pub successful_calls: u64,
    /// 失败调用次数
    pub failed_calls: u64,
    /// 总token数
    pub total_tokens: u64,
    /// 总成本（美元）
    pub total_cost_usd: f64,
    /// 平均每次调用成本
    pub average_cost_per_call: f64,
    /// 平均每次调用token数
    pub average_tokens_per_call: f64,
    /// 平均响应时间（毫秒）
    pub average_latency_ms: f64,
}

/// 成本统计（按模型）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCostStats {
    /// 模型名称
    pub model: String,
    /// 调用次数
    pub call_count: u64,
    /// 总token数
    pub total_tokens: u64,
    /// 总成本
    pub total_cost: f64,
}

/// 成本统计（按NPC）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NPCCostStats {
    /// NPC ID
    pub npc_id: String,
    /// 调用次数
    pub call_count: u64,
    /// 总token数
    pub total_tokens: u64,
    /// 总成本
    pub total_cost: f64,
}

/// 预算状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetStatus {
    /// 在预算内
    WithinBudget,
    /// 接近预算（警告阈值以上）
    NearBudget,
    /// 超出预算
    OverBudget,
}

/// 成本报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReport {
    /// 报告生成时间
    pub generated_at: u64,
    /// 报告时间段（秒）
    pub period_seconds: u64,
    /// 总体统计
    pub overall_stats: CostStatistics,
    /// 按模型统计
    pub by_model: Vec<ModelCostStats>,
    /// 按NPC统计
    pub by_npc: Vec<NPCCostStats>,
    /// 当前预算状态
    pub budget_status: BudgetStatus,
    /// 预算使用百分比
    pub budget_usage_percent: f32,
}

/// 成本追踪器
pub struct CostTracker {
    config: BudgetConfig,
    cost_estimator: CostEstimator,
    records: Arc<RwLock<Vec<APICallRecord>>>,
    /// 缓存的统计数据
    stats_cache: Arc<RwLock<HashMap<String, CostStatistics>>>,
}

impl CostTracker {
    /// 创建新的成本追踪器
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            cost_estimator: CostEstimator::new(),
            records: Arc::new(RwLock::new(Vec::new())),
            stats_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录API调用
    pub fn record_call(
        &self,
        model: &str,
        input_tokens: usize,
        output_tokens: usize,
        npc_id: &str,
    ) -> Result<(), String> {
        let cost = self.cost_estimator.estimate_cost(model, input_tokens, output_tokens);

        // 检查预算
        if self.config.enable_budget_control {
            if self.config.block_on_exceed && self.is_over_budget() {
                return Err("Budget exceeded, API call blocked".to_string());
            }

            let status = self.get_budget_status();
            if status == BudgetStatus::NearBudget {
                log::warn!(
                    "Approaching LLM budget limit: {:.1}% used",
                    self.get_budget_usage_percent()
                );
            }
        }

        let record = APICallRecord::new(model, npc_id, input_tokens, output_tokens, cost, 0, true);

        let mut records = self.records.write().unwrap();
        records.push(record);

        // 清除缓存
        let mut cache = self.stats_cache.write().unwrap();
        cache.clear();

        Ok(())
    }

    /// 记录失败的API调用
    pub fn record_failed_call(&self, model: &str, npc_id: &str, error: &str) {
        let record = APICallRecord {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            model: model.to_string(),
            npc_id: npc_id.to_string(),
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cost_usd: 0.0,
            duration_ms: 0,
            success: false,
            error: Some(error.to_string()),
        };

        let mut records = self.records.write().unwrap();
        records.push(record);
    }

    /// 记录API调用（带时长）
    pub fn record_call_with_duration(
        &self,
        model: &str,
        input_tokens: usize,
        output_tokens: usize,
        npc_id: &str,
        duration_ms: u64,
    ) -> Result<(), String> {
        let cost = self.cost_estimator.estimate_cost(model, input_tokens, output_tokens);

        if self.config.enable_budget_control {
            if self.config.block_on_exceed && self.is_over_budget() {
                return Err("Budget exceeded, API call blocked".to_string());
            }
        }

        let record = APICallRecord::new(
            model,
            npc_id,
            input_tokens,
            output_tokens,
            cost,
            duration_ms,
            true,
        );

        let mut records = self.records.write().unwrap();
        records.push(record);

        Ok(())
    }

    /// 获取指定时间段内的统计
    pub fn get_statistics(&self, period_seconds: u64) -> CostStatistics {
        let cache_key = format!("stats_{}", period_seconds);
        if let Some(cached) = self.stats_cache.read().unwrap().get(&cache_key) {
            return cached.clone();
        }

        let records = self.records.read().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let filtered: Vec<_> = records
            .iter()
            .filter(|r| r.success && (now - r.timestamp) <= period_seconds)
            .collect();

        let total_calls = filtered.len() as u64;
        let successful_calls = filtered.iter().filter(|r| r.success).count() as u64;
        let total_tokens: u64 = filtered.iter().map(|r| r.total_tokens as u64).sum();
        let total_cost: f64 = filtered.iter().map(|r| r.cost_usd).sum();
        let total_duration: u64 = filtered.iter().map(|r| r.duration_ms).sum();

        let stats = CostStatistics {
            total_calls,
            successful_calls,
            failed_calls: 0,
            total_tokens,
            total_cost_usd: total_cost,
            average_cost_per_call: if total_calls > 0 {
                total_cost / total_calls as f64
            } else {
                0.0
            },
            average_tokens_per_call: if total_calls > 0 {
                total_tokens as f64 / total_calls as f64
            } else {
                0.0
            },
            average_latency_ms: if total_calls > 0 {
                total_duration as f64 / total_calls as f64
            } else {
                0.0
            },
        };

        // 缓存结果
        let mut cache = self.stats_cache.write().unwrap();
        cache.insert(cache_key, stats.clone());

        stats
    }

    /// 获取按模型分组的统计
    pub fn get_statistics_by_model(&self, period_seconds: u64) -> Vec<ModelCostStats> {
        let records = self.records.read().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut model_stats: HashMap<String, ModelCostStats> = HashMap::new();

        for record in records.iter() {
            if !record.success || (now - record.timestamp) > period_seconds {
                continue;
            }

            let entry = model_stats.entry(record.model.clone()).or_insert_with(|| ModelCostStats {
                model: record.model.clone(),
                ..Default::default()
            });

            entry.call_count += 1;
            entry.total_tokens += record.total_tokens as u64;
            entry.total_cost += record.cost_usd;
        }

        model_stats.into_values().collect()
    }

    /// 获取按NPC分组的统计
    pub fn get_statistics_by_npc(&self, period_seconds: u64) -> Vec<NPCCostStats> {
        let records = self.records.read().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut npc_stats: HashMap<String, NPCCostStats> = HashMap::new();

        for record in records.iter() {
            if !record.success || (now - record.timestamp) > period_seconds {
                continue;
            }

            let entry = npc_stats.entry(record.npc_id.clone()).or_insert_with(|| NPCCostStats {
                npc_id: record.npc_id.clone(),
                ..Default::default()
            });

            entry.call_count += 1;
            entry.total_tokens += record.total_tokens as u64;
            entry.total_cost += record.cost_usd;
        }

        npc_stats.into_values().collect()
    }

    /// 生成成本报告
    pub fn generate_report(&self, period_seconds: u64) -> CostReport {
        CostReport {
            generated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            period_seconds,
            overall_stats: self.get_statistics(period_seconds),
            by_model: self.get_statistics_by_model(period_seconds),
            by_npc: self.get_statistics_by_npc(period_seconds),
            budget_status: self.get_budget_status(),
            budget_usage_percent: self.get_budget_usage_percent(),
        }
    }

    /// 获取当前预算状态
    pub fn get_budget_status(&self) -> BudgetStatus {
        if !self.config.enable_budget_control {
            return BudgetStatus::WithinBudget;
        }

        let usage_percent = self.get_budget_usage_percent();

        if usage_percent >= 100.0 {
            BudgetStatus::OverBudget
        } else if usage_percent >= (self.config.warning_threshold * 100.0) {
            BudgetStatus::NearBudget
        } else {
            BudgetStatus::WithinBudget
        }
    }

    /// 获取预算使用百分比
    pub fn get_budget_usage_percent(&self) -> f32 {
        let daily_cost = self.get_statistics(86400); // 24小时
        (daily_cost.total_cost_usd / self.config.daily_budget_usd * 100.0) as f32
    }

    /// 检查是否超出预算
    pub fn is_over_budget(&self) -> bool {
        if !self.config.enable_budget_control {
            return false;
        }
        self.get_budget_usage_percent() >= 100.0
    }

    /// 获取配置
    pub fn get_config(&self) -> &BudgetConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: BudgetConfig) {
        self.config = config;
    }

    /// 清空记录
    pub fn clear_records(&self) {
        let mut records = self.records.write().unwrap();
        records.clear();
    }

    /// 导出记录为CSV
    pub fn export_to_csv(&self, path: &str) -> Result<(), String> {
        let records = self.records.read().unwrap();

        let mut csv = String::from(
            "timestamp,model,npc_id,input_tokens,output_tokens,total_tokens,cost_usd,duration_ms,success,error\n",
        );

        for record in records.iter() {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                record.timestamp,
                record.model,
                record.npc_id,
                record.input_tokens,
                record.output_tokens,
                record.total_tokens,
                record.cost_usd,
                record.duration_ms,
                record.success,
                record.error.as_deref().unwrap_or("")
            ));
        }

        std::fs::write(path, csv).map_err(|e| format!("Failed to write CSV: {}", e))
    }

    /// 导出记录为JSON
    pub fn export_to_json(&self, path: &str) -> Result<(), String> {
        let records = self.records.read().unwrap();
        let json =
            serde_json::to_string_pretty(&*records).map_err(|e| format!("JSON error: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("Failed to write JSON: {}", e))
    }

    /// 从文件导入记录
    pub fn import_from_json(&self, path: &str) -> Result<(), String> {
        if !Path::new(path).exists() {
            return Err("File does not exist".to_string());
        }

        let data =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let imported: Vec<APICallRecord> =
            serde_json::from_str(&data).map_err(|e| format!("JSON error: {}", e))?;

        let mut records = self.records.write().unwrap();
        records.extend(imported);

        Ok(())
    }

    /// 获取记录数量
    pub fn record_count(&self) -> usize {
        self.records.read().unwrap().len()
    }
}

impl Clone for CostTracker {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cost_estimator: self.cost_estimator.clone(),
            records: Arc::clone(&self.records),
            stats_cache: Arc::clone(&self.stats_cache),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_config_default() {
        let config = BudgetConfig::default();
        assert_eq!(config.daily_budget_usd, 10.0);
        assert_eq!(config.monthly_budget_usd, 100.0);
        assert_eq!(config.warning_threshold, 0.8);
    }

    #[test]
    fn test_cost_tracker_record_call() {
        let tracker = CostTracker::new(BudgetConfig::default());
        let result = tracker.record_call("gpt-3.5-turbo", 100, 50, "npc1");
        assert!(result.is_ok());
        assert_eq!(tracker.record_count(), 1);
    }

    #[test]
    fn test_cost_statistics() {
        let tracker = CostTracker::new(BudgetConfig::default());
        tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc1").unwrap();
        tracker.record_call("gpt-3.5-turbo", 500, 250, "npc1").unwrap();

        let stats = tracker.get_statistics(86400);
        assert_eq!(stats.total_calls, 2);
        assert_eq!(stats.total_tokens, 2250);
    }

    #[test]
    fn test_budget_status() {
        let tracker = CostTracker::new(BudgetConfig {
            daily_budget_usd: 0.001, // 非常低的预算
            ..Default::default()
        });

        tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc1").unwrap();

        // 应该超出预算
        assert_eq!(tracker.get_budget_status(), BudgetStatus::OverBudget);
    }

    #[test]
    fn test_statistics_by_model() {
        let tracker = CostTracker::new(BudgetConfig::default());
        tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc1").unwrap();
        tracker.record_call("gpt-4", 500, 250, "npc2").unwrap();

        let stats = tracker.get_statistics_by_model(86400);
        assert_eq!(stats.len(), 2);
    }
}
