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
        assert_eq!(tx.compute_txid(), txid);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        use core::ops::ControlFlow;

        assert_eq!(std::mem::size_of::<Block>(), 56);

        assert_eq!(std::mem::size_of::<ControlFlow<()>>(), 1);
    }
}
