#!/usr/bin/env bash
# Autonomous optimization loop using opencode
# Similar to autoresearch's continuous experimentation

set -e

# Configuration
RESULTS_FILE="results.tsv"
BEST_TIME_FILE=".best_time"
ITERATION_COUNT_FILE=".iteration_count"
MAX_ITERATIONS=${MAX_ITERATIONS:-1000}  # Safety limit, set to 0 for unlimited

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Initialize results file if it doesn't exist
if [ ! -f "$RESULTS_FILE" ]; then
    echo -e "commit\ttime_ns\tthroughput_gib_s\tstatus\timprovement_pct\tdescription\titeration" > "$RESULTS_FILE"
    echo "Created $RESULTS_FILE"
fi

# Get initial baseline if needed
if [ ! -f "$BEST_TIME_FILE" ]; then
    echo "Establishing baseline..."
    python3 run_bench.py > baseline.json
    BASELINE_TIME=$(jq -r '.time_ns' baseline.json)
    echo "$BASELINE_TIME" > "$BEST_TIME_FILE"
    echo "Baseline: ${BASELINE_TIME} ns"
fi

# Initialize iteration counter
if [ ! -f "$ITERATION_COUNT_FILE" ]; then
    echo "0" > "$ITERATION_COUNT_FILE"
fi

BEST_TIME=$(cat "$BEST_TIME_FILE")
ITERATION=$(cat "$ITERATION_COUNT_FILE")

echo "Starting autonomous optimization loop..."
echo "Current best time: ${BEST_TIME} ns"
echo "Starting at iteration: ${ITERATION}"
echo ""

# Main loop
while true; do
    ITERATION=$((ITERATION + 1))
    echo "$ITERATION" > "$ITERATION_COUNT_FILE"

    # Check iteration limit
    if [ "$MAX_ITERATIONS" -gt 0 ] && [ "$ITERATION" -gt "$MAX_ITERATIONS" ]; then
        echo -e "${YELLOW}Reached maximum iterations ($MAX_ITERATIONS). Stopping.${NC}"
        break
    fi

    echo "=========================================="
    echo "Iteration $ITERATION"
    echo "=========================================="

    # Store current commit for potential revert
    PREV_COMMIT=$(git rev-parse HEAD)

    # Call opencode to make an optimization
    echo "Calling opencode to make optimization..."
    opencode --prompt "$(cat <<'EOF'
You are in iteration $ITERATION of autonomous optimization.

Current best benchmark time: $BEST_TIME nanoseconds

Read the program.md file for instructions, then:
1. Analyze the current code and identify ONE optimization opportunity
2. Make a simple, focused change (respecting all constraints)
3. Run tests: cargo test --all-features
4. If tests pass, commit your change with a descriptive message
5. DO NOT run the benchmark yourself - the automation script will handle it

Focus on one small, measurable improvement. Think step by step.
EOF
)"

    OPENCODE_EXIT=$?

    # Check if opencode made changes
    NEW_COMMIT=$(git rev-parse HEAD)

    if [ "$NEW_COMMIT" = "$PREV_COMMIT" ]; then
        echo -e "${YELLOW}No changes made by opencode. Skipping benchmark.${NC}"
        continue
    fi

    if [ $OPENCODE_EXIT -ne 0 ]; then
        echo -e "${RED}Opencode failed with exit code $OPENCODE_EXIT${NC}"
        echo "Reverting changes..."
        git reset --hard "$PREV_COMMIT"
        continue
    fi

    # Get commit info
    COMMIT_HASH=$(git rev-parse --short HEAD)
    COMMIT_MSG=$(git log -1 --pretty=format:'%s')

    echo ""
    echo "Running tests..."
    if ! cargo test --all-features --quiet; then
        echo -e "${RED}Tests failed! Reverting commit.${NC}"
        git reset --hard "$PREV_COMMIT"

        # Log failure
        echo -e "${COMMIT_HASH}\tN/A\tN/A\tfailed_tests\t0.0\t${COMMIT_MSG}\t${ITERATION}" >> "$RESULTS_FILE"
        continue
    fi

    echo -e "${GREEN}Tests passed!${NC}"
    echo ""

    # Run benchmark
    echo "Running benchmark..."
    if ! python3 run_bench.py > bench_result.json; then
        echo -e "${RED}Benchmark failed! Reverting commit.${NC}"
        git reset --hard "$PREV_COMMIT"

        # Log failure
        echo -e "${COMMIT_HASH}\tN/A\tN/A\tfailed_bench\t0.0\t${COMMIT_MSG}\t${ITERATION}" >> "$RESULTS_FILE"
        continue
    fi

    # Extract metrics
    NEW_TIME=$(jq -r '.time_ns' bench_result.json)
    THROUGHPUT=$(jq -r '.throughput_gib_s' bench_result.json)
    STATUS=$(jq -r '.status' bench_result.json)

    if [ "$NEW_TIME" = "null" ] || [ -z "$NEW_TIME" ]; then
        echo -e "${RED}Could not extract benchmark time! Reverting commit.${NC}"
        git reset --hard "$PREV_COMMIT"

        # Log failure
        echo -e "${COMMIT_HASH}\tN/A\tN/A\tfailed_extract\t0.0\t${COMMIT_MSG}\t${ITERATION}" >> "$RESULTS_FILE"
        continue
    fi

    # Calculate improvement
    IMPROVEMENT=$(echo "scale=4; (($BEST_TIME - $NEW_TIME) / $BEST_TIME) * 100" | bc)

    echo ""
    echo "Results:"
    echo "  Previous best: ${BEST_TIME} ns"
    echo "  New time:      ${NEW_TIME} ns"
    echo "  Change:        ${IMPROVEMENT}%"

    # Compare with best time
    if (( $(echo "$NEW_TIME < $BEST_TIME" | bc -l) )); then
        echo -e "${GREEN}🎉 IMPROVEMENT! Keeping commit.${NC}"
        echo "$NEW_TIME" > "$BEST_TIME_FILE"
        BEST_TIME=$NEW_TIME

        # Log success
        echo -e "${COMMIT_HASH}\t${NEW_TIME}\t${THROUGHPUT}\timproved\t${IMPROVEMENT}\t${COMMIT_MSG}\t${ITERATION}" >> "$RESULTS_FILE"
    else
        echo -e "${RED}No improvement. Reverting commit.${NC}"
        git reset --hard "$PREV_COMMIT"

        # Log regression
        echo -e "${COMMIT_HASH}\t${NEW_TIME}\t${THROUGHPUT}\treverted\t${IMPROVEMENT}\t${COMMIT_MSG}\t${ITERATION}" >> "$RESULTS_FILE"
    fi

    echo ""
    sleep 2  # Brief pause between iterations
done

echo ""
echo "Optimization loop completed!"
echo "Final best time: $(cat $BEST_TIME_FILE) ns"
echo "Total iterations: $(cat $ITERATION_COUNT_FILE)"
echo "Results saved to $RESULTS_FILE"
