use crate::{
    number::{U16, U32, U64},
    visit::Parse,
    Error,
};

/// The bitcoin compact int encoding, up to 253 it consumes 1 byte, then there are markers for
/// `u16`, `u32` or `u64`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Len {
    pub(crate) consumed: usize,
    pub(crate) n: u64,
}

/// Parse `Len` from the slice.
/// This is done without `Parse` trait to have better perfomance.
#[inline(always)]
pub fn parse_len(slice: &[u8]) -> Result<Len, Error> {
    Ok(match slice.first() {
        Some(0xFFu8) => U64::parse(&slice[1..])?.parsed_owned().to_len()?,
        Some(0xFEu8) => U32::parse(&slice[1..])?.parsed_owned().to_len()?,
        Some(0xFDu8) => U16::parse(&slice[1..])?.parsed_owned().to_len()?,
        Some(x) => Len {
            n: *x as u64,
            consumed: 1,
        },
        None => return Err(Error::MoreBytesNeeded),
    })
}

#[inline(always)]
pub fn scan_len(slice: &[u8], consumed: &mut usize) -> Result<u64, Error> {
    match slice.first() {
        Some(0xFFu8) => {
            let bytes: [u8; 8] = slice[1..].try_into().map_err(|_| Error::MoreBytesNeeded)?;
            let n = u64::from_le_bytes(bytes);
            if n > u32::MAX as u64 {
                *consumed += 9;
                Ok(n)
            } else {
                Err(Error::NonMinimalVarInt)
            }
        }
        Some(0xFEu8) => {
            let bytes: [u8; 4] = slice[1..].try_into().map_err(|_| Error::MoreBytesNeeded)?;
            let n = u32::from_le_bytes(bytes) as u64;
            if n > u16::MAX as u64 {
                *consumed += 5;
                Ok(n)
            } else {
                Err(Error::NonMinimalVarInt)
            }
        }

        Some(0xFDu8) => {
            let bytes: [u8; 2] = slice[1..].try_into().map_err(|_| Error::MoreBytesNeeded)?;
            let n = u16::from_le_bytes(bytes) as u64;
            if n >= 0xFD {
                *consumed += 3;
                Ok(n)
            } else {
                Err(Error::NonMinimalVarInt)
            }
        }
        Some(x) => {
            *consumed += 1;
            Ok(*x as u64)
        }

        None => Err(Error::MoreBytesNeeded),
    }
}

impl Len {
    /// The value encoded in this compact int
    pub fn n(&self) -> u64 {
        self.n
    }

    /// consumed
    pub fn consumed(&self) -> usize {
        self.consumed
    }

    /// slice_len
    pub fn slice_len(&self) -> usize {
        self.consumed + self.n as usize
    }
}

#[cfg(test)]
mod test {
    use crate::{
        bsl::{len::parse_len, Len},
        Error,
    };

    fn check(slice: &[u8], consumed: usize, n: u64) {
        assert_eq!(parse_len(slice), Ok(Len { consumed, n }));
    }

    #[test]
    fn test_parse_len() {
        assert_eq!(parse_len(&[]), Err(Error::MoreBytesNeeded));

        check(&[10u8], 1, 10);
        check(&[0xFCu8], 1, 0xFC);

        assert_eq!(parse_len(&[0xFDu8, 0xFC, 0]), Err(Error::NonMinimalVarInt));

        check(&[0xFDu8, 0xFD, 0x00], 3, 0xFD);
        check(&[0xFDu8, 0xFD, 0x33], 3, 0x33FD);
        check(&[0xFDu8, 0xFF, 0xF], 3, 0xFFF);
        check(&[10u8, 0u8], 1, 10);

        assert_eq!(parse_len(&[0xFDu8]), Err(Error::MoreBytesNeeded));
        assert_eq!(parse_len(&[0xFDu8, 0xFD]), Err(Error::MoreBytesNeeded));
        assert_eq!(parse_len(&[0xFEu8]), Err(Error::MoreBytesNeeded));
        check(&[0xFEu8, 0xF, 0xF, 0xF, 0xF], 5, 0xF0F0F0F);
        check(
            &[0xFFu8, 0xE0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0, 0],
            9,
            0xF0F0F0F0F0E0,
        );

        assert_eq!(parse_len(&[0xFFu8]), Err(Error::MoreBytesNeeded));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Len>(), 16);
    }
}
