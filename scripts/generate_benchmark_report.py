#!/usr/bin/env python3
"""
生成benchmark性能报告
解析Criterion JSON输出并生成可读的性能报告
"""

import json
import os
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any

def parse_criterion_output(benchmark_dir: Path) -> Dict[str, Dict[str, float]]:
    """
    解析Criterion JSON输出

    Args:
        benchmark_dir: 包含benchmark JSON文件的目录

    Returns:
        字典映射benchmark名称到性能指标
    """
    results = {}

    for benchmark_file in benchmark_dir.rglob("benchmark.json"):
        try:
            with open(benchmark_file, 'r') as f:
                data = json.load(f)
                name = data.get("group_name", "unknown")

                # 提取关键指标
                results[name] = {
                    "mean": data.get("mean", {}).get("estimate", 0) / 1e9,  # 转换为ns
                    "stddev": data.get("mean", {}).get("stddev", 0) / 1e9,
                    "median": data.get("median", {}).get("estimate", 0) / 1e9,
                    "min": data.get("min", {}).get("estimate", 0) / 1e9,
                    "max": data.get("max", {}).get("estimate", 0) / 1e9,
                    "throughput": data.get("throughput", {}).get("value", 0),
                }
        except (json.JSONDecodeError, KeyError, IOError) as e:
            print(f"Warning: Failed to parse {benchmark_file}: {e}", file=sys.stderr)
            continue

    return results

def compare_with_baseline(current: Dict[str, Dict[str, float]],
                         baseline: Dict[str, Dict[str, float]]) -> Dict[str, Dict[str, Any]]:
    """
    与baseline对比性能

    Args:
        current: 当前运行的benchmark结果
        baseline: baseline运行的benchmark结果

    Returns:
        对比结果字典
    """
    comparison = {}

    for name, current_metrics in current.items():
        if name in baseline:
            baseline_metrics = baseline[name]
            mean_change = current_metrics["mean"] - baseline_metrics["mean"]

            if baseline_metrics["mean"] > 0:
                pct_change = (mean_change / baseline_metrics["mean"]) * 100
            else:
                pct_change = 0

            # 判断性能状态
            if pct_change < -5:
                status = "🟢 Improved"
                status_emoji = "✅"
            elif pct_change > 5:
                status = "🔴 Regressed"
                status_emoji = "❌"
            else:
                status = "🟡 Stable"
                status_emoji = "➖"

            comparison[name] = {
                "current": current_metrics["mean"],
                "baseline": baseline_metrics["mean"],
                "change": mean_change,
                "pct_change": pct_change,
                "status": status,
                "status_emoji": status_emoji,
                "current_stddev": current_metrics["stddev"],
                "baseline_stddev": baseline_metrics["stddev"]
            }

    return comparison

