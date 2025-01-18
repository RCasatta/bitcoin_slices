use bitcoin::consensus::deserialize;
use bitcoin_hashes::sha256d;
use bitcoin_slices::bsl;
use bitcoin_slices::bsl::{parse_len, scan_len};
use bitcoin_slices::bsl::{Block, BlockHeader, FindTransaction, Transaction, TxOut};
use bitcoin_slices::{Parse, Visit, Visitor};
use bitcoin_test_data::blocks::mainnet_702861;
use core::ops::ControlFlow;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hex_lit::hex;
use std::str::FromStr;

const BENCH_TX: [u8; 193] = hex!("0100000001a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff0100e1f505000000001976a9140389035a9225b3839e2bbf32d826a1e222031fd888ac00000000");
const GENESIS_BLOCK_HEADER: [u8; 80] = hex!("0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c");

criterion_group!(
    benches,
    tx_deserialize,
    tx_id,
    block_deserialize,
    block_sum_outputs,
    hash_block_txs,
    find_tx,
    block_hash,
    len,
    script,
    witness,
);
criterion_main!(benches);

pub fn tx_deserialize(c: &mut Criterion) {
    c.benchmark_group("tx_deserialize")
        .throughput(criterion::Throughput::Bytes(BENCH_TX.len() as u64))
        .bench_function("slices", |b| {
            b.iter(|| {
                let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed_owned();
                black_box(&tx);
            })
        })
        .bench_function("bitcoin", |b| {
            b.iter(|| {
                let tx: bitcoin::Transaction = deserialize(&BENCH_TX).unwrap();
                black_box(&tx);
            })
        });
}

pub fn tx_id(c: &mut Criterion) {
    c.benchmark_group("tx_id")
        .throughput(criterion::Throughput::Bytes(BENCH_TX.len() as u64))
        .bench_function("slices_bitcoin_hashes", |b| {
            let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed_owned();
            b.iter(|| {
                black_box(tx.txid());
            })
        })
        .bench_function("slices_sha2", |b| {
            let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed_owned();
            b.iter(|| {
                black_box(tx.txid_sha2());
            })
        })
        .bench_function("bitcoin", |b| {
            let tx: bitcoin::Transaction = deserialize(&BENCH_TX[..]).unwrap();
            b.iter(|| {
                black_box(tx.compute_txid());
            })
        });
}

pub fn block_deserialize(c: &mut Criterion) {
    c.benchmark_group("block_deserialize")
        .throughput(criterion::Throughput::Bytes(mainnet_702861().len() as u64))
        .bench_function("slices", |b| {
            b.iter(|| {
                let block = Block::parse(mainnet_702861()).unwrap();
                black_box(&block);
            })
        })
        .bench_function("bitcoin", |b| {
            b.iter(|| {
                let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
                black_box(&block);
            })
        });
}

pub fn block_sum_outputs(c: &mut Criterion) {
    c.benchmark_group("block_sum_outputs")
        .throughput(criterion::Throughput::Bytes(mainnet_702861().len() as u64))
        .bench_function("slices", |b| {
            b.iter(|| {
                struct Sum(u64);
                impl Visitor for Sum {
                    fn visit_tx_out(&mut self, _vout: usize, tx_out: &TxOut) -> ControlFlow<()> {
                        self.0 += tx_out.value();
                        ControlFlow::Continue(())
                    }
                }
                let mut sum = Sum(0);
                let block = Block::visit(mainnet_702861(), &mut sum).unwrap();
                assert_eq!(sum.0, 2883682728990);
                black_box(&block);
            })
        })
        .bench_function("bitcoin", |b| {
            b.iter(|| {
                let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
                let sum: u64 = block
                    .txdata
                    .iter()
                    .flat_map(|t| t.output.iter())
                    .fold(0, |acc, e| acc + e.value.to_sat());
                assert_eq!(sum, 2883682728990);

                black_box(&block);
            })
        });
}

