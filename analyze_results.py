#!/usr/bin/env python3
"""
Analyze autoresearch results and generate insights.
"""

import sys
from pathlib import Path
from collections import defaultdict

def analyze_results(results_file="results.tsv"):
    """Parse and analyze results.tsv file."""
    if not Path(results_file).exists():
        print(f"Error: {results_file} not found", file=sys.stderr)
        return 1

    # Parse results
    improvements = []
    regressions = []
    failures = []
    all_times = []

    with open(results_file, 'r') as f:
        lines = f.readlines()[1:]  # Skip header

    for line in lines:
        parts = line.strip().split('\t')
        if len(parts) < 6:
            continue

        commit, time_ns, throughput, status, improvement_pct, description = parts[:6]
        iteration = parts[6] if len(parts) > 6 else "?"

        if time_ns != "N/A":
            time_val = float(time_ns)
            all_times.append((iteration, time_val, status, description))

            if status == "improved":
                improvements.append((iteration, time_val, improvement_pct, description))
            elif status == "reverted":
                regressions.append((iteration, time_val, improvement_pct, description))
        else:
            failures.append((iteration, status, description))

    # Print analysis
    print("=" * 80)
    print("AUTORESEARCH RESULTS ANALYSIS")
    print("=" * 80)
    print()

    # Summary statistics
    total_experiments = len(lines)
    num_improvements = len(improvements)
    num_regressions = len(regressions)
    num_failures = len(failures)
    success_rate = (num_improvements / total_experiments * 100) if total_experiments > 0 else 0

    print(f"Total experiments:       {total_experiments}")
    print(f"Improvements:            {num_improvements} ({success_rate:.1f}%)")
    print(f"Regressions:             {num_regressions}")
    print(f"Failures:                {num_failures}")
    print()

    # Best and worst times
    if all_times:
        all_times.sort(key=lambda x: x[1])  # Sort by time
        best = all_times[0]
        worst = all_times[-1]

        print(f"Best time:               {best[1]:.1f} ns (iteration {best[0]})")
        print(f"Worst time:              {worst[1]:.1f} ns (iteration {worst[0]})")
        print()

        # Calculate total improvement
        baseline = worst[1]
        final = best[1]
        total_improvement = ((baseline - final) / baseline) * 100

        print(f"Baseline:                {baseline:.1f} ns")
        print(f"Final best:              {final:.1f} ns")
        print(f"Total improvement:       {total_improvement:.2f}%")
        print()

    # Show top improvements
    if improvements:
        print("TOP 10 IMPROVEMENTS:")
        print("-" * 80)
        improvements.sort(key=lambda x: float(x[2]), reverse=True)  # Sort by improvement %
        for i, (iteration, time_ns, improvement_pct, description) in enumerate(improvements[:10], 1):
            print(f"{i:2d}. Iter {iteration:>4s}: {float(improvement_pct):+.3f}% | {time_ns:>10.1f} ns | {description}")
        print()

    # Failure analysis
    if failures:
        print("FAILURES:")
        print("-" * 80)
        failure_types = defaultdict(int)
        for iteration, status, description in failures:
            failure_types[status] += 1

        for status, count in sorted(failure_types.items(), key=lambda x: x[1], reverse=True):
            print(f"  {status}: {count}")
        print()

    # Insights
    print("INSIGHTS:")
    print("-" * 80)
    if num_improvements > 0:
        print(f"✓ {num_improvements} successful optimizations found")
        avg_improvement = sum(float(x[2]) for x in improvements) / len(improvements)
        print(f"✓ Average improvement per success: {avg_improvement:.3f}%")
    else:
        print("✗ No improvements found yet. Keep iterating!")

    if success_rate < 5:
        print(f"⚠ Low success rate ({success_rate:.1f}%). Consider:")
        print("  - Giving the agent more specific optimization hints")
        print("  - Profiling the code to find actual hotspots")
        print("  - Reviewing failed attempts to understand issues")
    elif success_rate > 20:
        print(f"✓ Good success rate ({success_rate:.1f}%)!")

    print()
    print("=" * 80)

    return 0

if __name__ == "__main__":
    sys.exit(analyze_results())
