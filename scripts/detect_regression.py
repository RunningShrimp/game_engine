#!/usr/bin/env python3
"""
检测性能回归
分析benchmark结果并检测超过阈值的性能回归
"""

import json
import sys
from pathlib import Path
from typing import List, Dict, Tuple
import argparse

# 颜色输出
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'


def load_criterion_data(directory: Path) -> Dict[str, Dict]:
    """
    加载Criterion JSON数据

    Args:
        directory: 包含benchmark.json文件的目录

    Returns:
        benchmark名称到数据的映射
    """
    results = {}

    if not directory.exists():
        print(f"{Colors.YELLOW}Warning: Directory not found: {directory}{Colors.ENDC}")
        return results

    for json_file in directory.rglob("benchmark.json"):
        try:
            with open(json_file, 'r') as f:
                data = json.load(f)
                name = data.get("group_name", "unknown")

                # 提取均值和标准差
                mean_data = data.get("mean", {})
                mean = mean_data.get("estimate", 0) / 1e9  # 转换为ns
                stddev = mean_data.get("stddev", 0) / 1e9

                results[name] = {
                    "mean": mean,
                    "stddev": stddev,
                    "median": data.get("median", {}).get("estimate", 0) / 1e9,
                }
        except (json.JSONDecodeError, KeyError, IOError) as e:
            print(f"{Colors.YELLOW}Warning: Failed to parse {json_file}: {e}{Colors.ENDC}")
            continue

    return results


def detect_regressions(baseline_data: Dict[str, Dict],
                      current_data: Dict[str, Dict],
                      threshold: float = 10.0,
                      min_improvement: float = -5.0) -> Tuple[List[Dict], List[Dict], List[Dict]]:
    """
    检测性能回归和改进

    Args:
        baseline_data: 基线数据
        current_data: 当前数据
        threshold: 回归阈值（百分比）
        min_improvement: 改进阈值（百分比）

    Returns:
        (回归列表, 改进列表, 稳定列表)
    """
    regressions = []
    improvements = []
    stable = []

    for name, current_metrics in current_data.items():
        if name not in baseline_data:
            continue

        baseline_metrics = baseline_data[name]
        baseline_mean = baseline_metrics["mean"]
        current_mean = current_metrics["mean"]

        if baseline_mean == 0:
            continue

        # 计算变化百分比
        change_pct = ((current_mean - baseline_mean) / baseline_mean) * 100
        change_abs = current_mean - baseline_mean

        # 计算统计显著性（使用3-sigma规则）
        pooled_stddev = (baseline_metrics["stddev"] + current_metrics["stddev"]) / 2
        is_significant = abs(change_abs) > 2 * pooled_stddev

        result = {
            "name": name,
            "baseline": baseline_mean,
            "current": current_mean,
            "change_abs": change_abs,
            "change_pct": change_pct,
            "baseline_stddev": baseline_metrics["stddev"],
            "current_stddev": current_metrics["stddev"],
            "is_significant": is_significant
        }

        if change_pct > threshold and is_significant:
            regressions.append(result)
        elif change_pct < min_improvement and is_significant:
            improvements.append(result)
        else:
            stable.append(result)

    return regressions, improvements, stable


