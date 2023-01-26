use super::len::{parse_len, Len};
use crate::bsl::{BlockHeader, Transaction};
use crate::{ParseResult, SResult, Visit, Visitor};

/// A Bitcoin block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<'a> {
    slice: &'a [u8],
    header: BlockHeader<'a>,
    total_txs: usize,
}

impl<'a> Visit<'a> for Block<'a> {
    fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Self> {
        let header = BlockHeader::visit(slice, visit)?;
        let Len { mut consumed, n } = parse_len(header.remaining())?;
        consumed += 80;
        let total_txs = n as usize;
        let mut remaining = &slice[consumed..];

        visit.visit_block_begin(total_txs);
        for _ in 0..total_txs {
            let tx = Transaction::visit(remaining, visit)?;
            remaining = tx.remaining();
            consumed += tx.consumed();
        }

        let (slice, remaining) = slice.split_at(consumed);
        let parsed = Block {
            slice,
            header: header.parsed_owned(),
            total_txs,
        };
        Ok(ParseResult::new(remaining, parsed))
    }
}

impl<'a> Block<'a> {
    /// Returns the hash of this block
    #[cfg(feature = "bitcoin_hashes")]
    pub fn block_hash(&self) -> crate::bitcoin_hashes::sha256d::Hash {
        self.header.block_hash()
    }

    /// Calculate the block hash using the sha2 crate.
    /// NOTE: the result type is not displayed backwards when converted to string.
    #[cfg(feature = "sha2")]
    pub fn block_hash_sha2(
        &self,
    ) -> crate::sha2::digest::generic_array::GenericArray<u8, crate::sha2::digest::typenum::U32>
    {
        self.header.block_hash_sha2()
    }

    /// Returns the total transactions in this block
    pub fn total_transactions(&self) -> usize {
        self.total_txs as usize
    }

    /// Returns the header in this block
    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
}

impl<'a> AsRef<[u8]> for Block<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(test)]
mod test {
    use crate::{
        bsl::{Block, BlockHeader},
        test_common::GENESIS_BLOCK,
        Parse,
    };

    #[test]
    fn parse_block() {
        let block_header = BlockHeader::parse(&GENESIS_BLOCK).unwrap();
        let block = Block::parse(&GENESIS_BLOCK).unwrap();

        assert_eq!(block.remaining(), &[][..]);
        assert_eq!(
            block.parsed(),
            &Block {
                slice: &GENESIS_BLOCK,
                header: block_header.parsed_owned(),
                total_txs: 1
            }
        );
        assert_eq!(block.consumed(), 285);

        // let mut iter = block.parsed.transactions();
        // let genesis_tx = iter.next().unwrap();
        // assert_eq!(genesis_tx.as_ref(), GENESIS_TX);
        // assert!(iter.next().is_none())
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Block>(), 56);
    }
}

#[cfg(bench)]
mod bench {
    use crate::bsl::{Block, TxOut};
    use crate::{Parse, Visit, Visitor};
    use bitcoin::consensus::deserialize;
    use test::{black_box, Bencher};

    const BENCH_BLOCK: &[u8; 1_381_836] = include_bytes!("../../test_data/mainnet_block_000000000000000000000c835b2adcaedc20fdf6ee440009c249452c726dafae.raw");

    #[bench]
    pub fn block_deserialize(bh: &mut Bencher) {
        bh.iter(|| {
            let block = Block::parse(BENCH_BLOCK).unwrap();
            black_box(&block);
        });
    }

    #[bench]
    pub fn block_deserialize_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(BENCH_BLOCK).unwrap();
            black_box(&block);
        });
    }

    #[bench]
    pub fn block_sum_outputs(bh: &mut Bencher) {
        bh.iter(|| {
            struct Sum(u64);
            impl Visitor for Sum {
                fn visit_tx_out(&mut self, _vout: usize, tx_out: &TxOut) {
                    self.0 += tx_out.value();
                }
            }
            let mut sum = Sum(0);
            let block = Block::visit(BENCH_BLOCK, &mut sum).unwrap();
            assert_eq!(sum.0, 2883682728990);
            black_box(&block);
        });
    }

    #[bench]
    pub fn block_sum_outputs_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(BENCH_BLOCK).unwrap();
            let sum: u64 = block
                .txdata
                .iter()
                .flat_map(|t| t.output.iter())
                .fold(0, |acc, e| acc + e.value);
            assert_eq!(sum, 2883682728990);

            black_box(&block);
        });
    }

    #[cfg(feature = "bitcoin_hashes")]
    #[bench]
    pub fn hash_block_txs(bh: &mut Bencher) {
        use bitcoin::hashes::sha256d;

        bh.iter(|| {
            struct VisitTx(Vec<sha256d::Hash>);
            let mut v = VisitTx(vec![]);
            impl crate::Visitor for VisitTx {
                fn visit_block_begin(&mut self, total_transactions: usize) {
                    self.0.reserve(total_transactions);
                }
                fn visit_transaction(&mut self, tx: &crate::bsl::Transaction) {
                    self.0.push(tx.txid());
                }
            }

            let block = Block::visit(&BENCH_BLOCK[..], &mut v).unwrap();

            assert_eq!(v.0.len(), 2500);

            black_box((&block, v));
        });
    }

    #[cfg(feature = "sha2")]
    #[bench]
    pub fn hash_block_txs_sha2(bh: &mut Bencher) {
        bh.iter(|| {
            struct VisitTx(
                Vec<
                    crate::sha2::digest::generic_array::GenericArray<
                        u8,
                        crate::sha2::digest::typenum::U32,
                    >,
                >,
            );
            let mut v = VisitTx(vec![]);
            impl crate::Visitor for VisitTx {
                fn visit_block_begin(&mut self, total_transactions: usize) {
                    self.0.reserve(total_transactions);
                }
                fn visit_transaction(&mut self, tx: &crate::bsl::Transaction) {
                    self.0.push(tx.txid_sha2());
                }
            }

            let block = Block::visit(&BENCH_BLOCK[..], &mut v).unwrap();

            assert_eq!(v.0.len(), 2500);

            black_box((&block, v));
        });
    }

    #[bench]
    pub fn hash_block_txs_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(BENCH_BLOCK).unwrap();
            let mut tx_hashes = Vec::with_capacity(block.txdata.len());

            for tx in block.txdata.iter() {
                tx_hashes.push(tx.txid())
            }
            assert_eq!(tx_hashes.len(), 2500);
            black_box((&block, tx_hashes));
        });
    }
}
