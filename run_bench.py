#!/usr/bin/env python3
"""
Run the specific benchmark and extract performance metrics.
Similar to autoresearch's train.py execution wrapper.
"""

import subprocess
import json
import re
import sys
from pathlib import Path

# Configuration
BENCHMARK_TARGET = "block_deserialize/slices_block"
BENCHMARK_FEATURES = "--all-features"
BENCHMARK_RUNS = 100  # Criterion sample size

def run_benchmark():
    """Execute the cargo benchmark and capture output."""
    cargo_cmd = [
        "cargo", "bench",
        BENCHMARK_FEATURES,
        "--bench", "benches",
        "--",
        BENCHMARK_TARGET,
        "--sample-size", str(BENCHMARK_RUNS),
        "--save-baseline", "current"
    ]

    # Wrap in direnv exec to ensure correct Rust version (1.74.0 from rust-toolchain.toml)
    # Using direnv exec reuses cache and is faster than nix develop
    cmd = ["direnv", "exec", "."] + cargo_cmd

    print(f"Running: {' '.join(cmd)}", file=sys.stderr)

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
        return result.stdout + result.stderr, result.returncode
    except subprocess.TimeoutExpired:
        return "TIMEOUT", -1

def extract_metrics(output):
    """
    Extract benchmark metrics from Criterion output.

    Expected format:
    block_deserialize/slices_block
                            time:   [104.85 µs 104.90 µs 104.96 µs]
                            thrpt:  [12.558 GiB/s 12.565 GiB/s 12.571 GiB/s]
    """
    metrics = {
        "time_ns": None,
        "throughput_gib_s": None,
        "status": "success"
    }

    if "TIMEOUT" in output:
        metrics["status"] = "timeout"
        return metrics

    if "error" in output.lower() or "failed" in output.lower():
        metrics["status"] = "failed"
        return metrics

    # Extract time in various units (convert to nanoseconds)
    time_patterns = [
        (r'time:\s+\[[\d.]+ [µu]s ([\d.]+) [µu]s [\d.]+ [µu]s\]', 1000),  # microseconds
        (r'time:\s+\[[\d.]+ ns ([\d.]+) ns [\d.]+ ns\]', 1),  # nanoseconds
        (r'time:\s+\[[\d.]+ ms ([\d.]+) ms [\d.]+ ms\]', 1_000_000),  # milliseconds
    ]

    for pattern, multiplier in time_patterns:
        match = re.search(pattern, output)
        if match:
            time_value = float(match.group(1))
            metrics["time_ns"] = time_value * multiplier
            break

    # Extract throughput
    throughput_match = re.search(r'thrpt:\s+\[[\d.]+ GiB/s ([\d.]+) GiB/s [\d.]+ GiB/s\]', output)
    if throughput_match:
        metrics["throughput_gib_s"] = float(throughput_match.group(1))

    return metrics

def get_git_commit():
    """Get current git commit hash."""
    result = subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        capture_output=True,
        text=True
    )
    return result.stdout.strip()

def get_changed_files():
    """Get list of files changed in the last commit."""
    result = subprocess.run(
        ["git", "diff", "--name-only", "HEAD~1", "HEAD"],
        capture_output=True,
        text=True
    )
    return result.stdout.strip().split('\n') if result.stdout else []

def main():
    """Run benchmark and output JSON results."""
    output, returncode = run_benchmark()
    metrics = extract_metrics(output)

    # Add metadata
    metrics["commit"] = get_git_commit()
    metrics["changed_files"] = get_changed_files()
    metrics["returncode"] = returncode

    # Print JSON for easy parsing
    print(json.dumps(metrics, indent=2))

    # Also save raw output for debugging
    Path("run.log").write_text(output)

    return 0 if metrics["status"] == "success" and metrics["time_ns"] else 1

if __name__ == "__main__":
    sys.exit(main())
