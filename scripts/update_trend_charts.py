#!/usr/bin/env python3
"""
生成性能趋势图表
加载历史benchmark数据并生成性能趋势可视化
"""

import json
import os
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List
import argparse

# 检查是否安装了matplotlib
try:
    import matplotlib
    matplotlib.use('Agg')  # 使用非交互式后端
    import matplotlib.pyplot as plt
    import matplotlib.dates as mdates
    MATPLOTLIB_AVAILABLE = True
except ImportError:
    MATPLOTLIB_AVAILABLE = False
    print("Warning: matplotlib not available. Charts will not be generated.", file=sys.stderr)
    print("Install with: pip install matplotlib", file=sys.stderr)


def load_historical_data(trends_dir: Path) -> Dict[datetime, Dict[str, Dict]]:
    """
    加载历史benchmark数据

    Args:
        trends_dir: 趋势数据目录

    Returns:
        按日期索引的历史数据字典
    """
    history = {}

    if not trends_dir.exists():
        trends_dir.mkdir(parents=True, exist_ok=True)
        print(f"Created trends directory: {trends_dir}")
        return history

    for json_file in trends_dir.glob("*.json"):
        # 跳过非数据文件
        if json_file.name.startswith("benchmark_"):
            continue

        try:
            date_str = json_file.stem
            # 尝试解析日期格式
            try:
                date = datetime.strptime(date_str, "%Y-%m-%d")
            except ValueError:
                # 尝试其他日期格式
                try:
                    date = datetime.strptime(date_str, "%Y%m%d")
                except ValueError:
                    print(f"Warning: Skipping {json_file} - invalid date format", file=sys.stderr)
                    continue

            with open(json_file, 'r') as f:
                data = json.load(f)
                history[date] = data

        except (json.JSONDecodeError, IOError) as e:
            print(f"Warning: Failed to load {json_file}: {e}", file=sys.stderr)
            continue

    return history


def save_current_results(results: Dict, trends_dir: Path, date: datetime = None) -> Path:
    """
    保存当前benchmark结果到历史数据

    Args:
        results: 当前benchmark结果
        trends_dir: 趋势数据目录
        date: 日期对象（默认为今天）

    Returns:
        保存的文件路径
    """
    if date is None:
        date = datetime.now()

    date_str = date.strftime("%Y-%m-%d")
    output_path = trends_dir / f"{date_str}.json"

    with open(output_path, 'w') as f:
        json.dump(results, f, indent=2)

    print(f"Saved benchmark data to: {output_path}")
    return output_path


