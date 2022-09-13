use crate::{
    error::{to_unknown, to_unknown_if},
    slice::read_slice,
    Parse, ParseResult, SResult,
};

use super::Len;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness<'a> {
    slice: &'a [u8],
    from: usize,
    n: u64,
}

impl<'a> AsRef<[u8]> for Witness<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, Witness<'a>> for Witness<'a> {
    fn parse(slice: &'a [u8]) -> SResult<Witness<'a>> {
        let ParseResult {
            mut remaining,
            parsed,
            mut consumed,
        } = Len::parse(slice).map_err(to_unknown)?;

        for i in 1..=parsed.n() {
            let len = Len::parse(remaining).map_err(to_unknown)?;
            let sl = read_slice(len.remaining, len.parsed.n() as usize)
                .map_err(|e| to_unknown_if(e, i != parsed.n()))?;
            remaining = sl.remaining;
            consumed += len.parsed.slice_len();
        }

        Ok(ParseResult::new(
            &slice[consumed..],
            Witness {
                slice: &slice[..consumed],
                from: parsed.len(),
                n: parsed.n(),
            },
            consumed,
        ))
    }
}
impl<'a> Witness<'a> {
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Witnesses<'a> {
    slice: &'a [u8],
    all_empty: bool,
}
impl<'a> AsRef<[u8]> for Witnesses<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}
impl<'a> Witnesses<'a> {
    pub fn parse(slice: &'a [u8], n: usize) -> SResult<Witnesses<'a>> {
        let mut remaining = slice;
        let mut consumed = 0;
        let mut all_empty = true;
        for _ in 0..n {
            let witness = Witness::parse(remaining)?;
            remaining = witness.remaining;
            consumed += witness.consumed;
            if !witness.parsed.is_empty() {
                all_empty = false;
            }
        }
        Ok(ParseResult::new(
            &slice[consumed..],
            Witnesses {
                slice: &slice[..consumed],
                all_empty,
            },
            consumed,
        ))
    }

    pub fn all_empty(&self) -> bool {
        self.all_empty
    }

    pub fn is_segwit(&self) -> bool {
        self.slice != &[]
    }
}

#[cfg(test)]
mod test {
    use hex_lit::hex;

    use crate::{
        bsl::{Witness, Witnesses},
        Parse, ParseResult, EMPTY,
    };

    #[test]
    fn parse_witness() {
        let witness = hex!("0201000100");

        let expected = Witness {
            slice: &witness[..],
            from: 1,
            n: 2,
        };

        assert_eq!(
            Witness::parse(&witness[..]),
            Ok(ParseResult::new_exact(expected))
        );
    }

    #[test]
    fn parse_witnesses() {
        let witnesses_bytes = hex!("01000201000100");
        let witnesses = Witnesses::parse(&witnesses_bytes[..], 2).unwrap();
        assert_eq!(witnesses.remaining, &EMPTY[..]);
        assert_eq!(witnesses.parsed.as_ref(), &witnesses_bytes[..]);
        assert_eq!(witnesses.consumed, 7);
    }
}
