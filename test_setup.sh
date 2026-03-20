#!/usr/bin/env bash
# Test that the autoresearch setup is working correctly

set -e

echo "Testing autoresearch setup..."
echo ""

# Check dependencies
echo "1. Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Install Rust: https://rustup.rs/"
    exit 1
fi
echo "  ✓ cargo found"

if ! command -v python3 &> /dev/null; then
    echo "❌ python3 not found"
    exit 1
fi
echo "  ✓ python3 found"

if ! command -v jq &> /dev/null; then
    echo "❌ jq not found. Install with your package manager (e.g., 'nix-env -iA nixpkgs.jq' or add to environment)"
    exit 1
fi
echo "  ✓ jq found"

if ! command -v bc &> /dev/null; then
    echo "❌ bc not found. Install with your package manager (e.g., 'nix-env -iA nixpkgs.bc' or add to environment)"
    exit 1
fi
echo "  ✓ bc found"

if ! command -v opencode &> /dev/null; then
    echo "⚠️  opencode not found. You'll need to install it to run autoresearch."
    echo "   For now, you can test the benchmark manually."
else
    echo "  ✓ opencode found"

    # Check if local model server is reachable
    if curl -s -m 5 http://ripper:11434/v1/models > /dev/null 2>&1; then
        echo "  ✓ Local model server (ripper:11434) is reachable"
    else
        echo "  ⚠️  Local model server (ripper:11434) is not reachable"
        echo "     Check that the llama.cpp server is running on ripper"
        echo "     You can still run benchmarks manually, but agent_loop.sh won't work"
    fi
fi

echo ""

# Check that the project builds
echo "2. Building project..."
if ! nix develop -c cargo build --all-features --quiet; then
    echo "❌ Build failed"
    exit 1
fi
echo "  ✓ Build successful"

echo ""

# Run tests
echo "3. Running tests..."
if ! nix develop -c cargo test --all-features --quiet; then
    echo "❌ Tests failed"
    exit 1
fi
echo "  ✓ Tests passed"

echo ""

# Run a quick benchmark
echo "4. Running benchmark (this may take 1-2 minutes)..."
if python3 run_bench.py > test_result.json; then
    TIME_NS=$(jq -r '.time_ns' test_result.json)
    STATUS=$(jq -r '.status' test_result.json)

    echo "  ✓ Benchmark completed"
    echo "  Time: ${TIME_NS} ns"
    echo "  Status: ${STATUS}"

    if [ "$STATUS" != "success" ]; then
        echo "  ⚠️  Benchmark status is not 'success'"
    fi
else
    echo "❌ Benchmark failed"
    exit 1
fi

echo ""

# Check git status
echo "5. Checking git repository..."
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not a git repository"
    exit 1
fi
echo "  ✓ Git repository found"

CURRENT_BRANCH=$(git branch --show-current)
echo "  Current branch: ${CURRENT_BRANCH}"

if [ "$CURRENT_BRANCH" = "main" ]; then
    echo "  ℹ️  You're on main. Consider creating an autoresearch branch:"
    echo "     git checkout -b autoresearch"
fi

echo ""

# Summary
echo "=========================================="
echo "Setup test PASSED! ✓"
echo "=========================================="
echo ""
echo "You're ready to start autoresearch!"
echo ""
echo "Quick start:"
echo "  1. Create branch:  git checkout -b autoresearch"
echo "  2. Run automated:  ./agent_loop.sh"
echo "  3. Monitor:        cat results.tsv | column -t -s \$'\\t'"
echo ""
echo "For manual mode, see: AUTORESEARCH.md"
echo ""

# Cleanup
rm -f test_result.json

exit 0
