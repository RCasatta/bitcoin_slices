use crate::{Error, ParseResult, SResult};

pub fn read_slice<'a>(slice: &'a [u8], len: usize) -> SResult<&'a [u8]> {
    if slice.len() < len {
        Err(Error::Needed(len - slice.len()))
    } else {
        let remaining = &slice[len..];
        let parsed = &slice[..len];

        Ok(ParseResult {
            remaining,
            parsed,
            consumed: len,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{Error, ParseResult};

    use super::read_slice as r;

    #[test]
    fn read_slice() {
        assert_eq!(r(&[], 0), Ok(ParseResult::new(&[][..], &[][..], 0)));
        assert_eq!(r(&[], 1), Err(Error::Needed(1)));
        assert_eq!(r(&[0u8], 1), Ok(ParseResult::new(&[][..], &[0u8][..], 1)));
        assert_eq!(
            r(&[0u8, 1u8], 1),
            Ok(ParseResult::new(&[1u8][..], &[0u8][..], 1))
        );
        assert_eq!(
            r(&[0u8, 1u8], 2),
            Ok(ParseResult::new(&[][..], &[0u8, 1u8][..], 2))
        );
    }
}