def generate_trend_chart(history: Dict[datetime, Dict], benchmark_name: str,
                        output_dir: Path) -> bool:
    """
    为特定benchmark生成趋势图

    Args:
        history: 历史数据字典
        benchmark_name: benchmark名称
        output_dir: 输出目录

    Returns:
        是否成功生成图表
    """
    if not MATPLOTLIB_AVAILABLE:
        return False

    # 收集数据点
    dates = sorted(history.keys())
    values = []
    stddevs = []

    for date in dates:
        data = history[date]
        if benchmark_name in data:
            benchmark_data = data[benchmark_name]
            values.append(benchmark_data.get("mean", 0))
            stddevs.append(benchmark_data.get("stddev", 0))

    if len(values) < 2:
        print(f"Warning: Not enough data points for {benchmark_name} (need at least 2)")
        return False

    # 创建图表
    plt.figure(figsize=(12, 6))

    # 绘制主曲线
    plt.plot(dates, values, marker='o', linewidth=2, markersize=6,
             label='Mean Time', color='#2196F3')

    # 添加标准差区域
    if stddevs:
        upper = [v + s for v, s in zip(values, stddevs)]
        lower = [max(0, v - s) for v, s in zip(values, stddevs)]
        plt.fill_between(dates, lower, upper, alpha=0.2, color='#2196F3',
                        label='Std Dev')

    # 格式化
    plt.title(f"{benchmark_name} Performance Trend", fontsize=14, fontweight='bold')
    plt.xlabel("Date", fontsize=12)
    plt.ylabel("Time (nanoseconds)", fontsize=12)
    plt.grid(True, alpha=0.3, linestyle='--')
    plt.legend(loc='best')

    # 格式化x轴日期
    plt.gca().xaxis.set_major_formatter(mdates.DateFormatter('%Y-%m-%d'))
    plt.gca().xaxis.set_major_locator(mdates.DayLocator(interval=max(1, len(dates)//10)))
    plt.xticks(rotation=45, ha='right')

    plt.tight_layout()

    # 保存图表
    output_path = output_dir / f"{benchmark_name.replace('/', '_')}_trend.png"
    plt.savefig(output_path, dpi=150, bbox_inches='tight')
    plt.close()

    print(f"Generated trend chart: {output_path}")
    return True


def generate_summary_report(history: Dict[datetime, Dict], output_dir: Path) -> None:
    """
    生成趋势摘要报告

    Args:
        history: 历史数据
        output_dir: 输出目录
    """
    if len(history) < 2:
        print("Not enough history for summary report")
        return

    dates = sorted(history.keys())
    oldest = dates[0]
    newest = dates[-1]

    oldest_data = history[oldest]
    newest_data = history[newest]

    report_lines = []
    report_lines.append("# Performance Trend Summary\n")
    report_lines.append(f"**Period:** {oldest.strftime('%Y-%m-%d')} to {newest.strftime('%Y-%m-%d')}\n")
    report_lines.append(f"**Data Points:** {len(history)}\n\n")

    report_lines.append("## Benchmark Trends\n\n")
    report_lines.append("| Benchmark | Oldest (ns) | Newest (ns) | Change | % Change |\n")
    report_lines.append("|-----------|-------------|-------------|--------|----------|\n")

    for benchmark_name in newest_data.keys():
        if benchmark_name in oldest_data:
            oldest_val = oldest_data[benchmark_name].get("mean", 0)
            newest_val = newest_data[benchmark_name].get("mean", 0)

            change = newest_val - oldest_val
            pct_change = (change / oldest_val * 100) if oldest_val > 0 else 0

            emoji = "✅" if pct_change < -5 else "❌" if pct_change > 5 else "➖"

            report_lines.append(f"| {benchmark_name} | {oldest_val:.2f} | {newest_val:.2f} | "
                              f"{change:+.2f} | {emoji} {pct_change:+.1f}% |\n")

    report_path = output_dir / "trend_summary.md"
    with open(report_path, 'w') as f:
        f.writelines(report_lines)

    print(f"Generated trend summary: {report_path}")


def main():
    """主函数"""
    parser = argparse.ArgumentParser(description='Generate performance trend charts')
    parser.add_argument('--input', '-i', type=str,
                       help='Input JSON file with current benchmark results')
    parser.add_argument('--trends-dir', '-t', type=str,
                       default='game_engine/benches/trends',
                       help='Trends data directory')
    parser.add_argument('--output-dir', '-o', type=str,
                       default='game_engine/benches/trends',
                       help='Output directory for charts')
    parser.add_argument('--no-save', action='store_true',
                       help='Do not save current results to history')

    args = parser.parse_args()

    # 设置路径
    root_dir = Path(__file__).parent.parent
    trends_dir = root_dir / args.trends_dir
    output_dir = root_dir / args.output_dir

    # 确保输出目录存在
    output_dir.mkdir(parents=True, exist_ok=True)

    # 加载历史数据
    print("Loading historical data...")
    history = load_historical_data(trends_dir)

    if not history:
        print("No historical data found")
        return 0

    print(f"Loaded {len(history)} historical data points")

    # 如果提供了输入文件，保存到历史
    if args.input and not args.no_save:
        input_path = Path(args.input)
        if input_path.exists():
            with open(input_path, 'r') as f:
                current_results = json.load(f)
            save_current_results(current_results, trends_dir)
            history = load_historical_data(trends_dir)  # 重新加载
        else:
            print(f"Warning: Input file not found: {args.input}", file=sys.stderr)

    # 获取最新的数据
    latest_data = list(history.values())[-1]

    print(f"Generating trend charts for {len(latest_data)} benchmarks...")

    # 为每个benchmark生成趋势图
    chart_count = 0
    for benchmark_name in latest_data.keys():
        if generate_trend_chart(history, benchmark_name, output_dir):
            chart_count += 1

    print(f"Generated {chart_count} trend charts")

    # 生成摘要报告
    if MATPLOTLIB_AVAILABLE:
        generate_summary_report(history, output_dir)

    return 0


if __name__ == "__main__":
    sys.exit(main())
