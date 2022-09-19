[![MIT license](https://img.shields.io/github/license/RCasatta/bitcoin_slices)](https://github.com/RCasatta/bitcoin_slices/blob/master/LICENSE)
[![Crates](https://img.shields.io/crates/v/bitcoin_slices.svg)](https://crates.io/crates/bitcoin_slices)
[![Docs](https://img.shields.io/badge/docs.rs-bitcoin_slices-green)](https://docs.rs/bitcoin_slices)

# Bitcoin slices

ZERO allocations parse library for Bitcoin data structures.

Data is accessed by providing visitor structs for the data the user is interested in.

Data structures are read-only and parsed data must be in memory, no streaming API.

## Tradeoffs

Check the CONS before using this library, use rust-bitcoin if they are too restrictive for your case.

### Pros

* Deserialization is amazingly fast, since no allocation is made during parsing.
* Serialization is incredibly fast, since a slice of the serialized data is kept in the structure.
* hashing a little faster because slice are ready without the need of re-serializing data.
* No mandatory dependency.
* No standard.
* Calculate txid and block hash via optional dep `bitcoin_hashes` or `sha2`.
* Visitor pattern to visit just what you are interested in.

### Cons

* Full data must be in memory, there is no streaming (Read/Write) API.
* Data structure are read-only, cannot be modified.
* Visitor pattern requires user-built data structure for visiting.

## Test

```sh
cargo test
```

## Bench

```sh
RUSTFLAGS='--cfg=bench' cargo +nightly bench --all-features
```

```sh
test bsl::block::bench::block_deserialize            ... bench:     344,768 ns/iter (+/- 683)
test bsl::block::bench::block_deserialize_bitcoin    ... bench:   1,565,341 ns/iter (+/- 174,542)
test bsl::block::bench::block_sum_outputs            ... bench:     347,763 ns/iter (+/- 2,027)
test bsl::block::bench::block_sum_outputs_bitcoin    ... bench:   1,469,288 ns/iter (+/- 143,139)
test bsl::block::bench::hash_block_txs               ... bench:   4,443,346 ns/iter (+/- 88,908)
test bsl::block::bench::hash_block_txs_bitcoin       ... bench:   6,303,477 ns/iter (+/- 45,197)
test bsl::block::bench::hash_block_txs_sha2          ... bench:   1,012,167 ns/iter (+/- 1,492)
test bsl::block_header::bench::block_hash            ... bench:         734 ns/iter (+/- 4)
test bsl::block_header::bench::block_hash_bitcoin    ... bench:         800 ns/iter (+/- 6)
test bsl::transaction::bench::tx_deserialize         ... bench:          58 ns/iter (+/- 1)
test bsl::transaction::bench::tx_deserialize_bitcoin ... bench:         139 ns/iter (+/- 2)
test bsl::transaction::bench::txid                   ... bench:       1,223 ns/iter (+/- 1)
test bsl::transaction::bench::txid_bitcoin           ... bench:       1,358 ns/iter (+/- 8)
test bsl::transaction::bench::txid_sha2              ... bench:         196 ns/iter (+/- 0)
```

* benches ending with `_bitcoin` use `rust-bitcoin`
* benches ending with `_sha2` use `sha2` lib instead of `bitcoin_hashes`

## Fuzz

Use [cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz)
Run fuzzing with transaction as target.

```sh
cargo +nightly fuzz run transaction
```

Other target available in `fuzz/fuzz_targets`


Miniminze corpus:
```
cargo +nightly fuzz cmin transaction
```

## Previous work and credits

* [Bitiodine](https://github.com/mikispag/bitiodine) use similar visitor pattern (parser credited to Mathias Svensson) 
* Some previous work on the idea to parse while reducing allocations in this [PR](https://github.com/rust-bitcoin/rust-bitcoin/pull/672)
* Matt Corallo mentioned something like this in a [comment](https://github.com/rust-bitcoin/rust-bitcoin/pull/672#pullrequestreview-769198159) in that PR

## TODO

- [ ] create rotating buffer that consume and produce keeping a linear memory (rotate back when it can't append), 
this would overcome a bit the absence of streaming API
- [ ] implements network types
- [ ] add github actions CI