def generate_markdown_report(comparison: Dict[str, Dict[str, Any]],
                            timestamp: str) -> str:
    """
    生成Markdown格式的性能报告

    Args:
        comparison: 对比结果字典
        timestamp: 报告时间戳

    Returns:
        Markdown格式的报告字符串
    """
    report = []
    report.append("# 🚀 Benchmark Performance Report\n")
    report.append(f"**Generated:** {timestamp}\n\n")

    # 统计摘要
    report.append("## 📊 Summary\n\n")

    improved = sum(1 for v in comparison.values() if v["pct_change"] < -5)
    regressed = sum(1 for v in comparison.values() if v["pct_change"] > 5)
    stable = len(comparison) - improved - regressed

    report.append(f"- ✅ **Improved:** {improved} benchmarks")
    report.append(f"- ❌ **Regressed:** {regressed} benchmarks")
    report.append(f"- ➖ **Stable:** {stable} benchmarks")
    report.append(f"- 📈 **Total:** {len(comparison)} benchmarks\n\n")

    # 关键指标
    if improved > 0:
        most_improved = min(comparison.items(), key=lambda x: x[1]["pct_change"])
        report.append(f"**Most Improved:** {most_improved[0]} ({most_improved[1]['pct_change']:.1f}%)\n")

    if regressed > 0:
        most_regressed = max(comparison.items(), key=lambda x: x[1]["pct_change"])
        report.append(f"**Most Regressed:** {most_regressed[0]} (+{most_regressed[1]['pct_change']:.1f}%)\n")

    report.append("\n---\n\n")

    # 详细结果表格
    report.append("## 📈 Detailed Results\n\n")
    report.append("| Benchmark | Status | Current (ns) | Baseline (ns) | Change | % Change |\n")
    report.append("|-----------|--------|--------------|---------------|--------|----------|\n")

    # 按性能变化排序
    sorted_results = sorted(comparison.items(), key=lambda x: x[1]["pct_change"])

    for name, metrics in sorted_results:
        status_emoji = metrics["status_emoji"]
        current_val = metrics["current"]
        baseline_val = metrics["baseline"]
        change = metrics["change"]
        pct_change = metrics["pct_change"]

        report.append(f"| {name} | {status_emoji} | {current_val:.2f} | {baseline_val:.2f} | "
                     f"{change:+.2f} | {pct_change:+.1f}% |\n")

    report.append("\n---\n\n")

    # 性能回归详情
    if regressed > 0:
        report.append("## ⚠️ Performance Regressions\n\n")
        report.append("The following benchmarks show significant performance regressions (>5%):\n\n")

        for name, metrics in sorted_results:
            if metrics["pct_change"] > 5:
                report.append(f"### ❌ {name}\n")
                report.append(f"- **Current:** {metrics['current']:.2f} ns ± {metrics['current_stddev']:.2f}\n")
                report.append(f"- **Baseline:** {metrics['baseline']:.2f} ns ± {metrics['baseline_stddev']:.2f}\n")
                report.append(f"- **Change:** +{metrics['pct_change']:.1f}%\n\n")

    # 性能改进详情
    if improved > 0:
        report.append("## ✅ Performance Improvements\n\n")
        report.append("The following benchmarks show significant performance improvements (>5%):\n\n")

        for name, metrics in sorted_results:
            if metrics["pct_change"] < -5:
                report.append(f"### ✅ {name}\n")
                report.append(f"- **Current:** {metrics['current']:.2f} ns ± {metrics['current_stddev']:.2f}\n")
                report.append(f"- **Baseline:** {metrics['baseline']:.2f} ns ± {metrics['baseline_stddev']:.2f}\n")
                report.append(f"- **Change:** {metrics['pct_change']:.1f}%\n\n")

    return "\n".join(report)

def save_report(report: str, output_path: Path) -> None:
    """
    保存报告到文件

    Args:
        report: 报告内容
        output_path: 输出文件路径
    """
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        f.write(report)

    print(f"Report saved to: {output_path}")

def main():
    """主函数"""
    # 设置路径
    root_dir = Path(__file__).parent.parent
    benchmark_dir = root_dir / "target" / "criterion"
    baseline_dir = benchmark_dir / "main"

    # 如果target目录下没有结果，尝试benches目录
    if not benchmark_dir.exists():
        benchmark_dir = root_dir / "game_engine" / "benches" / "results"
        baseline_dir = root_dir / "game_engine" / "benches" / "baseline" / "main"

    print(f"Looking for benchmark results in: {benchmark_dir}")
    print(f"Looking for baseline in: {baseline_dir}")

    # 解析当前结果
    current = parse_criterion_output(benchmark_dir)

    if not current:
        print("Error: No benchmark results found. Make sure to run benchmarks first.", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(current)} benchmark results")

    # 解析baseline
    baseline = {}
    if baseline_dir.exists():
        baseline = parse_criterion_output(baseline_dir)
        print(f"Found {len(baseline)} baseline results")
    else:
        print("Warning: No baseline found. Creating comparison with zero baseline.", file=sys.stderr)

    # 对比性能
    comparison = compare_with_baseline(current, baseline)

    if not comparison:
        print("Warning: No overlapping benchmarks found between current and baseline", file=sys.stderr)

    # 生成报告
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC")
    report = generate_markdown_report(comparison, timestamp)

    # 输出到stdout
    print("\n" + "="*80)
    print(report)
    print("="*80 + "\n")

    # 保存报告
    output_path = root_dir / "benchmark_report.md"
    save_report(report, output_path)

    # 如果有回归，返回非零退出码
    regressed_count = sum(1 for v in comparison.values() if v["pct_change"] > 5)
    if regressed_count > 0:
        print(f"\n⚠️ Warning: {regressed_count} benchmark(s) show performance regression")
        sys.exit(1)
    else:
        print("\n✅ No performance regressions detected")
        sys.exit(0)

if __name__ == "__main__":
    main()
