use crate::bsl::Script;
use crate::number::U64;
use crate::{Parse, ParseResult, SResult};

/// Contains a single transaction output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOut<'a> {
    slice: &'a [u8],
    value: u64,
    script_pubkey: Script<'a>,
}
impl<'a> Parse<'a> for TxOut<'a> {
    fn parse(slice: &'a [u8]) -> SResult<Self> {
        let value = U64::parse(slice)?;
        let script = Script::parse(value.remaining())?;
        let consumed = value.consumed() + script.consumed();
        let remaining = script.remaining();
        let tx_out = TxOut {
            slice: &slice[..consumed],
            value: value.parsed_owned().into(),
            script_pubkey: script.parsed_owned(),
        };
        Ok(ParseResult::new(remaining, tx_out))
    }
}
impl<'a> TxOut<'a> {
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
    use crate::{bsl::Script, bsl::TxOut, Parse, ParseResult};
    use hex_lit::hex;

    #[test]
    fn parse_tx_out() {
        let tx_out_bytes = hex!("ffffffffffffffff0100");

        let tx_out_expected = TxOut {
            slice: &tx_out_bytes[..],
            value: u64::MAX,
            script_pubkey: Script::parse(&hex!("0100")[..]).unwrap().parsed_owned(),
        };
        assert_eq!(
            TxOut::parse(&tx_out_bytes[..]),
            Ok(ParseResult::new_exact(tx_out_expected))
        );
    }
}
