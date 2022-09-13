use crate::{number::read_u32, slice::read_slice, Error, Parse, ParseResult, SResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutPoint<'a> {
    slice: &'a [u8],
    vout: u32,
}

impl<'a> AsRef<[u8]> for OutPoint<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, OutPoint<'a>> for OutPoint<'a> {
    fn parse(slice: &'a [u8]) -> SResult<OutPoint<'a>> {
        let txid = read_slice(slice, 32usize).map_err(|_| Error::Needed(36 - slice.len()))?;
        let vout = read_u32(txid.remaining)?;
        Ok(ParseResult::new(
            vout.remaining,
            OutPoint {
                slice: &slice[..36],
                vout: vout.parsed,
            },
            36,
        ))
    }
}

impl<'a> OutPoint<'a> {
    pub fn txid(&self) -> &[u8] {
        &self.slice[..32]
    }
    pub fn vout(&self) -> u32 {
        self.vout
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::OutPoint, Error, Parse, ParseResult};

    #[test]
    fn parse_out_point() {
        let expected = OutPoint {
            slice: &[0u8; 36],
            vout: 0,
        };
        assert_eq!(OutPoint::parse(&[1u8]), Err(Error::Needed(35)));
        assert_eq!(OutPoint::parse(&[0u8; 35]), Err(Error::Needed(1)));
        assert_eq!(
            OutPoint::parse(&[0u8; 36]),
            Ok(ParseResult::new_exact(expected.clone()))
        );
        assert_eq!(
            OutPoint::parse(&[0u8; 37]),
            Ok(ParseResult {
                remaining: &[0u8][..],
                parsed: expected,
                consumed: 36
            })
        );
        let vec: Vec<_> = (0..36).collect();
        let txid: Vec<_> = (0..32).collect();
        let out_point = OutPoint::parse(&vec[..]).unwrap();
        assert_eq!(out_point.parsed.txid(), &txid[..]);
    }
}
