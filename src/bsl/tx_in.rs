use crate::{
    bsl::{Len, OutPoint, Script},
    error::{to_unknown, to_unknown_if},
    number::read_u32,
    Parse, ParseResult, SResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxIn<'a> {
    slice: &'a [u8],
    prevout: OutPoint<'a>,
    script_sig: Script<'a>,
    sequence: u32,
}

impl<'a> TxIn<'a> {
    pub fn prevout(&self) -> &OutPoint {
        &self.prevout
    }
    pub fn script_sig(&self) -> &Script {
        &self.script_sig
    }
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
}

impl<'a> AsRef<[u8]> for TxIn<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, TxIn<'a>> for TxIn<'a> {
    fn parse(slice: &'a [u8]) -> SResult<TxIn<'a>> {
        let out_point = OutPoint::parse(slice).map_err(to_unknown)?;
        let script = Script::parse(out_point.remaining).map_err(to_unknown)?;
        let sequence = read_u32(script.remaining)?;
        let consumed = out_point.consumed + script.consumed + sequence.consumed;
        Ok(ParseResult::new(
            sequence.remaining,
            TxIn {
                slice: &slice[..consumed],
                prevout: out_point.parsed,
                script_sig: script.parsed,
                sequence: sequence.parsed,
            },
            consumed,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxIns<'a> {
    slice: &'a [u8],
    n: Len<'a>,
}

impl<'a> TxIns<'a> {
    pub fn n(&self) -> u64 {
        self.n.n()
    }
    pub fn first_tx_in(&self) -> &[u8] {
        &self.slice[self.n.len()..]
    }
    pub fn is_empty(&self) -> bool {
        self.n.n() == 0
    }
}

impl<'a> AsRef<[u8]> for TxIns<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, TxIns<'a>> for TxIns<'a> {
    fn parse(slice: &'a [u8]) -> SResult<TxIns<'a>> {
        let ParseResult {
            mut remaining,
            parsed,
            mut consumed,
        } = Len::parse(slice).map_err(to_unknown)?;

        for i in 1..=parsed.n() {
            let tx_in = TxIn::parse(remaining).map_err(|e| to_unknown_if(e, i != parsed.n()))?;
            remaining = tx_in.remaining;
            consumed += tx_in.consumed;
        }

        Ok(ParseResult::new(
            &slice[consumed..],
            TxIns {
                n: parsed,
                slice: &slice[..consumed],
            },
            consumed,
        ))
    }
}

#[cfg(test)]
mod test {
    use hex_lit::hex;

    use crate::{
        bsl::Script,
        bsl::TxIn,
        bsl::TxIns,
        bsl::{Len, OutPoint},
        Error, Parse, ParseResult,
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
        assert_eq!(tx_in_parsed.unwrap().parsed.len(), 42);

        assert_eq!(
            TxIn::parse(&tx_in_bytes[..tx_in_bytes.len() - 1]),
            Err(Error::Needed(1))
        );

        assert_eq!(TxIn::parse(&tx_in_bytes[..20]), Err(Error::UnknwonNeeded));
    }

    #[test]
    fn parse_tx_ins() {
        let tx_in_bytes = hex!("a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff");
        let mut tx_ins = vec![];
        tx_ins.push(2u8);
        tx_ins.extend(&tx_in_bytes);
        tx_ins.extend(&tx_in_bytes);
        let tx_ins_expected = TxIns {
            slice: &tx_ins[..],
            n: Len::new(&[2u8], 2),
        };
        assert_eq!(
            TxIns::parse(&tx_ins[..]),
            Ok(ParseResult::new_exact(tx_ins_expected))
        );

        assert_eq!(
            TxIns::parse(&[0u8][..]),
            Ok(ParseResult::new_exact(TxIns {
                slice: &[0u8][..],
                n: Len::new(&[0u8], 0)
            }))
        );

        assert_eq!(
            TxIns::parse(&tx_ins[..tx_ins.len() - 1]),
            Err(Error::Needed(1))
        );

        assert_eq!(TxIns::parse(&tx_ins[..22]), Err(Error::UnknwonNeeded));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<TxIns>(), 40);
    }
}