pub fn hash_block_txs(c: &mut Criterion) {
    c.benchmark_group("hash_block_txs")
        .throughput(criterion::Throughput::Bytes(mainnet_702861().len() as u64))
        .bench_function("slices", |b| {
            b.iter(|| {
                struct VisitTx(Vec<sha256d::Hash>);
                let mut v = VisitTx(vec![]);
                impl Visitor for VisitTx {
                    fn visit_block_begin(&mut self, total_transactions: usize) {
                        self.0.reserve(total_transactions);
                    }
                    fn visit_transaction(&mut self, tx: &Transaction) -> ControlFlow<()> {
                        self.0.push(tx.txid());
                        ControlFlow::Continue(())
                    }
                }

                let block = Block::visit(mainnet_702861(), &mut v).unwrap();

                assert_eq!(v.0.len(), 2500);

                black_box((&block, v));
            })
        })
        .bench_function("slices_sha2", |b| {
            b.iter(|| {
                struct VisitTx(
                    Vec<sha2::digest::generic_array::GenericArray<u8, sha2::digest::typenum::U32>>,
                );
                let mut v = VisitTx(vec![]);
                impl Visitor for VisitTx {
                    fn visit_block_begin(&mut self, total_transactions: usize) {
                        self.0.reserve(total_transactions);
                    }
                    fn visit_transaction(&mut self, tx: &Transaction) -> ControlFlow<()> {
                        self.0.push(tx.txid_sha2());
                        ControlFlow::Continue(())
                    }
                }

                let block = Block::visit(mainnet_702861(), &mut v).unwrap();

                assert_eq!(v.0.len(), 2500);

                black_box((&block, v));
            })
        })
        .bench_function("bitcoin", |b| {
            b.iter(|| {
                let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
                let mut tx_hashes = Vec::with_capacity(block.txdata.len());
                for tx in block.txdata.iter() {
                    tx_hashes.push(tx.compute_txid())
                }
                assert_eq!(tx_hashes.len(), 2500);
                black_box((&block, tx_hashes));
            })
        })
        .bench_function("bitcoin_block_ready", |b| {
            let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
            b.iter(|| {
                let mut tx_hashes = Vec::with_capacity(block.txdata.len());
                for tx in block.txdata.iter() {
                    tx_hashes.push(tx.compute_txid())
                }
                assert_eq!(tx_hashes.len(), 2500);
                black_box((&block, tx_hashes));
            })
        });
}

const TXID: &str = "416a5f96cb63e7649f6f272e7f82a43a97bcf6cfc46184c733344de96ff1e433";
pub fn find_tx(c: &mut Criterion) {
    c.benchmark_group("find_tx")
        .throughput(criterion::Throughput::Bytes(mainnet_702861().len() as u64))
        .bench_function("slices", |b| {
            let txid = bitcoin::Txid::from_str(TXID).unwrap();
            b.iter(|| {
                let mut visitor = FindTransaction::new(txid.clone());
                let _ = Block::visit(&mainnet_702861(), &mut visitor);
                let tx = visitor.tx_found().unwrap();
                assert_eq!(tx.compute_txid(), txid);
                core::hint::black_box(tx);
            })
        })
        .bench_function("bitcoin", |b| {
            let txid = bitcoin::Txid::from_str(TXID).unwrap();

            b.iter(|| {
                let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
                let mut tx = None;
                for current in block.txdata {
                    if current.compute_txid() == txid {
                        tx = Some(current);
                        break;
                    }
                }
                let tx = tx.unwrap();
                assert_eq!(tx.compute_txid(), txid);
                core::hint::black_box(&tx);
            })
        });
}

pub fn block_hash(c: &mut Criterion) {
    c.benchmark_group("block_hash")
        .bench_function("slices", |b| {
            let block_header = BlockHeader::parse(&GENESIS_BLOCK_HEADER)
                .unwrap()
                .parsed_owned();
            b.iter(|| {
                let hash = block_header.block_hash();
                black_box(&hash);
            })
        })
        .bench_function("bitcoin", |b| {
            let block_header: bitcoin::blockdata::block::Header =
                deserialize(&GENESIS_BLOCK_HEADER).unwrap();
            b.iter(|| {
                let hash = block_header.block_hash();
                black_box(&hash);
            })
        });
}

