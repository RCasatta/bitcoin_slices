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
        self.total_txs
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

#[cfg(all(feature = "bitcoin", feature = "sha2"))]
pub mod visitor {
    use core::ops::ControlFlow;

    use bitcoin::consensus::Decodable;
    use bitcoin::hashes::Hash;

    /// Implement a visitor to find a Transaction in a Block given its txid
    pub struct FindTransaction {
        to_find: bitcoin::Txid,
        tx_found: Option<bitcoin::Transaction>,
    }
    impl FindTransaction {
        /// Creates [`FindTransaction`] for txid `to_find`
        pub fn new(to_find: bitcoin::Txid) -> Self {
            Self {
                to_find,
                tx_found: None,
            }
        }
        /// Returns the transaction found if any
        pub fn tx_found(self) -> Option<bitcoin::Transaction> {
            self.tx_found
        }
    }
    impl crate::Visitor for FindTransaction {
        fn visit_transaction(&mut self, tx: &crate::bsl::Transaction) -> ControlFlow<()> {
            let current = bitcoin::Txid::from_slice(tx.txid_sha2().as_slice()).expect("32");
            if self.to_find == current {
                let tx_found = bitcoin::Transaction::consensus_decode(&mut tx.as_ref())
                    .expect("slice validated");
                self.tx_found = Some(tx_found);
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }
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

    #[cfg(all(feature = "bitcoin", feature = "sha2"))]
    #[test]
    fn find_tx() {
        use crate::Visit;
        use bitcoin_test_data::blocks::mainnet_702861;
        use core::str::FromStr;

        let txid = bitcoin::Txid::from_str(
            "416a5f96cb63e7649f6f272e7f82a43a97bcf6cfc46184c733344de96ff1e433",
        )
        .unwrap();
        let mut visitor = crate::bsl::FindTransaction::new(txid.clone());
        let _ = Block::visit(&mainnet_702861(), &mut visitor);
        let tx = visitor.tx_found().unwrap();
        assert_eq!(tx.txid(), txid);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        use core::ops::ControlFlow;

        assert_eq!(std::mem::size_of::<Block>(), 56);

        assert_eq!(std::mem::size_of::<ControlFlow<()>>(), 1);
    }
}

#[cfg(bench)]
mod bench {
    use crate::bsl::{Block, TxOut};
    use crate::{Parse, Visit, Visitor};
    use bitcoin::consensus::deserialize;
    use bitcoin_test_data::blocks::mainnet_702861;
    use test::{black_box, Bencher};

    #[bench]
    pub fn block_deserialize(bh: &mut Bencher) {
        bh.iter(|| {
            let block = Block::parse(mainnet_702861()).unwrap();
            black_box(&block);
        });
    }

    #[bench]
    pub fn block_deserialize_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
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
            let block = Block::visit(mainnet_702861(), &mut sum).unwrap();
            assert_eq!(sum.0, 2883682728990);
            black_box(&block);
        });
    }

    #[bench]
    pub fn block_sum_outputs_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
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
        use core::ops::ControlFlow;

        use bitcoin::hashes::sha256d;

        bh.iter(|| {
            struct VisitTx(Vec<sha256d::Hash>);
            let mut v = VisitTx(vec![]);
            impl crate::Visitor for VisitTx {
                fn visit_block_begin(&mut self, total_transactions: usize) {
                    self.0.reserve(total_transactions);
                }
                fn visit_transaction(&mut self, tx: &crate::bsl::Transaction) -> ControlFlow<()> {
                    self.0.push(tx.txid());
                    ControlFlow::Continue(())
                }
            }

            let block = Block::visit(mainnet_702861(), &mut v).unwrap();

            assert_eq!(v.0.len(), 2500);

            black_box((&block, v));
        });
    }

    #[cfg(feature = "sha2")]
    #[bench]
    pub fn hash_block_txs_sha2(bh: &mut Bencher) {
        use core::ops::ControlFlow;

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
                fn visit_transaction(&mut self, tx: &crate::bsl::Transaction) -> ControlFlow<()> {
                    self.0.push(tx.txid_sha2());
                    ControlFlow::Continue(())
                }
            }

            let block = Block::visit(mainnet_702861(), &mut v).unwrap();

            assert_eq!(v.0.len(), 2500);

            black_box((&block, v));
        });
    }

    #[bench]
    pub fn hash_block_txs_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
            let mut tx_hashes = Vec::with_capacity(block.txdata.len());

            for tx in block.txdata.iter() {
                tx_hashes.push(tx.txid())
            }
            assert_eq!(tx_hashes.len(), 2500);
            black_box((&block, tx_hashes));
        });
    }

    #[cfg(all(feature = "bitcoin", feature = "sha2"))]
    #[bench]
    pub fn find_tx(bh: &mut Bencher) {
        use std::str::FromStr;
        let txid = bitcoin::Txid::from_str(
            "416a5f96cb63e7649f6f272e7f82a43a97bcf6cfc46184c733344de96ff1e433",
        )
        .unwrap();

        bh.iter(|| {
            let mut visitor = crate::bsl::FindTransaction::new(txid.clone());
            let _ = Block::visit(&mainnet_702861(), &mut visitor);
            let tx = visitor.tx_found().unwrap();
            assert_eq!(tx.txid(), txid);
            core::hint::black_box(tx);
        });
    }

    #[cfg(feature = "bitcoin")]
    #[bench]
    pub fn find_tx_bitcoin(bh: &mut Bencher) {
        use std::str::FromStr;
        let txid = bitcoin::Txid::from_str(
            "416a5f96cb63e7649f6f272e7f82a43a97bcf6cfc46184c733344de96ff1e433",
        )
        .unwrap();
        bh.iter(|| {
            let block: bitcoin::Block = deserialize(mainnet_702861()).unwrap();
            let mut tx = None;
            for current in block.txdata {
                if current.txid() == txid {
                    tx = Some(current);
                    break;
                }
            }
            let tx = tx.unwrap();
            assert_eq!(tx.txid(), txid);
            core::hint::black_box(&tx);
        });
    }
}
