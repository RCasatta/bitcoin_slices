# Opencode + Local LLM Setup Guide

This guide explains how to configure opencode to work with your local Nemotron-3-Nano-30B model for autoresearch.

## NixOS Quick Start

If you're on NixOS, use the provided `shell.nix`:

```bash
nix-shell  # Provides all dependencies: cargo, python3, jq, bc, etc.
./test_setup.sh
```

## Prerequisites

### 1. Local LLM Server (llama.cpp)

You need llama.cpp server running with your model.

**Option A: llama-server (recommended)**

```bash
# Install llama.cpp (if not already installed)
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make

# Run server with your model
./llama-server \
    -m /path/to/Nemotron-3-Nano-30B-A3B-UD-Q4_K_XL.gguf \
    --host 0.0.0.0 \
    --port 8080 \
    --ctx-size 8192 \
    --n-predict 2048 \
    --threads 8
```

**Option B: llama.cpp with OpenAI-compatible API**

```bash
./llama-server \
    -m /path/to/Nemotron-3-Nano-30B-A3B-UD-Q4_K_XL.gguf \
    --port 8080 \
    --api-key "dummy-key"
```

**Test the server:**

```bash
curl http://localhost:8080/health
# Should return: {"status":"ok"}

# Test completion
curl http://localhost:8080/v1/completions \
    -H "Content-Type: application/json" \
    -d '{
        "model": "nemotron",
        "prompt": "Write a hello world in Rust:",
        "max_tokens": 100
    }'
```

### 2. Opencode Installation

If you don't have opencode installed, you'll need to install it according to your server's setup.

**Check if installed:**
```bash
which opencode
opencode --version
```

**If using a different CLI tool:**
If your setup uses a different tool to interact with local models, you can modify `agent_loop.sh` to use that tool instead. The key is that it should:
- Accept a prompt via command line or file
- Make code changes to the repository
- Exit with code 0 on success

## Configuring Opencode

### Option 1: Environment Variables

Set these in your shell or in `~/.bashrc`:

```bash
export OPENCODE_API_BASE="http://localhost:8080/v1"
export OPENCODE_MODEL="nemotron"
export OPENCODE_API_KEY="dummy"  # If your server requires auth
```

### Option 2: Config File

Create `~/.config/opencode/config.json`:

```json
{
  "api_base": "http://localhost:8080/v1",
  "model": "nemotron",
  "temperature": 0.7,
  "max_tokens": 2048,
  "timeout": 120
}
```

### Option 3: Per-Project Config

Use the provided `opencode_config.json` in this directory:

```bash
opencode --config opencode_config.json --prompt "..."
```

## Testing the Setup

### 1. Test LLM Server

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "nemotron",
    "messages": [{"role": "user", "content": "Say hello"}],
    "temperature": 0.7
  }'
```

### 2. Test Opencode

```bash
opencode --prompt "Show me the current directory structure"
```

### 3. Test Autoresearch Setup

```bash
./test_setup.sh
```

This will verify:
- All dependencies are installed
- Project builds and tests pass
- Benchmark runs successfully
- Git repository is ready

### 4. Test Manual Iteration

```bash
./manual_iteration.sh
```

This will run a single optimization iteration with human oversight, allowing you to verify the full workflow before running autonomous mode.

## Adjusting for Your Setup

### If opencode isn't available

You can use any LLM CLI tool. Just modify `agent_loop.sh` to replace the opencode call:

**For Claude CLI:**
```bash
claude --prompt "$PROMPT"
```

**For aider:**
```bash
aider --message "$PROMPT" --yes-always
```

**For custom script:**
```bash
python3 your_llm_script.py --prompt "$PROMPT"
```

### If you're using a different model server

Update the endpoint in your config or environment variables:

**For Ollama:**
```bash
export OPENCODE_API_BASE="http://localhost:11434/v1"
export OPENCODE_MODEL="nemotron-30b"
```

**For vLLM:**
```bash
export OPENCODE_API_BASE="http://localhost:8000/v1"
export OPENCODE_MODEL="Nemotron-3-Nano-30B"
```

**For text-generation-webui:**
```bash
export OPENCODE_API_BASE="http://localhost:5000/v1"
export OPENCODE_MODEL="nemotron"
```

## Optimizing for a Small Model

Since you're using Nemotron-3-Nano-30B (a relatively small 30B parameter model), here are tips to maximize success:

### 1. Give Specific Context

Instead of just running `./agent_loop.sh`, you can modify it to provide more specific hints:

```bash
# Edit agent_loop.sh to add specific guidance
PROMPT="$(cat <<EOF
Current best: $BEST_TIME ns

