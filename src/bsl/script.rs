use crate::{bsl::Len, slice::read_slice, ParseResult, SResult};

/// The Script, this type could be found in transaction outputs as `script_pubkey` or in transaction
/// inputs as `script_sig`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script<'a> {
    slice: &'a [u8],
    from: usize,
}

impl<'a> Script<'a> {
    /// Parse a script from the slice.
    pub fn parse(slice: &'a [u8]) -> SResult<Self> {
        let len = Len::parse(slice)?;
        Ok(
            read_slice(len.remaining(), len.parsed().n() as usize)?.map(|s| {
                ParseResult::new(
                    s.remaining(),
                    Script {
                        slice: &slice[..len.parsed().slice_len()],
                        from: len.parsed().as_ref().len(),
                    },
                )
            }),
        )
    }
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
    use crate::{bsl::Script, Error};

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
