# Autoresearch Quickstart

Get started with autonomous optimization in 5 minutes.

## Prerequisites Check

```bash
./test_setup.sh
```

This verifies you have: cargo, python3, jq, bc, and optionally opencode.

## Step 1: Start Your LLM Server

```bash
# Start llama.cpp server with your model
llama-server \
    -m ~/models/Nemotron-3-Nano-30B-A3B-UD-Q4_K_XL.gguf \
    --port 8080 \
    --ctx-size 8192
```

**Test it:**
```bash
curl http://localhost:8080/health
# Should return: {"status":"ok"}
```

## Step 2: Create Autoresearch Branch

```bash
git checkout -b autoresearch
```

## Step 3: Establish Baseline

```bash
python3 run_bench.py > baseline.json
cat baseline.json
# Should show: {"time_ns": ~104900, "status": "success", ...}

jq -r '.time_ns' baseline.json > .best_time
```

## Step 4: Try One Manual Iteration

```bash
./manual_iteration.sh
```

This walks you through one optimization cycle with human oversight.

## Step 5: Start Autonomous Loop

```bash
./agent_loop.sh
```

Or with a limit:
```bash
MAX_ITERATIONS=100 ./agent_loop.sh
```

## Step 6: Monitor Progress

**In another terminal:**

```bash
# Watch results in real-time
watch -n 5 'tail -15 results.tsv | column -t -s $"\t"'

# Check current best
cat .best_time

# Check iteration count
cat .iteration_count

# Analyze results
python3 analyze_results.py
```

## Step 7: Review Results

```bash
# After running for a while (hours or overnight)
python3 analyze_results.py

# View all improvements
grep "improved" results.tsv | column -t -s $'\t'

# View commit history
git log --oneline autoresearch
```

## What to Expect

### With Nemotron-3-Nano-30B (30B params)

- **Success rate:** 5-15% (1 in 10-20 attempts)
- **Per iteration:** 2-5 minutes
- **Per day:** 300-700 iterations (24/7)
- **Expected improvement:** 0.5-2% after overnight run

### Good Signs

✅ Agent makes small, focused changes
✅ Tests pass most of the time
✅ Some iterations show improvement (even 0.1%)
✅ Commit messages are descriptive

### Bad Signs

❌ Agent makes no changes (prompt too vague)
❌ Agent adds unsafe code (need better constraints)
❌ Tests fail frequently (agent confused)
❌ Changes too complex (need simpler instructions)

## Troubleshooting

**"opencode: command not found"**
- Install opencode or modify `agent_loop.sh` to use your CLI tool
- See `OPENCODE_SETUP.md` for alternatives

**"Benchmark failed to run"**
- Make sure project builds: `cargo build --all-features`
- Check that bitcoin-test-data is available

**"Tests keep failing"**
- Agent might be too aggressive. Add constraints in prompt.
- Check what's failing: `cargo test --all-features`

**"No improvements after many iterations"**
- Normal! Code is already optimized.
- Try profiling to find new hotspots: `cargo flamegraph`
- Give agent more specific hints about where to optimize

**"Results are noisy / inconsistent"**
- Increase benchmark sample size in `run_bench.py`
- Run benchmarks when system is idle (no other processes)
- Consider using `nice -n -20` for higher priority

## Key Files

- **program.md** - Instructions for the AI agent
- **run_bench.py** - Runs benchmark and extracts metrics
- **agent_loop.sh** - Main autonomous loop
- **manual_iteration.sh** - Single iteration with human control
- **test_setup.sh** - Verify setup is correct
- **analyze_results.py** - Generate insights from results

- **results.tsv** - Log of all experiments
- **.best_time** - Current best benchmark time
- **.iteration_count** - Current iteration number
- **run.log** - Raw output from last benchmark
- **bench_result.json** - Parsed metrics from last benchmark

## Tips for Success

1. **Start with manual mode** - Run `./manual_iteration.sh` a few times to understand the workflow

2. **Monitor the first 10 iterations** - Make sure the agent is behaving correctly

3. **Profile first** - Run `cargo flamegraph` to identify actual hotspots
   ```bash
   cargo flamegraph --bench benches -- block_deserialize/slices --profile-time 30
   firefox flamegraph.svg
   ```

4. **Give specific hints** - Modify `agent_loop.sh` to mention specific functions to optimize

5. **Be patient** - Small models need many iterations. Let it run overnight.

6. **Review winners** - After a run, review successful commits:
   ```bash
   git log --oneline autoresearch --grep="Optimize"
   ```

7. **Cherry-pick best** - If you want to apply findings to main:
   ```bash
   git checkout main
   git cherry-pick <commit-hash>
   ```

## Example Session

```bash
# Terminal 1: Start LLM server
llama-server -m ~/models/Nemotron-3-Nano-30B.gguf --port 8080

# Terminal 2: Run autoresearch
git checkout -b autoresearch
./test_setup.sh
python3 run_bench.py | tee baseline.json
jq -r '.time_ns' baseline.json > .best_time
./agent_loop.sh

# Terminal 3: Monitor
watch -n 5 'python3 analyze_results.py'
```

## Stopping and Resuming

**Stop:**
```bash
# Press Ctrl+C in the agent_loop.sh terminal
```

**Resume:**
```bash
# Just run it again - it will continue from where it left off
./agent_loop.sh
```

**Reset everything:**
```bash
rm results.tsv .best_time .iteration_count
git checkout main
git branch -D autoresearch
```

## Learn More

- **AUTORESEARCH.md** - Detailed documentation
- **OPENCODE_SETUP.md** - LLM server and opencode configuration
- **program.md** - Instructions given to the AI agent
- **Karpathy's autoresearch** - https://github.com/karpathy/autoresearch

## Philosophy

> "The goal is not to find the one perfect optimization, but to accumulate many tiny improvements that compound over time."
>
> "Prefer elegant 0.001% gains over hacky 0.1% gains."
>
> "Correctness first, speed second."

Happy optimizing! 🚀