Focus on: scan_len function in src/bsl/len.rs
This is the hottest path for parsing variable-length integers.

Possible optimizations:
- Better inline hints
- Reduce branching
- Improve instruction scheduling

Make ONE small change. Read the file, make the change, run tests, commit.
EOF
)"
```

### 2. Use Temperature Control

Lower temperature for more focused changes:

```json
{
  "temperature": 0.3,
  "top_p": 0.9
}
```

### 3. Provide Examples

Create an `examples/` directory with example optimizations:

```bash
mkdir examples
cat > examples/example_optimization.md <<EOF
# Example: Adding inline hint

Before:
fn scan_len(bytes: &[u8]) -> Option<(usize, usize)> {
    ...
}

After:
#[inline(always)]
fn scan_len(bytes: &[u8]) -> Option<(usize, usize)> {
    ...
}

This helps the compiler inline this hot function, reducing call overhead.
EOF
```

### 4. Run Profiling First

Help the model by providing profiling data:

```bash
cargo flamegraph --bench benches -- block_deserialize/slices --profile-time 30
```

Then mention hot functions in the prompt.

## Troubleshooting

### Problem: Opencode times out

**Solution:** Increase timeout in config:
```json
{"timeout": 300}
```

### Problem: Model makes no changes

**Solution:** Be more specific in the prompt, or provide examples of the kind of changes you want.

### Problem: Model violates constraints (adds unsafe code)

**Solution:** Add a post-commit hook to check:

```bash
cat > .git/hooks/post-commit <<'EOF'
#!/bin/bash
if grep -r "unsafe" src/; then
    echo "ERROR: unsafe code detected!"
    git reset --hard HEAD~1
    exit 1
fi
EOF
chmod +x .git/hooks/post-commit
```

### Problem: Benchmark results are noisy

**Solution:** Increase sample size in `run_bench.py`:
```python
BENCHMARK_RUNS = 500  # More samples = more stable results
```

### Problem: Model makes too complex changes

**Solution:** Add code review in `agent_loop.sh`:

```bash
# After commit, before test
DIFF_LINES=$(git diff HEAD~1 HEAD | wc -l)
if [ $DIFF_LINES -gt 50 ]; then
    echo "Change too large ($DIFF_LINES lines). Reverting."
    git reset --hard HEAD~1
    continue
fi
```

## Running Autoresearch

Once everything is configured:

```bash
# Create branch
git checkout -b autoresearch

# Run test
./test_setup.sh

# Start autonomous loop
./agent_loop.sh

# Monitor in another terminal
watch -n 5 'tail -10 results.tsv | column -t -s $"\t"'
```

## Performance Expectations

With a 30B parameter model, expect:

- **Success rate:** 5-15% (1 in 10-20 attempts improves)
- **Time per iteration:** 2-5 minutes (depending on model speed)
- **Iterations per day:** 300-700 (if running 24/7)
- **Total improvement:** 0.5-2% after overnight run (many small gains)

Karpathy's autoresearch uses much larger models (Claude Opus, GPT-4), so your success rate will be lower, but that's part of the experiment!

## Next Steps

1. ✅ Install and start llama-server
2. ✅ Configure opencode (or alternative)
3. ✅ Run `./test_setup.sh`
4. ✅ Try `./manual_iteration.sh` once
5. ✅ Start `./agent_loop.sh`
6. ✅ Let it run overnight
7. ✅ Analyze results with `python3 analyze_results.py`

Good luck with your experiment!
