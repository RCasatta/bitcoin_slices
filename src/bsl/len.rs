use crate::{
    number::{read_u16, read_u32, read_u64, read_u8},
    ParseResult, SResult,
};

/// The bitcoin compact int encoding, up to 253 it consumes 1 byte, then there are markers for
/// `u16`, `u32` and `u64`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Len<'a> {
    slice: &'a [u8],
    n: u64,
}

impl<'a> Len<'a> {
    /// Try to parse a compact int from the `slice`
    pub fn parse(slice: &[u8]) -> SResult<Len> {
        let p = read_u8(slice)?;
        let (n, consumed) = match p.parsed().into() {
            0xFFu8 => read_u64(p.remaining())?.map(|p| (u64::from(p.parsed()), 9)),
            0xFEu8 => read_u32(p.remaining())?.map(|p| (u32::from(p.parsed()) as u64, 5)),
            0xFDu8 => read_u16(p.remaining())?.map(|p| (u16::from(p.parsed()) as u64, 3)),
            x => (x as u64, 1),
        };
        let (slice, remaining) = slice.split_at(consumed);
        let len = Len { slice, n };
        Ok(ParseResult::new(remaining, len))
    }

    /// The value encoded in this compact int
    pub fn n(&self) -> u64 {
        self.n
    }

    /// the len of self serialized plus the value contained, which is equivalent to the space
    /// needed to serialize a `Vec<u8>` (lenght + content)
    pub fn slice_len(&self) -> usize {
        self.n as usize + self.as_ref().len()
    }
}

impl<'a> AsRef<[u8]> for Len<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(test)]
impl<'a> Len<'a> {
    pub(crate) fn new(slice: &[u8], n: u64) -> Len {
        Len { n, slice }
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Len, Error, ParseResult};

    fn check(slice: &[u8], n: u64) {
        assert_eq!(
            Len::parse(slice),
            Ok(ParseResult::new_exact(Len { slice, n }))
        );
    }

    #[test]
    fn parse_len() {
        assert_eq!(Len::parse(&[]), Err(Error::Needed(1)));

        check(&[10u8], 10);
        check(&[0xFCu8], 0xFC);
        check(&[0xFDu8, 0xFD, 0], 0xFD);
        check(&[0xFDu8, 0xFF, 0xF], 0xFFF);

        assert_eq!(
            Len::parse(&[10u8, 0u8]),
            Ok(ParseResult::new(
                &[0u8][..],
                Len {
                    slice: &[10u8],
                    n: 10
                },
            ))
        );
        assert_eq!(Len::parse(&[0xFDu8]), Err(Error::Needed(2)));
        assert_eq!(Len::parse(&[0xFDu8, 0xFD]), Err(Error::Needed(1)));
        assert_eq!(Len::parse(&[0xFEu8]), Err(Error::Needed(4)));
        check(&[0xFEu8, 0xF, 0xF, 0xF, 0xF], 0xF0F0F0F);
        check(
            &[0xFFu8, 0xE0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0, 0],
            0xF0F0F0F0F0E0,
        );

        assert_eq!(Len::parse(&[0xFFu8]), Err(Error::Needed(8)));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Len>(), 24);
    }
}
