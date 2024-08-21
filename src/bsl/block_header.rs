use core::ops::ControlFlow;

use crate::{
    number::{I32, U32},
    slice::read_slice,
    Parse, ParseResult, SResult, Visit, Visitor,
};

/// The block header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader<'a> {
    slice: &'a [u8],
    version: i32,
    time: u32,
    bits: u32,
    nonce: u32,
}

impl<'a> Visit<'a> for BlockHeader<'a> {
    fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Self> {
        let version = I32::parse(slice)?;
        let hashes = read_slice(version.remaining(), 64)?;
        let time = U32::parse(hashes.remaining())?;
        let bits = U32::parse(time.remaining())?;
        let nonce = U32::parse(bits.remaining())?;
        let header = BlockHeader {
            slice: &slice[..80],
            version: version.parsed().into(),
            time: time.parsed().into(),
            bits: bits.parsed().into(),
            nonce: nonce.parsed().into(),
        };
        if let ControlFlow::Break(_) = visit.visit_block_header(&header) {
            return Err(crate::Error::VisitBreak);
        }
        Ok(ParseResult::new(nonce.remaining(), header))
    }
}

impl<'a> BlockHeader<'a> {
    /// Returns the block header version.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Returns the hash of the previous block header.
    pub fn prev_blockhash(&self) -> &[u8] {
        &self.slice[4..36]
    }

    /// Returns the hash of the root of the merkle tree of the transactions in this block.
    pub fn merkle_root(&self) -> &[u8] {
        &self.slice[36..68]
    }

    /// Returns the UNIX timestamp of the block header.
    pub fn time(&self) -> u32 {
        self.time
    }

    /// Returns the nonce of this block header.
    pub fn nonce(&self) -> u32 {
        self.nonce
    }

    /// Returns the block hash preimage, the data that must be fed to the hash algorithm (double sha256)
    /// to get the block hash
    pub fn block_hash_preimage(&self) -> &[u8] {
        self.slice
    }

    /// Returns the hash of this block header
    #[cfg(feature = "bitcoin_hashes")]
    pub fn block_hash(&self) -> crate::bitcoin_hashes::sha256d::Hash {
        use crate::bitcoin_hashes::{sha256d, Hash, HashEngine};
        let mut engine = sha256d::Hash::engine();
        engine.input(self.block_hash_preimage());
        sha256d::Hash::from_engine(engine)
    }

    /// Calculate the block hash using the sha2 crate.
    /// NOTE: the result type is not displayed backwards when converted to string.
    #[cfg(feature = "sha2")]
    pub fn block_hash_sha2(
        &self,
    ) -> crate::sha2::digest::generic_array::GenericArray<u8, crate::sha2::digest::typenum::U32>
    {
        use crate::sha2::{Digest, Sha256};
        let first = Sha256::digest(self.block_hash_preimage());
        Sha256::digest(&first[..])
    }
}

impl<'a> AsRef<[u8]> for BlockHeader<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
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

        assert_eq!(block_header.remaining(), &[][..]);
        assert_eq!(
            block_header.parsed(),
            &BlockHeader {
                slice: &GENESIS_BLOCK_HEADER,
                version: 1,
                time: 1231006505,
                bits: 486604799,
                nonce: 2083236893
            }
        );
        assert_eq!(block_header.consumed(), 80);

        assert_eq!(
            block_header.parsed().prev_blockhash(),
            hex!("0000000000000000000000000000000000000000000000000000000000000000")
        );
        assert_eq!(
            block_header.parsed().merkle_root(),
            hex!("3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a")
        );

        check_hash(
            &block_header.parsed(),
            hex!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"),
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<BlockHeader>(), 32);
    }

    #[cfg(all(not(feature = "sha2"), not(feature = "bitcoin_hashes")))]
    fn check_hash(_block: &BlockHeader, _expected: [u8; 32]) {}

    #[cfg(all(not(feature = "sha2"), feature = "bitcoin_hashes"))]
    fn check_hash(block: &BlockHeader, expected: [u8; 32]) {
        use crate::test_common::reverse;
        assert_eq!(&block.block_hash()[..], &reverse(expected)[..]);
    }

    #[cfg(all(feature = "sha2", not(feature = "bitcoin_hashes")))]
    fn check_hash(block: &BlockHeader, expected: [u8; 32]) {
        use crate::test_common::reverse;
        assert_eq!(&block.block_hash_sha2()[..], &reverse(expected)[..]);
    }

    #[cfg(all(feature = "sha2", feature = "bitcoin_hashes"))]
    fn check_hash(block: &BlockHeader, expected: [u8; 32]) {
        use crate::test_common::reverse;
        assert_eq!(&block.block_hash()[..], &reverse(expected)[..]);
        assert_eq!(&block.block_hash_sha2()[..], &reverse(expected)[..]);
    }
}
