#!/bin/bash
# Manually run a single autoresearch iteration
# Useful for testing and debugging

set -e

echo "Manual Autoresearch Iteration"
echo "=============================="
echo ""

# Get current best time
if [ -f ".best_time" ]; then
    BEST_TIME=$(cat .best_time)
    echo "Current best time: ${BEST_TIME} ns"
else
    echo "No baseline found. Run: python3 run_bench.py > baseline.json"
    echo "Then: jq -r '.time_ns' baseline.json > .best_time"
    exit 1
fi

echo ""

# Store current commit
PREV_COMMIT=$(git rev-parse HEAD)
echo "Current commit: ${PREV_COMMIT}"
echo ""

# Prompt user for what to optimize
read -p "Enter optimization description (or press Enter to use opencode): " OPT_DESC

if [ -z "$OPT_DESC" ]; then
    # Use opencode
    echo ""
    echo "Calling opencode..."
    echo ""

    PROMPT="You are optimizing the bitcoin_slices benchmark.

Current best time: ${BEST_TIME} nanoseconds

Read program.md for full instructions. Then:
1. Read the relevant source files
2. Identify ONE specific optimization opportunity
3. Make a simple, focused change
4. Commit your change with a descriptive message

Remember the constraints:
- NO unsafe code
- NO complicated code
- NO functions longer than 100 lines
- Must pass all tests

Focus on a single, measurable improvement."

    opencode --prompt "$PROMPT"

    if [ $? -ne 0 ]; then
        echo "Opencode failed!"
        exit 1
    fi
else
    echo ""
    echo "Manual mode: Make your changes now, then press Enter to continue..."
    read -p ""

    # Commit the changes
    echo ""
    read -p "Commit message: " COMMIT_MSG
    git add -A
    git commit -m "${COMMIT_MSG}"
fi

echo ""

# Check if changes were made
NEW_COMMIT=$(git rev-parse HEAD)
if [ "$NEW_COMMIT" = "$PREV_COMMIT" ]; then
    echo "No changes made. Exiting."
    exit 0
fi

COMMIT_HASH=$(git rev-parse --short HEAD)
COMMIT_MSG=$(git log -1 --pretty=format:'%s')

echo "New commit: ${COMMIT_HASH} - ${COMMIT_MSG}"
echo ""

# Run tests
echo "Running tests..."
if cargo test --all-features --quiet; then
    echo "✓ Tests passed"
else
    echo "✗ Tests FAILED!"
    read -p "Revert changes? (y/n): " REVERT
    if [ "$REVERT" = "y" ]; then
        git reset --hard "$PREV_COMMIT"
        echo "Changes reverted."
    fi
    exit 1
fi

echo ""

# Run benchmark
echo "Running benchmark..."
if python3 run_bench.py > bench_result.json; then
    echo "✓ Benchmark completed"
else
    echo "✗ Benchmark FAILED!"
    read -p "Revert changes? (y/n): " REVERT
    if [ "$REVERT" = "y" ]; then
        git reset --hard "$PREV_COMMIT"
        echo "Changes reverted."
    fi
    exit 1
fi

echo ""

# Extract results
NEW_TIME=$(jq -r '.time_ns' bench_result.json)
THROUGHPUT=$(jq -r '.throughput_gib_s' bench_result.json)
STATUS=$(jq -r '.status' bench_result.json)

if [ "$NEW_TIME" = "null" ] || [ -z "$NEW_TIME" ]; then
    echo "✗ Could not extract time from benchmark!"
    cat bench_result.json
    exit 1
fi

# Calculate improvement
IMPROVEMENT=$(echo "scale=4; (($BEST_TIME - $NEW_TIME) / $BEST_TIME) * 100" | bc)

echo "Results:"
echo "  Previous best: ${BEST_TIME} ns"
echo "  New time:      ${NEW_TIME} ns"
echo "  Change:        ${IMPROVEMENT}%"
echo ""

# Compare
KEEP=0
if (( $(echo "$NEW_TIME < $BEST_TIME" | bc -l) )); then
    echo "🎉 IMPROVEMENT!"
    KEEP=1
else
    echo "No improvement."
fi

echo ""

# Ask user what to do
if [ $KEEP -eq 1 ]; then
    read -p "Keep this commit? (Y/n): " RESPONSE
    RESPONSE=${RESPONSE:-Y}
else
    read -p "Keep this commit anyway? (y/N): " RESPONSE
    RESPONSE=${RESPONSE:-N}
fi

if [[ "$RESPONSE" =~ ^[Yy]$ ]]; then
    echo "Keeping commit."

    if [ $KEEP -eq 1 ]; then
        echo "$NEW_TIME" > .best_time
        echo "Updated best time to ${NEW_TIME} ns"

        # Log to results
        if [ ! -f "results.tsv" ]; then
            echo -e "commit\ttime_ns\tthroughput_gib_s\tstatus\timprovement_pct\tdescription\titeration" > results.tsv
        fi
        ITER=$(wc -l < results.tsv)
        echo -e "${COMMIT_HASH}\t${NEW_TIME}\t${THROUGHPUT}\timproved\t${IMPROVEMENT}\t${COMMIT_MSG}\t${ITER}" >> results.tsv
    else
        # Log as kept but not improved
        if [ ! -f "results.tsv" ]; then
            echo -e "commit\ttime_ns\tthroughput_gib_s\tstatus\timprovement_pct\tdescription\titeration" > results.tsv
        fi
        ITER=$(wc -l < results.tsv)
        echo -e "${COMMIT_HASH}\t${NEW_TIME}\t${THROUGHPUT}\tkept\t${IMPROVEMENT}\t${COMMIT_MSG}\t${ITER}" >> results.tsv
    fi
else
    echo "Reverting commit..."
    git reset --hard "$PREV_COMMIT"
    echo "Changes reverted."

    # Log as reverted
    if [ ! -f "results.tsv" ]; then
        echo -e "commit\ttime_ns\tthroughput_gib_s\tstatus\timprovement_pct\tdescription\titeration" > results.tsv
    fi
    ITER=$(wc -l < results.tsv)
    echo -e "${COMMIT_HASH}\t${NEW_TIME}\t${THROUGHPUT}\treverted\t${IMPROVEMENT}\t${COMMIT_MSG}\t${ITER}" >> results.tsv
fi

echo ""
echo "Done!"