def print_results(regressions: List[Dict],
                 improvements: List[Dict],
                 stable: List[Dict],
                 verbose: bool = False) -> int:
    """
    打印检测结果

    Args:
        regressions: 回归列表
        improvements: 改进列表
        stable: 稳定列表
        verbose: 是否显示详细信息

    Returns:
        退出码（0表示无回归，1表示有回归）
    """
    total = len(regressions) + len(improvements) + len(stable)

    print(f"\n{Colors.BOLD}Performance Regression Analysis{Colors.ENDC}")
    print("=" * 80)
    print(f"Total benchmarks analyzed: {total}")
    print(f"  - Regressed: {len(regressions)}")
    print(f"  - Improved: {len(improvements)}")
    print(f"  - Stable: {len(stable)}")
    print("=" * 80)
    print()

    # 打印回归
    if regressions:
        print(f"{Colors.RED}{Colors.BOLD}❌ PERFORMANCE REGRESSIONS DETECTED{Colors.ENDC}")
        print(f"{Colors.RED}{'=' * 80}{Colors.ENDC}")

        for reg in sorted(regressions, key=lambda x: x["change_pct"], reverse=True):
            print(f"\n{Colors.RED}{Colors.BOLD}  ❌ {reg['name']}{Colors.ENDC}")
            print(f"     Baseline: {reg['baseline']:.2f} ± {reg['baseline_stddev']:.2f} ns")
            print(f"     Current:  {reg['current']:.2f} ± {reg['current_stddev']:.2f} ns")
            print(f"     Change:   +{reg['change_pct']:.2f}% ({reg['change_abs']:+.2f} ns)")
            if reg['is_significant']:
                print(f"     {Colors.YELLOW}Statistically significant{Colors.ENDC}")
        print()

    # 打印改进
    if improvements:
        print(f"{Colors.GREEN}{Colors.BOLD}✅ PERFORMANCE IMPROVEMENTS{Colors.ENDC}")
        print(f"{Colors.GREEN}{'=' * 80}{Colors.ENDC}")

        for imp in sorted(improvements, key=lambda x: x["change_pct"]):
            print(f"\n{Colors.GREEN}{Colors.BOLD}  ✅ {imp['name']}{Colors.ENDC}")
            print(f"     Baseline: {imp['baseline']:.2f} ± {imp['baseline_stddev']:.2f} ns")
            print(f"     Current:  {imp['current']:.2f} ± {imp['current_stddev']:.2f} ns")
            print(f"     Change:   {imp['change_pct']:.2f}% ({imp['change_abs']:+.2f} ns)")
        print()

    # 打印稳定
    if stable and verbose:
        print(f"{Colors.YELLOW}{Colors.BOLD}➖ STABLE BENCHMARKS{Colors.ENDC}")
        print(f"{Colors.YELLOW}{'=' * 80}{Colors.ENDC}")

        for stab in stable:
            print(f"{Colors.YELLOW}  ➖ {stab['name']}: {stab['change_pct']:+.2f}%{Colors.ENDC}")
        print()

    # 最终结论
    if regressions:
        print(f"{Colors.RED}{Colors.BOLD}❌ FAIL: {len(regressions)} benchmark(s) show significant performance regression{Colors.ENDC}")
        return 1
    elif improvements:
        print(f"{Colors.GREEN}{Colors.BOLD}✅ PASS: No regressions detected, {len(improvements)} benchmark(s) improved{Colors.ENDC}")
        return 0
    else:
        print(f"{Colors.GREEN}{Colors.BOLD}✅ PASS: No significant performance changes detected{Colors.ENDC}")
        return 0


def main():
    """主函数"""
    parser = argparse.ArgumentParser(
        description='Detect performance regressions in benchmark results'
    )
    parser.add_argument(
        '--baseline', '-b',
        type=str,
        default='target/criterion/main',
        help='Baseline benchmark directory (default: target/criterion/main)'
    )
    parser.add_argument(
        '--current', '-c',
        type=str,
        default='target/criterion',
        help='Current benchmark directory (default: target/criterion)'
    )
    parser.add_argument(
        '--threshold', '-t',
        type=float,
        default=10.0,
        help='Regression threshold in percent (default: 10.0)'
    )
    parser.add_argument(
        '--improvement', '-i',
        type=float,
        default=-5.0,
        help='Improvement threshold in percent (default: -5.0)'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show stable benchmarks as well'
    )
    parser.add_argument(
        '--exit-zero',
        action='store_true',
        help='Always exit with 0 (useful for CI)'
    )

    args = parser.parse_args()

    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    # 构建完整路径
    baseline_dir = project_root / args.baseline
    current_dir = project_root / args.current

    print(f"{Colors.BLUE}Loading baseline data from: {baseline_dir}{Colors.ENDC}")
    baseline_data = load_criterion_data(baseline_dir)
    print(f"{Colors.BLUE}Found {len(baseline_data)} baseline benchmarks{Colors.ENDC}")

    print(f"{Colors.BLUE}Loading current data from: {current_dir}{Colors.ENDC}")
    current_data = load_criterion_data(current_dir)
    print(f"{Colors.BLUE}Found {len(current_data)} current benchmarks{Colors.ENDC}")

    if not baseline_data:
        print(f"{Colors.RED}Error: No baseline data found{Colors.ENDC}", file=sys.stderr)
        return 1

    if not current_data:
        print(f"{Colors.RED}Error: No current data found{Colors.ENDC}", file=sys.stderr)
        return 1

    # 检测回归
    regressions, improvements, stable = detect_regressions(
        baseline_data,
        current_data,
        args.threshold,
        args.improvement
    )

    # 打印结果
    exit_code = print_results(regressions, improvements, stable, args.verbose)

    # 如果指定--exit-zero，始终返回0
    if args.exit_zero:
        return 0

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
