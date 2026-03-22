# Autonomous Optimization Program

You are an autonomous agent optimizing the performance of the `block_deserialize/slices` benchmark in the bitcoin_slices Rust library.

## Objective

Minimize the **time_ns** metric (lower is better) for parsing a 1.38 MB Bitcoin block containing 2,500 transactions.

## Constraints

You MUST follow these rules when making code changes:

1. **NO UNSAFE CODE** - Do not add `unsafe` blocks or use unsafe Rust features
2. **NO COMPLICATED CODE** - Keep changes simple, readable, and maintainable
3. **NO TOO LONG CODE** - Avoid adding functions longer than 100 lines
4. **PRESERVE CORRECTNESS** - All existing tests must continue to pass
5. **NO NEW DEPENDENCIES** - Only use existing crates in Cargo.toml
6. **NO API CHANGES** - Do not break the public API (preserve function signatures)
7. **NO NIGHTLY FEATURES** - Only use stable Rust features (MSRV: 1.74.0). Do not use nightly-only attributes, APIs, or language features

## Files You Can Modify

Focus your optimizations on these hot-path files:

**Primary targets:**
- `src/bsl/block.rs` - Block parsing logic (Block::parse)
- `src/bsl/transaction.rs` - Transaction parsing (Transaction::visit)
- `src/bsl/len.rs` - Variable-length integer parsing (scan_len)
- `src/bsl/tx_in.rs`, `src/bsl/tx_ins.rs` - Transaction input parsing
- `src/bsl/tx_out.rs`, `src/bsl/tx_outs.rs` - Transaction output parsing
- `src/bsl/witness.rs`, `src/bsl/witnesses.rs` - Witness parsing
- `src/slice.rs` - Slice utility functions
- `src/number.rs` - Number parsing utilities

**Secondary targets:**
- `src/bsl/block_header.rs` - Block header parsing - There is only one header and many transaction in a block, most likely not worth it

**DO NOT MODIFY:**
- `benches/benches.rs` - Benchmark definitions
- `src/error.rs` - Error types
- Public API surface in `src/lib.rs`

## Workflow

### Phase 1: Setup
1. Verify you're on a git branch: `git branch --show-current`
2. If on main, create a new branch: `git checkout -b autoresearch`
3. Read the key source files to understand the current implementation
4. Run initial benchmark to establish baseline: `python3 run_bench.py`

### Phase 2: Experimentation Loop

Repeat this cycle indefinitely:

1. **Analyze:** Review the current code and identify ONE specific optimization opportunity
   - Examples: better inlining, improved branching, cache-friendly data layout, algorithmic improvements

2. **Plan:** Decide on a simple, focused change that respects all constraints

3. **Implement:** Make the code change to a single file (or small set of related files)

4. **Test:** Run tests to ensure correctness
   ```bash
   nix develop -c cargo test -q --all-features
   ```

5. **Commit:** If tests pass, commit your change with descriptive message
   ```bash
   git add <modified_files>
   git commit -m "Optimize <specific_aspect>: <brief_description>"
   ```

6. **Benchmark:** Run the benchmark
   ```bash
   python3 run_bench.py > bench_result.json
   ```

7. **Evaluate:** Check if time_ns improved
   - If `time_ns < previous_best`: SUCCESS - keep the commit
   - If `time_ns >= previous_best`: FAILURE - revert the commit
     ```bash
     git reset --hard HEAD~1
     ```

8. **Log:** Append results to results.tsv (format: commit, time_ns, throughput, status, description)

9. **Continue:** Go back to step 1

### Phase 3: Validation

After every successful improvement:
1. Run extended tests: `nix develop -c cargo test -q --all-features --release`
2. Verify no unsafe code: `grep -r "unsafe" src/`
3. Verify no nightly features: `grep -r "#\!\[feature" src/`
4. Check code complexity: ensure functions stay under 100 lines

## Optimization Ideas

Here are potential areas to explore (start simple, get more sophisticated over time):

**Quick wins:**
- Add `#[inline]` or `#[inline(always)]` to hot functions
- Reduce bounds checking with strategic `.get_unchecked()` patterns (within safe abstractions)
- Eliminate redundant calculations

**Medium complexity:**
- Optimize memory layout of structs (field ordering, alignment)
- Improve cache locality in loops
- Reduce instruction count in tight loops
- Better compiler hints for optimization

**Advanced:**
- SIMD-friendly code patterns (without explicit SIMD)
- Algorithmic improvements (fewer passes over data)
- Specialized fast paths for common transaction patterns

## Success Criteria

An improvement is considered successful if:
1. `time_ns` decreases by 0.5%
2. All tests pass: `nix develop -c cargo test -q --all-features`
3. No unsafe code added
4. Code remains simple and readable
5. No functions exceed 100 lines

## Results Tracking

Maintain `results.tsv` with these columns:
- `commit`: Git commit hash
- `time_ns`: Benchmark time in nanoseconds
- `throughput_gib_s`: Throughput in GiB/s
- `status`: success/failed/timeout
- `improvement_pct`: Percentage improvement over baseline
- `description`: Brief description of the change

## Example Session

```bash
# Initial setup
git checkout -b autoresearch
python3 run_bench.py > baseline.json

# Experiment 1: Add inline hints
vim src/bsl/len.rs  # Add #[inline(always)] to scan_len
nix develop -c cargo test -q --all-features  # Pass
git commit -am "Add inline hint to scan_len"
python3 run_bench.py > result1.json
# time_ns: 104500 (improved!) -> KEEP

# Experiment 2: Optimize branch
vim src/bsl/transaction.rs  # Reorder if conditions
nix develop -c cargo test -q --all-features  # Pass
git commit -am "Optimize transaction branch prediction"
python3 run_bench.py > result2.json
# time_ns: 104600 (worse!) -> REVERT
git reset --hard HEAD~1

# Continue indefinitely...
```

## Remember

- **Smaller changes are better** - One optimization at a time
- **Profile-guided is better than guessing** - Use `cargo flamegraph` if needed
- **Correctness first** - Never sacrifice correctness for speed
- **Simple over clever** - Readable code that's fast is better than unreadable code that's slightly faster
- **Patience** - Even 0.5% improvements compound over many iterations

Good luck! Start simple and iterate.
