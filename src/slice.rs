use crate::{Error, ParseResult, SResult};

/// Return a slice legnth `len` from `from` if it's long enough, error otherwise.
#[inline(always)]
#[deprecated]
pub fn read_slice(from: &[u8], len: usize) -> SResult<&[u8]> {
    if from.len() < len {
        Err(Error::MoreBytesNeeded)
    } else {
        let (parsed, remaining) = from.split_at(len);

        Ok(ParseResult::new(remaining, parsed))
    }
}

#[cfg(test)]
mod test {
    use crate::{Error, ParseResult};

    #[allow(deprecated)]
    use super::read_slice as r;

    #[allow(deprecated)]
    #[test]
    fn read_slice() {
        assert_eq!(r(&[], 0), Ok(ParseResult::new(&[][..], &[][..])));
        assert_eq!(r(&[], 1), Err(Error::MoreBytesNeeded));
        assert_eq!(r(&[0u8], 1), Ok(ParseResult::new(&[][..], &[0u8][..])));
        assert_eq!(
            r(&[0u8, 1u8], 1),
            Ok(ParseResult::new(&[1u8][..], &[0u8][..]))
        );
        assert_eq!(
            r(&[0u8, 1u8], 2),
            Ok(ParseResult::new(&[][..], &[0u8, 1u8][..]))
        );
    }
}
