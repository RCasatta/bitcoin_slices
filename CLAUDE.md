# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`bitcoin_slices` is a zero-allocation parsing library for Bitcoin data structures (blocks, transactions, etc.). It uses a visitor pattern to access data without deserialization overhead. Key design principle: parse Bitcoin data by keeping references to byte slices rather than allocating new structures.

## Core Architecture

### Parsing Model

The library provides two primary traits in `src/visit.rs`:

- **`Parse<'a>`**: Parse objects without visiting (e.g., `bsl::Transaction::parse(bytes)`)
- **`Visit<'a>`**: Parse while calling visitor methods for data you're interested in

All parsed types (`bsl::Block`, `bsl::Transaction`, etc.) keep a reference to the original byte slice. "Deserialization" is essentially free since it's just creating thin wrappers around existing bytes.

### Visitor Pattern

Users implement `Visitor` trait to extract specific data during parsing. The visitor pattern avoids:
- Re-parsing data multiple times
- Allocating collections to hold intermediate results

Example: To sum transaction outputs, implement `visit_tx_out()` instead of deserializing all outputs into a Vec.

### Module Structure

- **`src/bsl/`**: Bitcoin Slice types (Block, Transaction, TxIn, TxOut, BlockHeader, Script, etc.)
  - Each type wraps a byte slice and exposes methods to access fields
  - No iteration APIs - use visitors instead
- **`src/visit.rs`**: Core traits (`Parse`, `Visit`, `Visitor`) and `EmptyVisitor`
- **`src/error.rs`**: Error types (keep `Error` size small for performance - currently 8 bytes on 64-bit)
- **`src/slice_cache.rs`**: Optional FIFO cache for serialized objects (requires `slice_cache` feature)
- **`src/number.rs`**: VarInt and numeric parsing utilities

## Development Commands

### Building
```bash
# Default build (no features)
cargo build

# Build with all features
cargo build --all-features

# Build with specific features
cargo build --features bitcoin,sha2
cargo build --features slice_cache
cargo build --features redb
```

### Testing
```bash
# Run all tests with all features
cargo test --all-features

# Run tests with default features
cargo test

# Run a specific test
cargo test test_name
```

### Benchmarking
```bash
# Run all benchmarks (requires all features)
cargo bench --all-features

# Compact output format
cargo bench --bench benches --all-features -- --output-format bencher
```

### Linting
```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy -- -D warnings

# Format code
cargo fmt --all
```

### Fuzzing
```bash
# Run fuzzing on a specific target (requires nightly)
cargo +nightly fuzz run transaction
cargo +nightly fuzz run block
cargo +nightly fuzz run block_header

# List all fuzz targets
ls fuzz/fuzz_targets/

# Minimize corpus
cargo +nightly fuzz cmin transaction

# For NixOS users, there's a flake in fuzz/ directory
cd fuzz
nix develop
cargo fuzz run transaction
```

Available fuzz targets: `block`, `block_header`, `len`, `out_point`, `script`, `transaction`, `tx_in`, `tx_ins`, `tx_out`, `tx_outs`, `witness`, `witnesses`

### Nix Development Environment

A `flake.nix` is provided that automatically uses the Rust version specified in `rust-toolchain.toml`:

```bash
# Enter the Nix development shell
nix develop

# Or run a command directly
nix develop --command cargo build
```

### Documentation
```bash
# Build and open docs with all features
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open
```

## Features

- **`bitcoin_hashes`**: Calculate txid/block hash using `bitcoin_hashes` crate
- **`sha2`**: Calculate txid/block hash using `sha2` crate (faster than bitcoin_hashes)
- **`bitcoin`**: Conversions to/from `rust-bitcoin` types
- **`redb`**: Use bsl types as keys/values in redb database
- **`slice_cache`**: FIFO cache for serialized objects (requires allocator)
- **`prometheus`**: Prometheus metrics for slice_cache (implies slice_cache)

Default features: none (no_std by default)

## MSRV

Minimum Supported Rust Version: 1.74.0 (defined in `rust-toolchain.toml`)

## Important Implementation Notes

### no_std Support

The crate is `no_std` by default. Only `slice_cache` and `prometheus` features require an allocator (via `alloc` crate).

### Performance Considerations

- Keep `Error` type small (8 bytes on 64-bit) - this impacts performance
- Avoid allocations during parsing - use visitors instead
- Serialization is instant since we keep the original slice
- Hashing is faster because we use the slice directly without re-serializing

### Visitor Implementation

When implementing visitors:
- Return `ControlFlow::Continue(())` to continue visiting
- Return `ControlFlow::Break(())` to stop early (will return `Error::VisitBreak`)
- Not every visit method is called for every parse (e.g., `visit_block_header` is only called when parsing blocks)

### Type Conversions

With the `bitcoin` feature, bsl types can convert to rust-bitcoin types. This is useful when you need more convenient field access after parsing with bitcoin_slices for performance.

### Database Integration

With the `redb` feature, bsl types implement `RedbValue` and `RedbKey`, making them ideal for database storage since conversion to/from bytes is zero-cost.
