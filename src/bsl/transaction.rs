use crate::{
    bsl::{TxIn, TxIns, TxOut, TxOuts, Witnesses},
    number::{read_i32, read_u32, read_u8},
    Error, Parse, ParseResult, SResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction<'a> {
    slice: &'a [u8],
    inputs: TxIns<'a>,
    outputs: TxOuts<'a>,
    witnesses: Witnesses<'a>,
}

impl<'a> Transaction<'a> {
    pub fn version(&self) -> i32 {
        read_i32(&self.slice[..4])
            .expect("slice length granted during parsing")
            .parsed
    }

    pub fn locktime(&self) -> u32 {
        let from = self.slice.len() - 4; // slice length granted during parsing
        read_u32(&self.slice[from..])
            .expect("slice length granted during parsing")
            .parsed
    }

    pub fn total_inputs(&self) -> u64 {
        self.inputs.n()
    }

    pub fn inputs(&self) -> impl Iterator<Item = TxIn> {
        TxInIterator {
            slice: &self.inputs.first_tx_in(),
            n: self.inputs.n(),
        }
    }

    pub fn total_outputs(&self) -> u64 {
        self.outputs.n()
    }

    pub fn outputs(&self) -> impl Iterator<Item = TxOut> {
        TxOutIterator {
            slice: &self.outputs.first_tx_out(),
            n: self.outputs.n(),
        }
    }

    pub fn txid_preimage(&self) -> (&'a [u8], &'a [u8], &'a [u8]) {
        if self.witnesses.is_segwit() {
            let inputs_outputs_len = self.inputs.len() + self.outputs.len();
            (
                &self.slice[..4],                       // version
                &self.slice[6..inputs_outputs_len + 6], // input & outputs (but first skips segwit markers)
                &self.slice[self.len() - 4..],          // locktime
            )
        } else {
            (&self.slice, &[], &[])
        }
    }

    #[cfg(feature = "hashes")]
    pub fn txid(&self) -> bitcoin_hashes::sha256d::Hash {
        use bitcoin_hashes::{sha256d, Hash, HashEngine};
        let (a, b, c) = self.txid_preimage();
        let mut engine = sha256d::Hash::engine();
        engine.input(a);
        engine.input(b);
        engine.input(c);
        sha256d::Hash::from_engine(engine)
    }
}

impl<'a> AsRef<[u8]> for Transaction<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, Transaction<'a>> for Transaction<'a> {
    fn parse(slice: &'a [u8]) -> SResult<Transaction<'a>> {
        let version = read_i32(slice)?;
        let inputs = TxIns::parse(version.remaining)?;
        if inputs.parsed.is_empty() {
            let segwit_flag = read_u8(inputs.remaining)?;
            match segwit_flag.parsed {
                1 => {
                    let inputs = TxIns::parse(segwit_flag.remaining)?;
                    let outputs = TxOuts::parse(inputs.remaining)?;
                    let witnesses =
                        Witnesses::parse(outputs.remaining, inputs.parsed.n() as usize)?;

                    if !inputs.parsed.is_empty() && witnesses.parsed.all_empty() {
                        return Err(Error::SegwitFlagWithoutWitnesses);
                    }

                    let _locktime = read_u32(witnesses.remaining)?;
                    let consumed = 10 + inputs.consumed + outputs.consumed + witnesses.consumed;

                    return Ok(ParseResult::new(
                        &slice[consumed..],
                        Transaction {
                            slice: &slice[..consumed],
                            inputs: inputs.parsed,
                            outputs: outputs.parsed,
                            witnesses: witnesses.parsed,
                        },
                        consumed,
                    ));
                }
                x => return Err(Error::UnknownSegwitFlag(x)),
            }
        } else {
            let outputs = TxOuts::parse(inputs.remaining)?;
            let _locktime = read_u32(outputs.remaining)?;
            let consumed = inputs.consumed + outputs.consumed + 8;

            return Ok(ParseResult::new(
                &slice[consumed..],
                Transaction {
                    slice: &slice[..consumed],
                    inputs: inputs.parsed,
                    outputs: outputs.parsed,
                    witnesses: Witnesses::default(),
                },
                consumed,
            ));
        }
    }
}

pub struct TxInIterator<'a> {
    slice: &'a [u8],
    n: u64,
}
impl<'a> Iterator for TxInIterator<'a> {
    type Item = TxIn<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tx_in = TxIn::parse(self.slice).ok()?;
        self.slice = tx_in.remaining;
        Some(tx_in.parsed)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n as usize, Some(self.n as usize))
    }
}
impl<'a> ExactSizeIterator for TxInIterator<'a> {}

pub struct TxOutIterator<'a> {
    slice: &'a [u8],
    n: u64,
}
impl<'a> Iterator for TxOutIterator<'a> {
    type Item = TxOut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tx_out = TxOut::parse(self.slice).ok()?;
        self.slice = tx_out.remaining;
        Some(tx_out.parsed)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n as usize, Some(self.n as usize))
    }
}
impl<'a> ExactSizeIterator for TxOutIterator<'a> {}

#[cfg(test)]
mod test {
    use crate::{bsl::Transaction, test_common::GENESIS_TX, Parse, EMPTY};
    use hex_lit::hex;

