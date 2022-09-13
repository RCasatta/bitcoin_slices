use crate::bsl::{BlockHeader, Len, Transaction};
use crate::ParseResult;
use crate::{error::to_unknown, Parse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<'a> {
    slice: &'a [u8],
    block_header: BlockHeader<'a>,
}

impl<'a> Block<'a> {
    #[cfg(feature = "hashes")]
    pub fn block_hash(&self) -> bitcoin_hashes::sha256d::Hash {
        self.block_header.block_hash()
    }

    pub fn total_transactions(&self) -> u64 {
        self.len_transactions().n()
    }
    fn len_transactions(&self) -> Len {
        Len::parse(&self.slice[80..])
            .expect("slice verified during parsing")
            .parsed
    }

    pub fn transactions(&self) -> impl Iterator<Item = Transaction> {
        let len_transactions = self.len_transactions();
        let from = len_transactions.len() + 80;
        TxIterator {
            slice: &self.slice[from..],
            n: len_transactions.n(),
        }
    }
}

pub struct TxIterator<'a> {
    slice: &'a [u8],
    n: u64,
}
impl<'a> Iterator for TxIterator<'a> {
    type Item = Transaction<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let transaction = Transaction::parse(self.slice).ok()?;
        self.slice = transaction.remaining;
        Some(transaction.parsed)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n as usize, Some(self.n as usize))
    }
}
impl<'a> ExactSizeIterator for TxIterator<'a> {}

impl<'a> AsRef<[u8]> for Block<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, Block<'a>> for Block<'a> {
    fn parse(slice: &'a [u8]) -> crate::SResult<Block<'a>> {
        let block_header = BlockHeader::parse(slice).map_err(to_unknown)?;
        let ParseResult {
            mut remaining,
            parsed,
            mut consumed,
        } = Len::parse(block_header.remaining)?;
        for _ in 0..parsed.n() {
            let tx = Transaction::parse(remaining)?;
            remaining = tx.remaining;
            consumed += tx.consumed;
        }
        consumed += 80;

        Ok(ParseResult::new(
            &slice[consumed..],
            Block {
                slice,
                block_header: block_header.parsed,
            },
            consumed,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        bsl::{Block, BlockHeader},
        test_common::GENESIS_BLOCK,
        Parse,
    };

    use crate::test_common::GENESIS_TX;

    #[test]
    fn parse_block() {
        let block_header = BlockHeader::parse(&GENESIS_BLOCK).unwrap();
        let block = Block::parse(&GENESIS_BLOCK).unwrap();

        assert_eq!(block.remaining, &[][..]);
        assert_eq!(
            block.parsed,
            Block {
                slice: &GENESIS_BLOCK,
                block_header: block_header.parsed
            }
        );
        assert_eq!(block.consumed, 285);

        let mut iter = block.parsed.transactions();
        let genesis_tx = iter.next().unwrap();
        assert_eq!(genesis_tx.as_ref(), GENESIS_TX);
        assert!(iter.next().is_none())
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Block>(), 48);
    }
}

#[cfg(bench)]
mod bench {
    use crate::{bsl::Block, Parse};
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
    pub fn block_iter_tx(bh: &mut Bencher) {
        let block = Block::parse(BENCH_BLOCK).unwrap().parsed;
        bh.iter(|| {
            for tx in block.transactions() {
                black_box(&tx);
            }
        });
    }

    #[bench]
    pub fn block_iter_tx_bitcoin(bh: &mut Bencher) {
        let block: bitcoin::Block = deserialize(BENCH_BLOCK).unwrap();
        bh.iter(|| {
            for tx in block.txdata.iter() {
                black_box(&tx);
            }
        });
    }
}
