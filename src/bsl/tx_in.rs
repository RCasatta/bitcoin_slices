use crate::{
    bsl::{OutPoint, Script},
    number::read_u32,
    ParseResult, SResult,
};

/// A transaction input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxIn<'a> {
    slice: &'a [u8],
    prevout: OutPoint<'a>,
    script_sig: Script<'a>,
    sequence: u32,
}

impl<'a> TxIn<'a> {
    /// Parse a transaction input from this slice
    pub fn parse(slice: &'a [u8]) -> SResult<Self> {
        let out_point = OutPoint::parse(slice)?;
        let script = Script::parse(out_point.remaining)?;
        let sequence = read_u32(script.remaining)?;
        let consumed = out_point.consumed + script.consumed + sequence.consumed;
        let tx_in = TxIn {
            slice: &slice[..consumed],
            prevout: out_point.parsed,
            script_sig: script.parsed,
            sequence: sequence.parsed,
        };
        Ok(ParseResult::new(sequence.remaining, tx_in, consumed))
    }
    /// Returns the previous output index spent by this transaction input
    pub fn prevout(&self) -> &OutPoint {
        &self.prevout
    }
    /// Return the script sig of this transaction input
    pub fn script_sig(&self) -> &[u8] {
        self.script_sig.script()
    }
    /// Returns the sequence of this transaction input
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
}

impl<'a> AsRef<[u8]> for TxIn<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(test)]
mod test {
    use hex_lit::hex;

    use crate::{
        bsl::{OutPoint, Script, TxIn},
        Error, ParseResult,
    };

    #[test]
    fn parse_tx_in() {
        let tx_in_bytes = hex!(
            "a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000000100ffffffff"
        );
        let out_point_bytes =
            hex!("a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece01000000");
        let script_bytes = hex!("0100");

        let tx_in_expected = TxIn {
            prevout: OutPoint::parse(&out_point_bytes[..]).unwrap().parsed,
            script_sig: Script::parse(&script_bytes[..]).unwrap().parsed,
            sequence: 4294967295u32,
            slice: &tx_in_bytes[..],
        };
        let tx_in_parsed = TxIn::parse(&tx_in_bytes[..]);

        assert_eq!(tx_in_parsed, Ok(ParseResult::new_exact(tx_in_expected)));
        assert_eq!(tx_in_parsed.unwrap().parsed.as_ref().len(), 42);

        assert_eq!(
            TxIn::parse(&tx_in_bytes[..tx_in_bytes.len() - 1]),
            Err(Error::Needed(1))
        );

        assert_eq!(TxIn::parse(&tx_in_bytes[..20]), Err(Error::Needed(12)));
    }
}
