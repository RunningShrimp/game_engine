#!/usr/bin/env python3
"""
导出benchmark数据为JSON格式
解析Criterion输出并导出为统一的JSON格式供dashboard使用
"""

import json
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, Any


def parse_criterion_results(criterion_dir: Path) -> Dict[str, Dict[str, Any]]:
    """
    解析Criterion benchmark结果

    Args:
        criterion_dir: Criterion输出目录

    Returns:
        benchmark数据字典
    """
    results = {}

    if not criterion_dir.exists():
        print(f"Warning: Criterion directory not found: {criterion_dir}", file=sys.stderr)
        return results

    # 查找所有benchmark.json文件
    for json_file in criterion_dir.rglob("benchmark.json"):
        try:
            with open(json_file, 'r') as f:
                data = json.load(f)

                name = data.get("group_name", json_file.parent.name)

                # 提取关键指标
                mean_data = data.get("mean", {})
                median_data = data.get("median", {})
                min_data = data.get("min", {})
                max_data = data.get("max", {})

                # 转换为纳秒
                mean = mean_data.get("estimate", 0) / 1e9
                stddev = mean_data.get("stddev", 0) / 1e9
                median = median_data.get("estimate", 0) / 1e9
                min_val = min_data.get("estimate", 0) / 1e9
                max_val = max_data.get("estimate", 0) / 1e9

                # 尝试加载baseline数据（如果存在）
                baseline = mean
                baseline_stddev = 0

                # 查找baseline目录
                baseline_file = json_file.parent / "baseline" / "benchmark.json"
                if baseline_file.exists():
                    try:
                        with open(baseline_file, 'r') as bf:
                            baseline_data = json.load(bf)
                            baseline_mean = baseline_data.get("mean", {})
                            baseline = baseline_mean.get("estimate", mean) / 1e9
                            baseline_stddev = baseline_mean.get("stddev", 0) / 1e9
                    except (json.JSONDecodeError, IOError):
                        pass

                results[name] = {
                    "mean": mean,
                    "stddev": stddev,
                    "median": median,
                    "min": min_val,
                    "max": max_val,
                    "baseline": baseline,
                    "baseline_stddev": baseline_stddev,
                    "unit": "ns",
                    "timestamp": datetime.now().isoformat()
                }

        except (json.JSONDecodeError, KeyError, IOError) as e:
            print(f"Warning: Failed to parse {json_file}: {e}", file=sys.stderr)
            continue

    return results


def export_to_json(data: Dict[str, Dict[str, Any]], output_file: Path) -> None:
    """
    导出数据到JSON文件

    Args:
        data: benchmark数据
        output_file: 输出文件路径
    """
    output_file.parent.mkdir(parents=True, exist_ok=True)

    # 添加元数据
    export_data = {
        "metadata": {
            "timestamp": datetime.now().isoformat(),
            "count": len(data),
            "version": "1.0"
        },
        "benchmarks": data
    }

    with open(output_file, 'w') as f:
        json.dump(export_data, f, indent=2)

    print(f"Exported {len(data)} benchmarks to {output_file}")


def export_to_csv(data: Dict[str, Dict[str, Any]], output_file: Path) -> None:
    """
    导出数据到CSV文件（可选）

    Args:
        data: benchmark数据
        output_file: 输出文件路径
    """
    import csv

    output_file.parent.mkdir(parents=True, exist_ok=True)

    with open(output_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'Benchmark', 'Mean (ns)', 'StdDev (ns)', 'Median (ns)',
            'Min (ns)', 'Max (ns)', 'Baseline (ns)', 'Change (%)'
        ])

        for name, metrics in sorted(data.items()):
            change_pct = 0
            if metrics['baseline'] > 0:
                change_pct = ((metrics['mean'] - metrics['baseline']) / metrics['baseline']) * 100

            writer.writerow([
                name,
                f"{metrics['mean']:.2f}",
                f"{metrics['stddev']:.2f}",
                f"{metrics['median']:.2f}",
                f"{metrics['min']:.2f}",
                f"{metrics['max']:.2f}",
                f"{metrics['baseline']:.2f}",
                f"{change_pct:+.2f}"
            ])

    print(f"Exported CSV to {output_file}")


def main():
    """主函数"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Export benchmark data to JSON format'
    )
    parser.add_argument(
        '--input', '-i',
        type=str,
        default='target/criterion',
        help='Criterion output directory (default: target/criterion)'
    )
    parser.add_argument(
        '--output', '-o',
        type=str,
        default='game_engine/benches/trends/benchmark_data.json',
        help='Output JSON file path'
    )
    parser.add_argument(
        '--csv',
        action='store_true',
        help='Also export to CSV format'
    )
    parser.add_argument(
        '--pretty',
        action='store_true',
        help='Pretty print JSON to stdout'
    )

    args = parser.parse_args()

    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    # 构建路径
    criterion_dir = project_root / args.input
    output_file = project_root / args.output

    print(f"Parsing Criterion results from: {criterion_dir}")
    data = parse_criterion_results(criterion_dir)

    if not data:
        print("Error: No benchmark results found", file=sys.stderr)
        print("Make sure to run 'cargo bench' first", file=sys.stderr)
        return 1

    # 导出JSON
    export_to_json(data, output_file)

    # 导出CSV（如果需要）
    if args.csv:
        csv_file = output_file.with_suffix('.csv')
        export_to_csv(data, csv_file)

    # 打印到stdout（如果需要）
    if args.pretty:
        print("\n" + "=" * 80)
        print(json.dumps(data, indent=2))
        print("=" * 80 + "\n")

    return 0


if __name__ == "__main__":
    sys.exit(main())
