use core::num::NonZeroU32;

use crate::{
    bsl::{TxIns, TxOuts, Witnesses},
    number::{read_i32, read_u32, read_u8},
    EmptyVisitor, Error, ParseResult, SResult, Visitor,
};

/// A Bitcoin transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction<'a> {
    slice: &'a [u8],

    /// The length of the slice inlcuding all inputs and outputs of the transaction.
    /// If some the tx is segwit
    inputs_outputs_len: Option<NonZeroU32>,
}

impl<'a> Transaction<'a> {
    /// Parse the transaction in the slice
    pub fn parse(slice: &'a [u8]) -> SResult<Self> {
        Self::visit(slice, &mut EmptyVisitor {})
    }
    /// Visit the transaction in the slice
    pub fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Self> {
        let version = read_i32(slice)?;
        let inputs = TxIns::visit(version.remaining(), visit)?;
        if inputs.parsed().is_empty() {
            let segwit_flag = read_u8(inputs.remaining())?;
            let segwit_flag_u8 = segwit_flag.parsed().into();
            if segwit_flag_u8 == 1 {
                let inputs = TxIns::visit(segwit_flag.remaining(), visit)?;
                let outputs = TxOuts::visit(inputs.remaining(), visit)?;
                let witnesses =
                    Witnesses::visit(outputs.remaining(), inputs.parsed().n() as usize, visit)?;

                if !inputs.parsed().is_empty() && witnesses.parsed().all_empty() {
                    return Err(Error::SegwitFlagWithoutWitnesses);
                }

                let _locktime = read_u32(witnesses.remaining())?;
                let consumed = 10 + inputs.consumed() + outputs.consumed() + witnesses.consumed();
                let inputs_outputs_len =
                    inputs.parsed().as_ref().len() + outputs.parsed().as_ref().len();

                let tx = Transaction {
                    slice: &slice[..consumed],
                    inputs_outputs_len: NonZeroU32::new(inputs_outputs_len as u32), // inputs_outputs_len is at least 2 bytes if both empty, they contain the compact int len
                };
                visit.visit_transaction(&tx);
                Ok(ParseResult::new(&slice[consumed..], tx))
            } else {
                Err(Error::UnknownSegwitFlag(segwit_flag_u8))
            }
        } else {
            let outputs = TxOuts::visit(inputs.remaining(), visit)?;
            let _locktime = read_u32(outputs.remaining())?;
            let consumed = inputs.consumed() + outputs.consumed() + 8;

            let tx = Transaction {
                slice: &slice[..consumed],
                inputs_outputs_len: None,
            };
            visit.visit_transaction(&tx);
            Ok(ParseResult::new(&slice[consumed..], tx))
        }
    }

    /// Returns the transaction version.
    pub fn version(&self) -> i32 {
        read_i32(&self.slice[..4])
            .expect("slice length granted during parsing")
            .parsed_owned()
            .into()
    }

    /// Returns the transaction locktime.
    pub fn locktime(&self) -> u32 {
        let from = self.slice.len() - 4; // slice length granted during parsing
        read_u32(&self.slice[from..])
            .expect("slice length granted during parsing")
            .parsed_owned()
            .into()
    }

    /// Return the txid preimage, or the data that must be fed to the hashing function (double sha256)
    /// to obtain the txid.
    /// It is a tuple of 3 because for segwit transactions they are 3 non-contiguos bytes slices and
    /// we don't want to depend on standard and accept a `Write` parameter nor allocate.
    pub fn txid_preimage(&self) -> (&'a [u8], &'a [u8], &'a [u8]) {
        if let Some(len) = self.inputs_outputs_len.as_ref() {
            (
                &self.slice[..4],                       // version
                &self.slice[6..len.get() as usize + 6], // input & outputs (but first skips segwit markers, why bip143 didn't want to hash those?)
                &self.slice[self.as_ref().len() - 4..], // locktime
            )
        } else {
            (self.slice, &[], &[])
        }
    }

    /// Return the transaction identifier.
    /// If the transaction is legacy (non-segwit) this identifier could be malleated, meaning
    /// the same transaction effect could have different identifiers.
    #[cfg(feature = "bitcoin_hashes")]
    pub fn txid(&self) -> bitcoin_hashes::sha256d::Hash {
        use bitcoin_hashes::{sha256d, Hash, HashEngine};
        let (a, b, c) = self.txid_preimage();
        let mut engine = sha256d::Hash::engine();
        engine.input(a);
        engine.input(b);
        engine.input(c);
        sha256d::Hash::from_engine(engine)
    }

