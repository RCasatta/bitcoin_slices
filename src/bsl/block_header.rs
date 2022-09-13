use crate::{
    number::{read_i32, read_u32},
    slice::read_slice,
    Error, Parse, ParseResult, SResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader<'a> {
    slice: &'a [u8],
    version: i32,
    time: u32,
    bits: u32,
    nonce: u32,
}

impl<'a> BlockHeader<'a> {
    pub fn version(&self) -> i32 {
        self.version
    }
    pub fn prev_blockhash(&self) -> &[u8] {
        &self.slice[4..36]
    }
    pub fn merkle_root(&self) -> &[u8] {
        &self.slice[36..68]
    }
    pub fn time(&self) -> u32 {
        self.time
    }
    pub fn nonce(&self) -> u32 {
        self.nonce
    }
    pub fn block_hash_preimage(&self) -> &[u8] {
        self.slice
    }
    #[cfg(feature = "hashes")]
    pub fn block_hash(&self) -> bitcoin_hashes::sha256d::Hash {
        use bitcoin_hashes::{sha256d, Hash, HashEngine};
        let mut engine = sha256d::Hash::engine();
        engine.input(self.block_hash_preimage());
        sha256d::Hash::from_engine(engine)
    }
}

impl<'a> AsRef<[u8]> for BlockHeader<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, BlockHeader<'a>> for BlockHeader<'a> {
    fn parse(slice: &'a [u8]) -> SResult<BlockHeader<'a>> {
        if slice.len() < 80 {
            return Err(Error::Needed(80 - slice.len()));
        }
        let version = read_i32(slice)?;
        let hashes = read_slice(version.remaining, 64)?;
        let time = read_u32(hashes.remaining)?;
        let bits = read_u32(time.remaining)?;
        let nonce = read_u32(bits.remaining)?;
        Ok(ParseResult::new(
            nonce.remaining,
            BlockHeader {
                slice: &slice[..80],
                version: version.parsed,
                time: time.parsed,
                bits: bits.parsed,
                nonce: nonce.parsed,
            },
            80,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::BlockHeader, test_common::GENESIS_BLOCK_HEADER, Parse};

    use hex_lit::hex;

    #[test]
    fn parse_block() {
        // genesis block
        let block_header = BlockHeader::parse(&GENESIS_BLOCK_HEADER).unwrap();

        assert_eq!(block_header.remaining, &[][..]);
        assert_eq!(
            block_header.parsed,
            BlockHeader {
                slice: &GENESIS_BLOCK_HEADER,
                version: 1,
                time: 1231006505,
                bits: 486604799,
                nonce: 2083236893
            }
        );
        assert_eq!(block_header.consumed, 80);

        assert_eq!(
            block_header.parsed.prev_blockhash(),
            hex!("0000000000000000000000000000000000000000000000000000000000000000")
        );
        assert_eq!(
            block_header.parsed.merkle_root(),
            hex!("3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a")
        );

        check_hash(
            &block_header.parsed,
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<BlockHeader>(), 32);
    }

    #[cfg(feature = "hashes")]
    fn check_hash(block: &BlockHeader, expected: &str) {
        assert_eq!(format!("{}", block.block_hash()), expected);
    }

    #[cfg(not(feature = "hashes"))]
    fn check_hash(_block: &BlockHeader, _expected: &str) {}
}

#[cfg(bench)]
mod bench {
    use crate::{bsl::BlockHeader, Parse};
    use bitcoin::consensus::deserialize;
    use test::{black_box, Bencher};

    #[bench]
    pub fn block_hash(bh: &mut Bencher) {
        let block_header = BlockHeader::parse(&crate::test_common::GENESIS_BLOCK_HEADER)
            .unwrap()
            .parsed;

        bh.iter(|| {
            let hash = block_header.block_hash();
            black_box(&hash);
        });
    }
    #[bench]
    pub fn block_hash_bitcoin(bh: &mut Bencher) {
        let block_header: bitcoin::BlockHeader =
            deserialize(&crate::test_common::GENESIS_BLOCK_HEADER).unwrap();

        bh.iter(|| {
            let hash = block_header.block_hash();
            black_box(&hash);
        });
    }
}
