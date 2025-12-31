//! 成本追踪系统使用示例
//!
//! 演示如何使用LLM成本追踪和预算控制

use game_engine::ai::cost_tracking::{BudgetConfig, BudgetStatus, CostTracker};
use game_engine::ai::llm_cache::{CacheConfig, CacheKey, LLMCache};
use std::thread;
use std::time::Duration;

fn main() {
    // 创建成本追踪器
    let cost_tracker = CostTracker::new(BudgetConfig {
        daily_budget_usd: 5.0,
        monthly_budget_usd: 50.0,
        warning_threshold: 0.8,
        block_on_exceed: false,
        enable_budget_control: true,
    });

    println!("=== LLM Cost Tracking System ===\n");

    // 记录一些API调用
    println!("Recording API calls...");
    cost_tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc_merchant").unwrap();
    cost_tracker.record_call("gpt-4", 500, 300, "npc_elder").unwrap();
    cost_tracker.record_call("gpt-3.5-turbo", 800, 400, "npc_guard").unwrap();

    thread::sleep(Duration::from_millis(100));

    // 获取统计信息
    println!("\n=== Statistics (24h) ===");
    let stats = cost_tracker.get_statistics(86400);
    println!("Total calls: {}", stats.total_calls);
    println!("Total tokens: {}", stats.total_tokens);
    println!("Total cost: ${:.4}", stats.total_cost_usd);
    println!("Average cost per call: ${:.4}", stats.average_cost_per_call);
    println!("Average tokens per call: {:.1}", stats.average_tokens_per_call);

    // 按模型统计
    println!("\n=== Cost by Model ===");
    let model_stats = cost_tracker.get_statistics_by_model(86400);
    for model_stat in model_stats {
        println!(
            "{}: {} calls, {} tokens, ${:.4}",
            model_stat.model, model_stat.call_count, model_stat.total_tokens, model_stat.total_cost
        );
    }

    // 按NPC统计
    println!("\n=== Cost by NPC ===");
    let npc_stats = cost_tracker.get_statistics_by_npc(86400);
    for npc_stat in npc_stats {
        println!(
            "{}: {} calls, {} tokens, ${:.4}",
            npc_stat.npc_id, npc_stat.call_count, npc_stat.total_tokens, npc_stat.total_cost
        );
    }

    // 预算状态
    println!("\n=== Budget Status ===");
    let budget_status = cost_tracker.get_budget_status();
    let usage_percent = cost_tracker.get_budget_usage_percent();
    println!("Status: {:?}", budget_status);
    println!("Budget usage: {:.1}%", usage_percent);

    if budget_status == BudgetStatus::NearBudget {
        println!("⚠️  Warning: Approaching budget limit!");
    } else if budget_status == BudgetStatus::OverBudget {
        println!("❌ Error: Budget exceeded!");
    } else {
        println!("✓ Budget OK");
    }

    // 生成报告
    println!("\n=== Cost Report ===");
    let report = cost_tracker.generate_report(86400);
    println!("Period: {} seconds", report.period_seconds);
    println!("Overall stats:");
    println!("  Calls: {}", report.overall_stats.total_calls);
    println!("  Cost: ${:.4}", report.overall_stats.total_cost_usd);

    // 导出为JSON
    println!("\n=== Exporting Data ===");
    if let Err(e) = cost_tracker.export_to_json("cost_report.json") {
        println!("Failed to export: {}", e);
    } else {
        println!("✓ Exported to cost_report.json");
    }

    // 导出为CSV
    if let Err(e) = cost_tracker.export_to_csv("cost_report.csv") {
        println!("Failed to export: {}", e);
    } else {
        println!("✓ Exported to cost_report.csv");
    }
}

/// 演示缓存系统
fn demonstrate_cache_system() {
    println!("\n=== LLM Cache System ===\n");

    let cache = LLMCache::new(CacheConfig::default());

    // 创建缓存键
    let key1 = CacheKey::from_prompt("npc1", "Hello, traveler!", "gpt-3.5-turbo");
    let key2 = CacheKey::from_context("npc1", "Hello, traveler!", "friendly_merchant", "gpt-3.5-turbo");

    // 首次获取（未命中）
    println!("First lookup (miss): {:?}", cache.get(&key1));

    // 添加到缓存
    cache.put(key1.clone(), "Greetings and welcome to my shop!".to_string(), 50);

    // 再次获取（命中）
    println!("Second lookup (hit): {:?}", cache.get(&key1));

    // 获取统计
    let stats = cache.get_stats();
    println!("\nCache statistics:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.1}%", stats.hit_rate() * 100.0);
    println!("  Saved tokens: {}", stats.saved_tokens);
    println!("  Current entries: {}", stats.current_entries);

    // 检查预算警告
    println!("\n=== Budget Warnings ===");
    let budget_tracker = CostTracker::new(BudgetConfig {
        daily_budget_usd: 0.01, // 非常低的预算用于演示
        warning_threshold: 0.5,
        ..Default::default()
    });

    // 记录一个会超出预算的调用
    let result = budget_tracker.record_call("gpt-4", 10000, 5000, "npc1");
    println!("API call result: {:?}", result);

    if let BudgetStatus::OverBudget = budget_tracker.get_budget_status() {
        println!("⚠️  Budget exceeded! Consider disabling LLM or using cheaper models.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_creation() {
        let tracker = CostTracker::new(BudgetConfig::default());
        assert_eq!(tracker.record_count(), 0);
    }

    #[test]
    fn test_record_call() {
        let tracker = CostTracker::new(BudgetConfig::default());
        let result = tracker.record_call("gpt-3.5-turbo", 100, 50, "npc1");
        assert!(result.is_ok());
        assert_eq!(tracker.record_count(), 1);
    }

    #[test]
    fn test_statistics() {
        let tracker = CostTracker::new(BudgetConfig::default());
        tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc1").unwrap();
        tracker.record_call("gpt-3.5-turbo", 500, 250, "npc1").unwrap();

        let stats = tracker.get_statistics(86400);
        assert_eq!(stats.total_calls, 2);
        assert_eq!(stats.total_tokens, 2250);
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = LLMCache::new(CacheConfig::default());
        let key = CacheKey::from_prompt("npc1", "test", "gpt-3.5-turbo");

        assert!(cache.get(&key).is_none());
        cache.put(key.clone(), "response".to_string(), 100);
        assert_eq!(cache.get(&key), Some("response".to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let cache = LLMCache::new(CacheConfig::default());
        let key = CacheKey::from_prompt("npc1", "test", "gpt-3.5-turbo");

        cache.put(key.clone(), "response".to_string(), 100);
        cache.get(&key);

        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.saved_tokens, 100);
    }

    #[test]
    fn test_budget_status() {
        let tracker = CostTracker::new(BudgetConfig {
            daily_budget_usd: 0.001,
            warning_threshold: 0.5,
            ..Default::default()
        });

        tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc1").unwrap();

        assert_eq!(tracker.get_budget_status(), BudgetStatus::OverBudget);
    }
}
