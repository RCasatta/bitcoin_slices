use crate::bsl::Script;
use crate::{number::read_u64, ParseResult, SResult};

/// Contains a single transaction output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOut<'a> {
    slice: &'a [u8],
    value: u64,
    script_pubkey: Script<'a>,
}
impl<'a> TxOut<'a> {
    /// Parse the transaction output in this slice
    pub fn parse(slice: &'a [u8]) -> SResult<Self> {
        let value = read_u64(slice)?;
        let script = Script::parse(value.remaining)?;
        let consumed = value.consumed + script.consumed;
        let tx_out = TxOut {
            slice: &slice[..consumed],
            value: value.parsed,
            script_pubkey: script.parsed,
        };
        Ok(ParseResult::new(script.remaining, tx_out, consumed))
    }
    /// Return the amount of this output (satoshi)
    pub fn value(&self) -> u64 {
        self.value
    }
    /// Return the script pubkey of this output
    pub fn script_pubkey(&self) -> &[u8] {
        self.script_pubkey.script()
    }
}

impl<'a> AsRef<[u8]> for TxOut<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Script, bsl::TxOut, ParseResult};
    use hex_lit::hex;

    #[test]
    fn parse_tx_out() {
        let tx_out_bytes = hex!("ffffffffffffffff0100");

        let tx_out_expected = TxOut {
            slice: &tx_out_bytes[..],
            value: u64::MAX,
            script_pubkey: Script::parse(&hex!("0100")[..]).unwrap().parsed,
        };
        assert_eq!(
            TxOut::parse(&tx_out_bytes[..]),
            Ok(ParseResult::new_exact(tx_out_expected))
        );
    }
}
