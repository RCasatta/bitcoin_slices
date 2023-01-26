use crate::{slice::read_slice, Parse, ParseResult, SResult};

use super::len::{parse_len, Len};

/// The Script, this type could be found in transaction outputs as `script_pubkey` or in transaction
/// inputs as `script_sig`.
///
/// Note the slice returned with `as_ref()` contains the initial compact int, use [`Script::script()`]
/// to have only the script bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script<'a> {
    slice: &'a [u8],
    from: usize,
}

impl<'a> Parse<'a> for Script<'a> {
    /// Parse a script from the slice.
    fn parse(slice: &'a [u8]) -> SResult<Self> {
        let Len { consumed, n } = parse_len(slice)?;
        let n = n as usize;
        let remaining = &slice[consumed..];
        Ok(read_slice(remaining, n)?.map(|s| {
            ParseResult::new(
                s.remaining(),
                Script {
                    slice: &slice[..consumed + n],
                    from: consumed,
                },
            )
        }))
    }
}
impl<'a> Script<'a> {
    /// return the script bytes (exclude the compact int representing the length)
    pub fn script(&self) -> &[u8] {
        &self.slice[self.from..]
    }
}

impl<'a> AsRef<[u8]> for Script<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Script, Error, Parse};

    fn check(slice: &[u8], script_slice: &[u8]) {
        let script = Script::parse(slice);
        assert!(script.is_ok());
        let p = script.unwrap();
        assert_eq!(p.remaining(), &[]);
        assert_eq!(p.parsed().script(), script_slice);
    }

    #[test]
    fn parse_script() {
        check(&[1u8, 1], &[1u8]);
        check(&[1u8, 11], &[11u8]);
        check(&[3u8, 0, 1, 2], &[0u8, 1, 2]);

        assert_eq!(Script::parse(&[1u8]), Err(Error::Needed(1)));
        assert_eq!(Script::parse(&[100u8]), Err(Error::Needed(100)));
    }
}
