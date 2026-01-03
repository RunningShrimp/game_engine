#!/usr/bin/env python3
"""
Performance Regression Detection Script

Checks benchmark results against a baseline and reports regressions beyond
a specified threshold.
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Tuple

# Performance thresholds (percentage increase is considered regression)
DEFAULT_THRESHOLD = 10.0
CRITICAL_THRESHOLD = 20.0

# Critical benchmarks that must not regress
CRITICAL_BENCHMARKS = [
    "entity_create",
    "entity_read",
    "undo_operations",
    "frustum_culling_cpu",
    "vram_allocation",
]

def parse_criterion_json(json_path: Path) -> Dict:
    """Parse Criterion.rs benchmark JSON output."""
    if not json_path.exists():
        print(f"Warning: Benchmark file {json_path} not found")
        return {}

    with open(json_path, 'r') as f:
        return json.load(f)


def compare_measurements(
    baseline: float, current: float, threshold: float
) -> Tuple[bool, float]:
    """
    Compare two measurements and return (is_regression, percentage_change).
    Positive percentage_change means regression (worse performance).
    """
    if baseline == 0:
        return False, 0.0

    # For time measurements, higher is worse (regression)
    percentage_change = ((current - baseline) / baseline) * 100.0
    is_regression = percentage_change > threshold

    return is_regression, percentage_change


def check_benchmark_regressions(
    baseline_dir: Path, current_dir: Path, threshold: float
) -> List[Dict]:
    """Check all benchmarks for regressions."""
    regressions = []

    # Iterate through benchmark directories
    for benchmark_path in baseline_dir.glob("*/"):
        benchmark_name = benchmark_path.name

        baseline_json = benchmark_path / "benchmark.json"
        current_json = current_dir / benchmark_name / "benchmark.json"

        if not baseline_json.exists() or not current_json.exists():
            continue

        baseline_data = parse_criterion_json(baseline_json)
        current_data = parse_criterion_json(current_json)

        if not baseline_data or not current_data:
            continue

        # Compare measurements
        for group_name, group_data in baseline_data.get("groups", {}).items():
            for benchmark in group_data.get("benchmarks", []):
                bench_id = benchmark.get("id", "")

                # Find corresponding current measurement
                current_group = current_data.get("groups", {}).get(group_name, {})
                current_bench = next(
                    (b for b in current_group.get("benchmarks", []) if b.get("id") == bench_id),
                    None,
                )

                if not current_bench:
                    continue

                # Compare mean execution times
                baseline_mean = benchmark.get("mean", {}).get("ns", 0)
                current_mean = current_bench.get("mean", {}).get("ns", 0)

                is_regression, change = compare_measurements(
                    baseline_mean, current_mean, threshold
                )

                if is_regression:
                    is_critical = bench_id in CRITICAL_BENCHMARKS or change > CRITICAL_THRESHOLD

                    regressions.append(
                        {
                            "benchmark": bench_id,
                            "baseline_ns": baseline_mean,
                            "current_ns": current_mean,
                            "percentage_change": change,
                            "is_critical": is_critical,
                        }
                    )

    return regressions


def format_regressions(regressions: List[Dict]) -> str:
    """Format regression report as Markdown."""
    if not regressions:
        return "✅ No performance regressions detected!"

    lines = [
        "## Performance Regression Report",
        "",
        f"Found {len(regressions)} potential regression(s):",
        "",
        "| Benchmark | Baseline (ns) | Current (ns) | Change | Status |",
        "|-----------|---------------|--------------|--------|--------|",
    ]

    for reg in regressions:
        status = "🔴 CRITICAL" if reg["is_critical"] else "⚠️ WARNING"
        change_str = f"+{reg['percentage_change']:.2f}%"

        lines.append(
            f"| {reg['benchmark']} | {reg['baseline_ns']:.0f} | "
            f"{reg['current_ns']:.0f} | {change_str} | {status} |"
        )

    lines.extend([
        "",
        "### Recommendations",
        "",
        "- Review recent changes that may have affected performance",
        "- Consider rolling back critical regressions",
        "- Update baseline if the change is intentional and acceptable",
        "",
        "### Notes",
        "",
        f"- Threshold: {DEFAULT_THRESHOLD}% increase considered regression",
        f"- Critical threshold: {CRITICAL_THRESHOLD}% increase",
    ])

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Check benchmark results for performance regressions"
    )
    parser.add_argument(
        "--baseline-dir",
        type=Path,
        default=Path("benches/target/criterion/main"),
        help="Path to baseline benchmark results",
    )
    parser.add_argument(
        "--current-dir",
        type=Path,
        default=Path("benches/target/criterion/new"),
        help="Path to current benchmark results",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=DEFAULT_THRESHOLD,
        help="Regression threshold percentage (default: 10.0)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("benchmark_regressions.md"),
        help="Output file for regression report",
    )

    args = parser.parse_args()

    if not args.baseline_dir.exists():
        print(f"Error: Baseline directory {args.baseline_dir} not found")
        sys.exit(1)

    if not args.current_dir.exists():
        print(f"Error: Current directory {args.current_dir} not found")
        sys.exit(1)

    regressions = check_benchmark_regressions(
        args.baseline_dir, args.current_dir, args.threshold
    )

    report = format_regressions(regressions)

    # Write report
    args.output.write_text(report)
    print(f"Regression report written to {args.output}")

    # Print summary
    print("\n" + "=" * 60)
    print(report)
    print("=" * 60)

    # Exit with error if critical regressions found
    critical_regressions = [r for r in regressions if r["is_critical"]]
    if critical_regressions:
        print(f"\n❌ Found {len(critical_regressions)} critical regression(s)!")
        sys.exit(1)
    elif regressions:
        print(f"\n⚠️ Found {len(regressions)} non-critical regression(s)")
        sys.exit(0)
    else:
        print("\n✅ All benchmarks passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
