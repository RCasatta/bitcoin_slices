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

## Features

### Hashing

Use feature `sha2` or `bitcoin_hashes` to calculate hashes of blocks and transactions.
The former is faster, the latter is more likely to be in your tree if you work with rust-bitcoin 
ecosystem's crates.

### redb

With the `redb` feature activated some type allows to be used as value and key in the 
[redb](https://github.com/cberner/redb) database. Bitcoin slices types are well suited to be used
as key and values in the database because conversion from/to slices is immediate.

### rust-bitcoin

With the feature `bitcoin` activated some types allows to be converted in the `rust-bitcoin` 
counterpart: for example `bsl::Transaction` could be converted in `bitcoin::Transaction`. 
You may think if you need `bitcoin::Transaction` you can decode the bytes directly into it without
using this library, and is mostly true, but sometimes it may be convenient to use both types, for
example using bitcoin slices with datatabases is very convenient because conversion is free, but 
you may need to access fields more conveniently than writing a visitor for it and thus convert to 
rust-bitcoin types. Moreover, conversions may leverage type invariants and be faster than starting 
from a generic byte stream.

## Test

```sh
cargo test
```

## Bench

```sh
RUSTFLAGS='--cfg=bench' cargo +nightly bench --all-features
```

```sh
test bsl::block::bench::block_deserialize            ... bench:     289,421 ns/iter (+/- 46,179)
test bsl::block::bench::block_deserialize_bitcoin    ... bench:   2,719,666 ns/iter (+/- 459,186)
test bsl::block::bench::block_sum_outputs            ... bench:     288,248 ns/iter (+/- 39,013)
test bsl::block::bench::block_sum_outputs_bitcoin    ... bench:   2,607,791 ns/iter (+/- 321,212)
test bsl::block::bench::hash_block_txs               ... bench:   8,406,341 ns/iter (+/- 938,119)
test bsl::block::bench::hash_block_txs_bitcoin       ... bench:  11,843,590 ns/iter (+/- 1,052,109)
test bsl::block::bench::hash_block_txs_sha2          ... bench:   7,891,956 ns/iter (+/- 1,047,439)
test bsl::block_header::bench::block_hash            ... bench:       1,399 ns/iter (+/- 205)
test bsl::block_header::bench::block_hash_bitcoin    ... bench:       1,510 ns/iter (+/- 222)
test bsl::transaction::bench::tx_deserialize         ... bench:          38 ns/iter (+/- 8)
test bsl::transaction::bench::tx_deserialize_bitcoin ... bench:         219 ns/iter (+/- 30)
test bsl::transaction::bench::txid                   ... bench:       2,185 ns/iter (+/- 166)
test bsl::transaction::bench::txid_bitcoin           ... bench:       2,416 ns/iter (+/- 213)
test bsl::transaction::bench::txid_sha2              ... bench:       2,085 ns/iter (+/- 216)
```

* benches ending with `_bitcoin` use `rust-bitcoin`
* benches ending with `_sha2` use `sha2` lib instead of `bitcoin_hashes`

### Comparison against rust-bitcoin

`block_deserialize` is almost 10 times faster then `block_deserialize_bitcoin`. It may see unfair 
comparison since you can't for example iterate transactions from the resulted object in case of 
`block_deserialize`, but looking at the `sum_outputs` example where a visitor is used to access 
every outputs in a block we se there isn't noticeable difference.

### Hashing 

`block_hash` and `block_hash_bitcoin` use the same code to hash, however bitcoin_slice is about 7% 
faster because use a slice already available instead of serializing back data.
Similar results apply between `txid` and `txid_bitcoin`.
The performance increase is more notable (30%) between `hash_block_txs` and `hash_block_txs_bitcoin`.

`*_sha2` are not really representative on virtual CI machines since they are not hardware-accellerated. 

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

## Doc

To build docs:

```
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open
```


## Previous work and credits

* [Bitiodine](https://github.com/mikispag/bitiodine) use similar visitor pattern (parser credited to Mathias Svensson) 
* Some previous work on the idea to parse while reducing allocations in this [PR](https://github.com/rust-bitcoin/rust-bitcoin/pull/672)
* Matt Corallo mentioned something like this in a [comment](https://github.com/rust-bitcoin/rust-bitcoin/pull/672#pullrequestreview-769198159) in that PR

## TODO

- [ ] create rotating buffer that consume and produce keeping a linear memory (rotate back when it can't append), 
this would overcome a bit the absence of streaming API
- [ ] implements network types
- [ ] add limited time fuzzing in CI