pub fn len(c: &mut Criterion) {
    let slices = [
        &[0x01u8][..],                                               // small value
        &[0xFC][..],                                                 // max small value
        &[0xFD, 0xFD, 0x00][..],                                     // minimal FD encoding
        &[0xFD, 0xFF, 0xFF][..],                                     // max FD value
        &[0xFE, 0x00, 0x00, 0x01, 0x00][..],                         // minimal FE encoding
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF][..],                         // max FE value
        &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00][..], // minimal FF encoding
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF][..], // max FF value
    ];
    c.benchmark_group("len")
        .bench_function("scan_len", |b| {
            b.iter(|| {
                let mut sum = 0u128;
                let mut value_ok = 0u64;
                let mut consumed = 0;
                for slice in slices {
                    let len = scan_len(&slice[..], &mut consumed).unwrap();
                    sum += len as u128;
                    value_ok += 1;
                    black_box(&len);
                }
                assert_eq!(sum, 18446744082299617783); // All values summed
                assert_eq!(value_ok, 8);
            })
        })
        .bench_function("parse_len", |b| {
            b.iter(|| {
                let mut sum = 0u128;
                let mut value_ok = 0u64;
                for slice in slices {
                    let len = parse_len(&slice[..]).unwrap();

                    sum += len.n() as u128;
                    value_ok += 1;
                    black_box(&len.n());
                    black_box(&len.consumed());
                }
                assert_eq!(sum, 18446744082299617783); // All values summed
                assert_eq!(value_ok, 8);
            })
        });
}

pub fn script(c: &mut Criterion) {
    let slices = [
        &[0x01u8, 0x00][..],                               // 1 byte script
        &[0x06u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06][..], // 6 byte script
        &[0xFDu8, 0xFD, 0x00, 0x00][..],                   // minimal FD encoding
        &[0xFDu8, 0xFF, 0xFF][..],                         // invalid - not enough bytes
        &[0xFEu8, 0x00, 0x00, 0x01, 0x00, 0x00][..],       // minimal FE encoding
        &[0xFEu8, 0xFF, 0xFF, 0xFF][..],                   // invalid - not enough bytes
        &[0xFFu8, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00][..], // minimal FF encoding
        &[0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF][..], // invalid - not enough bytes
    ];
    c.benchmark_group("script").bench_function("parse", |b| {
        b.iter(|| {
            for slice in slices {
                let script = bsl::Script::parse(&slice[..]);
                black_box(&script);
            }
        })
    });
}
pub fn witness(c: &mut Criterion) {
    c.benchmark_group("witness")
        .bench_function("parse_single_witness", |b| {
            let slices = [
                &[0x00][..],                                     // empty witness
                &[0x01, 0x01, 0x00][..],                         // one empty element
                &[0x02, 0x01, 0x00, 0x01, 0x00][..],             // two empty elements
                &[0x02, 0x01, 0x42, 0x01, 0x43][..],             // two single byte elements
                &[0x02, 0x02, 0x42, 0x43, 0x02, 0x44, 0x45][..], // two two-byte elements
                &[0x02][..],                         // invalid - missing witness elements
                &[0x01, 0x01][..],                   // invalid - missing witness data
                &[0x02, 0x02, 0x42, 0x43, 0x02][..], // invalid - incomplete second witness
                &[0x02, 0x02, 0x42][..],             // invalid - incomplete first witness
                &[0xFF, 0xFF][..],                   // invalid - too many witness elements
            ];
            b.iter(|| {
                for slice in slices {
                    let witnesses = bsl::Witness::parse(&slice[..]);
                    black_box(&witnesses);
                }
            })
        });
    c.benchmark_group("witness")
        .bench_function("parse_multiple_witnesses", |b| {
            let slices = [
                &[0x01, 0x01, 0x00, 0x02, 0x01, 0x00, 0x01, 0x00][..], // first witness is [[0]], second witness is [[0][0]]
                &[0x01, 0x00][..],                                     // empty witness
                &[0x01, 0x01, 0x00][..],                               // one empty element
                &[0x02, 0x01, 0x00, 0x01, 0x00][..],                   // two empty elements
                &[0x02, 0x01, 0x42, 0x01, 0x43][..],                   // two single byte elements
                &[0x02, 0x02, 0x42, 0x43, 0x02, 0x44, 0x45][..],       // two two-byte elements
                &[0x03, 0x01, 0x42, 0x02, 0x43, 0x44, 0x03, 0x45, 0x46, 0x47][..], // three elements of increasing size
            ];
            b.iter(|| {
                for slice in slices {
                    let witnesses = bsl::Witnesses::parse(&slice[..], 1);
                    black_box(&witnesses);
                }
            })
        });
}