    #[test]
    fn parse_genesis_transaction() {
        let tx = Transaction::parse(&GENESIS_TX[..]).unwrap();
        assert_eq!(tx.remaining, &EMPTY[..]);
        assert_eq!(tx.parsed.as_ref(), &GENESIS_TX[..]);
        assert_eq!(tx.consumed, 204);
        assert_eq!(tx.parsed.version(), 1);
        assert_eq!(tx.parsed.locktime(), 0);

        check_hash(
            &tx.parsed,
            "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        );

        let mut iter = tx.parsed.inputs();
        let first_input = iter.next().unwrap();
        assert_eq!(first_input.sequence(), 0xffffffff);
        assert!(iter.next().is_none());

        let mut iter = tx.parsed.outputs();
        let first_output = iter.next().unwrap();
        assert_eq!(first_output.value(), 5_000_000_000);
        assert_eq!(first_output.script_pubkey().script(), hex!("4104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn parse_segwit_transaction() {
        let segwit_tx = hex!("010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff3603da1b0e00045503bd5704c7dd8a0d0ced13bb5785010800000000000a636b706f6f6c122f4e696e6a61506f6f6c2f5345475749542fffffffff02b4e5a212000000001976a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac0000000000000000266a24aa21a9edf91c46b49eb8a29089980f02ee6b57e7d63d33b18b4fddac2bcd7db2a39837040120000000000000000000000000000000000000000000000000000000000000000000000000");
        let tx = Transaction::parse(&segwit_tx[..]).unwrap();
        assert_eq!(tx.remaining, &EMPTY[..]);
        assert_eq!(tx.parsed.as_ref(), &segwit_tx[..]);
        assert_eq!(tx.consumed, 222);
        assert_eq!(tx.parsed.version(), 1);
        assert_eq!(tx.parsed.locktime(), 0);

        check_hash(
            &tx.parsed,
            "4be105f158ea44aec57bf12c5817d073a712ab131df6f37786872cfc70734188", // testnet tx
        );

        let mut iter = tx.parsed.inputs();
        let first_input = iter.next().unwrap();
        assert_eq!(first_input.sequence(), 0xffffffff);
        assert!(iter.next().is_none());

        let mut iter = tx.parsed.outputs();
        let first_output = iter.next().unwrap();
        assert_eq!(first_output.value(), 312665524);
        assert_eq!(
            first_output.script_pubkey().script(),
            hex!("76a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac")
        );
        let second_output = iter.next().unwrap();
        assert_eq!(second_output.value(), 0);
        assert_eq!(
            second_output.script_pubkey().script(),
            hex!("6a24aa21a9edf91c46b49eb8a29089980f02ee6b57e7d63d33b18b4fddac2bcd7db2a3983704")
        );
        assert!(iter.next().is_none());
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Transaction>(), 120);
    }

    #[cfg(feature = "hashes")]
    fn check_hash(tx: &Transaction, expected: &str) {
        assert_eq!(format!("{}", tx.txid()), expected);
    }

    #[cfg(not(feature = "hashes"))]
    fn check_hash(_tx: &Transaction, _expected: &str) {}
}

#[cfg(bench)]
mod bench {
    use crate::{bsl::Transaction, Parse};
    use bitcoin::consensus::deserialize;
    use hex_lit::hex;
    use test::{black_box, Bencher};

    const BENCH_TX: [u8; 193] = hex!("0100000001a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff0100e1f505000000001976a9140389035a9225b3839e2bbf32d826a1e222031fd888ac00000000");

    #[bench]
    pub fn tx_deserialize(bh: &mut Bencher) {
        bh.iter(|| {
            let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed;
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

    #[cfg(feature = "hashes")]
    #[bench]
    pub fn txid(bh: &mut Bencher) {
        let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed;
        bh.iter(|| {
            black_box(&tx.txid());
        });
    }

    #[bench]
    pub fn txid_bitcoin(bh: &mut Bencher) {
        let tx: bitcoin::Transaction = deserialize(&BENCH_TX[..]).unwrap();
        bh.iter(|| {
            black_box(&tx.txid());
        });
    }

    #[bench]
    pub fn tx_iter_inputs(bh: &mut Bencher) {
        let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed;
        bh.iter(|| {
            for input in tx.inputs() {
                black_box(&input);
            }
        });
    }
    #[bench]
    pub fn tx_iter_inputs_bitcoin(bh: &mut Bencher) {
        let tx: bitcoin::Transaction = deserialize(&BENCH_TX[..]).unwrap();
        bh.iter(|| {
            for input in tx.input.iter() {
                black_box(&input);
            }
        });
    }

    #[bench]
    pub fn tx_iter_outputs(bh: &mut Bencher) {
        let tx = Transaction::parse(&BENCH_TX[..]).unwrap().parsed;
        bh.iter(|| {
            for output in tx.outputs() {
                black_box(&output);
            }
        });
    }
    #[bench]
    pub fn tx_iter_outputs_bitcoin(bh: &mut Bencher) {
        let tx: bitcoin::Transaction = deserialize(&BENCH_TX[..]).unwrap();
        bh.iter(|| {
            for output in tx.output.iter() {
                black_box(&output);
            }
        });
    }
}
