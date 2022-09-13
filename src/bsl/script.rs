use crate::{bsl::Len, error::to_unknown, slice::read_slice, Parse, ParseResult, SResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script<'a> {
    slice: &'a [u8],
    from: usize,
}

impl<'a> Script<'a> {
    pub fn script(&self) -> &[u8] {
        &self.slice[self.from..]
    }
}

impl<'a> AsRef<[u8]> for Script<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a, Script<'a>> for Script<'a> {
    fn parse(slice: &'a [u8]) -> SResult<Script<'a>> {
        let len = Len::parse(slice).map_err(to_unknown)?;
        Ok(
            read_slice(len.remaining, len.parsed.n() as usize)?.map(|s| ParseResult {
                remaining: s.remaining,
                parsed: Script {
                    slice: &slice[..len.parsed.slice_len()],
                    from: len.parsed.len(),
                },
                consumed: len.consumed + s.consumed,
            }),
        )
    }
}

#[cfg(test)]

mod test {
    use crate::{bsl::Script, Error, Parse};

    fn check(slice: &[u8], script_slice: &[u8]) {
        let script = Script::parse(slice);
        assert!(script.is_ok());
        let p = script.unwrap();
        assert_eq!(p.remaining, &[]);
        assert_eq!(p.parsed.script(), script_slice);
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
