use crate::{number::U32, slice::read_slice, Parse, ParseResult, SResult};

/// The out point of a transaction input, identifying the previous output being spent
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

impl<'a> Parse<'a> for OutPoint<'a> {
    /// Parse the out point from the given slice
    fn parse(slice: &'a [u8]) -> SResult<Self> {
        let txid = read_slice(slice, 32usize)?;
        let vout = U32::parse(txid.remaining())?;
        Ok(ParseResult::new(
            vout.remaining(),
            OutPoint {
                slice: &slice[..36],
                vout: vout.parsed().into(),
            },
        ))
    }
}
impl<'a> OutPoint<'a> {
    /// Returns the transaction txid of the previous output
    pub fn txid(&self) -> &[u8] {
        &self.slice[..32]
    }
    /// Returns the vout of the previous output
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
        assert_eq!(OutPoint::parse(&[1u8]), Err(Error::Needed(31)));
        assert_eq!(OutPoint::parse(&[0u8; 35]), Err(Error::Needed(1)));
        assert_eq!(
            OutPoint::parse(&[0u8; 36]),
            Ok(ParseResult::new_exact(expected.clone()))
        );
        assert_eq!(
            OutPoint::parse(&[0u8; 37]),
            Ok(ParseResult::new(&[0u8][..], expected,))
        );
        let vec: Vec<_> = (0..36).collect();
        let txid: Vec<_> = (0..32).collect();
        let out_point = OutPoint::parse(&vec[..]).unwrap();
        assert_eq!(out_point.parsed().txid(), &txid[..]);
    }
}
