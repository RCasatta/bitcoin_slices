# Bitcoin Slices Autoresearch Setup

This directory is set up for autonomous optimization similar to [Karpathy's autoresearch](https://github.com/karpathy/autoresearch).

## Overview

An LLM agent continuously makes small optimizations to the `block_deserialize/slices` benchmark, keeping improvements and reverting regressions.

**Target benchmark:** Parsing a 1.38 MB Bitcoin block (2,500 transactions)
**Current baseline:** ~104,900 nanoseconds (~105 microseconds)
**Goal:** Minimize parsing time while maintaining correctness

## Setup

### Prerequisites

1. **Rust toolchain** with cargo and cargo-criterion
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Python 3** with jq for JSON parsing
   ```bash
   # NixOS: Add to environment.systemPackages or use nix-shell
   nix-shell -p python3 jq bc

   # Or install to user profile
   nix-env -iA nixpkgs.jq nixpkgs.bc

   # Other systems
   # Ubuntu/Debian: sudo apt-get install python3 jq bc
   # macOS: brew install jq bc
   ```

3. **Opencode CLI** configured for your local LLM
   ```bash
   # Install opencode (adjust for your setup)
   # Configure it to point to your llama server running Nemotron-3-Nano-30B
   ```

4. **Local LLM server** (e.g., llama.cpp)
   ```bash
   # Start your llama server with Nemotron-3-Nano-30B-A3B-UD-Q4_K_XL.gguf
   # Example:
   llama-server -m /path/to/Nemotron-3-Nano-30B-A3B-UD-Q4_K_XL.gguf \
                --host localhost --port 8080
   ```

### Initial Setup

1. **Create autoresearch branch:**
   ```bash
   git checkout -b autoresearch
   ```

2. **Establish baseline:**
   ```bash
   python3 run_bench.py > baseline.json
   cat baseline.json
   ```

3. **Review the program:**
   ```bash
   cat program.md  # Read the agent instructions
   ```

## Running Autoresearch

### Automatic Mode (Continuous Loop)

Start the autonomous optimization loop:

```bash
./agent_loop.sh
```

This will:
- Call opencode to make ONE optimization
- Run tests to verify correctness
- Run the benchmark
- Keep the change if it improves time_ns, revert otherwise
- Log all results to `results.tsv`
- Repeat indefinitely (or until MAX_ITERATIONS)

**To limit iterations:**
```bash
MAX_ITERATIONS=100 ./agent_loop.sh
```

**To stop:** Press Ctrl+C

### Manual Mode (Step-by-Step)

For more control, run each step manually:

```bash
# 1. Let opencode make an optimization
opencode --prompt "Read program.md and make one optimization to improve block_deserialize/slices benchmark. Follow all constraints."

# 2. Run tests
cargo test --all-features

# 3. If tests pass, commit
git add <changed_files>
git commit -m "Optimize: <description>"

# 4. Run benchmark
python3 run_bench.py > result.json
cat result.json

# 5. Compare with best time
BEST=$(cat .best_time)
NEW=$(jq -r '.time_ns' result.json)
echo "Best: $BEST ns, New: $NEW ns"

# 6. Keep or revert
if (( $(echo "$NEW < $BEST" | bc -l) )); then
    echo "Improved! Keeping."
    echo "$NEW" > .best_time
else
    echo "Regressed. Reverting."
    git reset --hard HEAD~1
fi
```

## Files

- **run_bench.py** - Runs cargo benchmark and extracts metrics as JSON
- **program.md** - Instructions for the LLM agent
- **agent_loop.sh** - Autonomous optimization loop
- **opencode_config.json** - Configuration for opencode/LLM
- **results.tsv** - Log of all experiments (commit, time, status, improvement%)
- **.best_time** - Current best benchmark time in nanoseconds
- **.iteration_count** - Current iteration number
- **run.log** - Raw benchmark output from last run
- **bench_result.json** - Parsed metrics from last run

## Constraints

The agent MUST follow these rules:

1. ❌ **NO UNSAFE CODE** - No `unsafe` blocks
2. ❌ **NO COMPLICATED CODE** - Keep it simple and readable
3. ❌ **NO TOO LONG CODE** - Functions stay under 100 lines
4. ✅ **PRESERVE CORRECTNESS** - All tests must pass
5. ✅ **NO NEW DEPENDENCIES** - Use only existing crates
6. ✅ **NO API CHANGES** - Keep public API stable

## Monitoring Progress

**View results:**
```bash
cat results.tsv | column -t -s $'\t'
```

**Check current best:**
```bash
echo "Best time: $(cat .best_time) ns"
```

**View successful improvements only:**
```bash
grep "improved" results.tsv
```

**Calculate total improvement:**
```bash
BASELINE=$(head -2 results.tsv | tail -1 | cut -f2)
BEST=$(cat .best_time)
IMPROVEMENT=$(echo "scale=2; (($BASELINE - $BEST) / $BASELINE) * 100" | bc)
echo "Total improvement: ${IMPROVEMENT}%"
```

**View commit history:**
```bash
git log --oneline autoresearch
```

## Tips for Success

1. **Start simple** - The agent should make ONE small change per iteration
2. **Be patient** - Small model + small improvements = many iterations needed
3. **Monitor closely** (at first) - Watch the first 10-20 iterations to ensure it's working
4. **Let it run overnight** - The real gains come from hundreds of experiments
5. **Review winners** - Periodically check which optimizations actually worked

## Expected Behavior

**Good iteration:**
```
Iteration 42
Calling opencode...
Tests passed!
Running benchmark...
Results:
  Previous best: 104500 ns
  New time:      104450 ns
  Change:        0.0478%
🎉 IMPROVEMENT! Keeping commit.
```

**Bad iteration:**
```
Iteration 43
Calling opencode...
Tests passed!
Running benchmark...
Results:
  Previous best: 104450 ns
  New time:      104600 ns
  Change:        -0.1436%
No improvement. Reverting commit.
```

## Troubleshooting

**Benchmark fails to run:**
- Ensure all features are available: `cargo build --all-features`
- Check that bitcoin-test-data is downloaded

**Tests fail:**
- The agent violated correctness - revert will happen automatically
- Review commit to understand what broke

**Opencode doesn't respond:**
- Check that llama server is running: `curl http://localhost:8080/health`
- Verify opencode configuration

**No improvements after many iterations:**
- Normal! The code is already well-optimized
- Even 0.1% improvement is a success
- Try running cargo flamegraph to identify new hotspots

## Advanced: Profiling

To help the agent find optimization opportunities:

```bash
# Install flamegraph
cargo install flamegraph

# Generate profile
cargo flamegraph --bench benches -- block_deserialize/slices --profile-time 30

# Open flamegraph.svg in browser
firefox flamegraph.svg
```

Look for hot functions and mention them in the prompt to opencode.

## Results Analysis

After a long run, analyze the results:

```bash
# Count experiments
echo "Total experiments: $(tail -n +2 results.tsv | wc -l)"

# Count improvements
echo "Successful improvements: $(grep -c "improved" results.tsv)"

# Show all improvements
grep "improved" results.tsv | cut -f2,5,6 | column -t -s $'\t'

# Plot results (requires gnuplot)
tail -n +2 results.tsv | cut -f2 | grep -v "N/A" | \
    gnuplot -e "set terminal dumb; plot '-' with lines title 'time_ns'"
```

## Cleaning Up

To reset and start over:

```bash
rm results.tsv .best_time .iteration_count bench_result.json run.log
git checkout main
git branch -D autoresearch
```

## Philosophy

Like Karpathy's autoresearch:
- **Simplicity over complexity** - Prefer elegant 0.001% gains over hacky 0.1% gains
- **Autonomous operation** - The agent runs without human intervention
- **Accumulation of gains** - Many tiny improvements compound over time
- **Correctness first** - Never sacrifice correctness for speed

Good luck! May your benchmarks be fast and your code be elegant.
