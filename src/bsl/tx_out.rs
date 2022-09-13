use crate::error::to_unknown_if;
use crate::{error::to_unknown, number::read_u64, Parse, ParseResult, SResult};

use crate::bsl::{Len, Script};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOut<'a> {
    slice: &'a [u8],
    value: u64,
    script_pubkey: Script<'a>,
}
impl<'a> TxOut<'a> {
    pub fn value(&self) -> u64 {
        self.value
    }
    pub fn script_pubkey(&self) -> &Script {
        &self.script_pubkey
    }
}

impl<'a> AsRef<[u8]> for TxOut<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, TxOut<'a>> for TxOut<'a> {
    fn parse(slice: &'a [u8]) -> SResult<TxOut<'a>> {
        let value = read_u64(slice)?;
        let script = Script::parse(value.remaining)?;
        let consumed = value.consumed + script.consumed;
        Ok(ParseResult::new(
            script.remaining,
            TxOut {
                slice: &slice[..consumed],
                value: value.parsed,
                script_pubkey: script.parsed,
            },
            consumed,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOuts<'a> {
    slice: &'a [u8],
    n: Len<'a>,
}

impl<'a> TxOuts<'a> {
    pub fn n(&self) -> u64 {
        self.n.n()
    }
    pub fn first_tx_out(&self) -> &[u8] {
        &self.slice[self.n.len()..]
    }
    pub fn is_empty(&self) -> bool {
        self.n.n() == 0
    }
}

impl<'a> AsRef<[u8]> for TxOuts<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, TxOuts<'a>> for TxOuts<'a> {
    fn parse(slice: &'a [u8]) -> SResult<TxOuts<'a>> {
        let ParseResult {
            mut remaining,
            parsed,
            mut consumed,
        } = Len::parse(slice).map_err(to_unknown)?;

        for i in 1..=parsed.n() {
            let tx_out = TxOut::parse(remaining).map_err(|e| to_unknown_if(e, i != parsed.n()))?;
            remaining = tx_out.remaining;
            consumed += tx_out.consumed;
        }
        Ok(ParseResult::new(
            &slice[consumed..],
            TxOuts {
                slice: &slice[..consumed],
                n: parsed,
            },
            consumed,
        ))
    }
}

#[cfg(test)]
mod test {
    use hex_lit::hex;

    use crate::{
        bsl::TxOut,
        bsl::TxOuts,
        bsl::{Len, Script},
        Error, Parse, ParseResult,
    };

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

    #[test]
    fn parse_tx_outs() {
        let tx_out_bytes = hex!("ffffffffffffffff0100");
        let mut tx_outs = vec![];
        tx_outs.push(2u8);
        tx_outs.extend(&tx_out_bytes);
        tx_outs.extend(&tx_out_bytes);
        let tx_outs_expected = TxOuts {
            slice: &tx_outs[..],
            n: Len::new(&[2u8], 2),
        };
        assert_eq!(
            TxOuts::parse(&tx_outs[..]),
            Ok(ParseResult::new_exact(tx_outs_expected))
        );

        assert_eq!(TxOuts::parse(&tx_outs[8..]), Err(Error::UnknwonNeeded));
        assert_eq!(
            TxOuts::parse(&tx_outs[..tx_outs.len() - 1]),
            Err(Error::Needed(1))
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<TxOuts>(), 40);
    }
}