    /// Calculate the txid using the sha2 crate.
    /// NOTE: the result type is not displayed backwards when converted to string.
    #[cfg(feature = "sha2")]
    pub fn txid_sha2(
        &self,
    ) -> sha2::digest::generic_array::GenericArray<u8, sha2::digest::typenum::U32> {
        use sha2::{Digest, Sha256};
        let (a, b, c) = self.txid_preimage();
        let mut hasher = Sha256::new();
        hasher.update(a);
        hasher.update(b);
        hasher.update(c);
        let hash = hasher.finalize();
        Sha256::digest(&hash[..])
    }
}

impl<'a> AsRef<[u8]> for Transaction<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Transaction, test_common::GENESIS_TX};
    use hex_lit::hex;

    #[test]
    fn parse_genesis_transaction() {
        let tx = Transaction::parse(&GENESIS_TX[..]).unwrap();
        assert_eq!(tx.remaining(), &[][..]);
        assert_eq!(tx.parsed().as_ref(), &GENESIS_TX[..]);
        assert_eq!(tx.consumed(), 204);
        assert_eq!(tx.parsed().version(), 1);
        assert_eq!(tx.parsed().locktime(), 0);

        check_hash(
            &tx.parsed(),
            hex!("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"),
        );
    }

    #[test]
    fn parse_segwit_transaction() {
        let segwit_tx = hex!("010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff3603da1b0e00045503bd5704c7dd8a0d0ced13bb5785010800000000000a636b706f6f6c122f4e696e6a61506f6f6c2f5345475749542fffffffff02b4e5a212000000001976a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac0000000000000000266a24aa21a9edf91c46b49eb8a29089980f02ee6b57e7d63d33b18b4fddac2bcd7db2a39837040120000000000000000000000000000000000000000000000000000000000000000000000000");
        let tx = Transaction::parse(&segwit_tx[..]).unwrap();
        assert_eq!(tx.remaining(), &[]);
        assert_eq!(tx.parsed().as_ref(), &segwit_tx[..]);
        assert_eq!(tx.consumed(), 222);
        assert_eq!(tx.parsed().version(), 1);
        assert_eq!(tx.parsed().locktime(), 0);

        check_hash(
            &tx.parsed(),
            hex!("4be105f158ea44aec57bf12c5817d073a712ab131df6f37786872cfc70734188"), // testnet tx
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Transaction>(), 24);
    }

    #[cfg(all(not(feature = "sha2"), not(feature = "bitcoin_hashes")))]
    fn check_hash(_tx: &Transaction, _expected: [u8; 32]) {}

    #[cfg(all(not(feature = "sha2"), feature = "bitcoin_hashes"))]
    fn check_hash(tx: &Transaction, expected: [u8; 32]) {
        use crate::test_common::reverse;
        assert_eq!(&tx.txid()[..], &reverse(expected)[..]);
    }
    #[cfg(all(feature = "sha2", not(feature = "bitcoin_hashes")))]
    fn check_hash(tx: &Transaction, expected: [u8; 32]) {
        use crate::test_common::reverse;
        assert_eq!(&tx.txid_sha2()[..], &reverse(expected)[..]);
    }
    #[cfg(all(feature = "sha2", feature = "bitcoin_hashes"))]
    fn check_hash(tx: &Transaction, expected: [u8; 32]) {
        use crate::test_common::reverse;
        assert_eq!(&tx.txid()[..], &reverse(expected)[..]);
        assert_eq!(&tx.txid_sha2()[..], &reverse(expected)[..]);
    }
}

#[cfg(bench)]
mod bench {
    use crate::bsl::Transaction;
    use bitcoin::consensus::deserialize;
    use hex_lit::hex;
    use test::{black_box, Bencher};

    const BENCH_TX: [u8; 193] = hex!("0100000001a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff0100e1f505000000001976a9140389035a9225b3839e2bbf32d826a1e222031fd888ac00000000");

    #[bench]
    pub fn tx_deserialize(bh: &mut Bencher) {
        bh.iter(|| {
            let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed_owned();
            black_box(&tx);
        });
    }

    #[bench]
    pub fn tx_deserialize_bitcoin(bh: &mut Bencher) {
        bh.iter(|| {
            let tx: bitcoin::Transaction = deserialize(&BENCH_TX).unwrap();
            black_box(&tx);
        });
    }

    #[cfg(feature = "bitcoin_hashes")]
    #[bench]
    pub fn txid(bh: &mut Bencher) {
        let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed_owned();
        bh.iter(|| {
            black_box(&tx.txid());
        });
    }

    #[cfg(feature = "sha2")]
    #[bench]
    pub fn txid_sha2(bh: &mut Bencher) {
        let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed_owned();
        bh.iter(|| {
            black_box(&tx.txid_sha2());
        });
    }

    #[bench]
    pub fn txid_bitcoin(bh: &mut Bencher) {
        let tx: bitcoin::Transaction = deserialize(&BENCH_TX[..]).unwrap();
        bh.iter(|| {
            black_box(&tx.txid());
        });
    }
}
