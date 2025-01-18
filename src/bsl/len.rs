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
#[deprecated = "use scan_len instead"]
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
/// Same as `parse_len` but mutates the `consumed` variable and returns only the value.
pub fn scan_len(slice: &[u8], consumed: &mut usize) -> Result<u64, Error> {
    match slice.first() {
        Some(0xFFu8) => {
            let bytes: [u8; 8] = slice
                .get(1..9)
                .ok_or(Error::MoreBytesNeeded)?
                .try_into()
                .expect("static bounds check");
            let n = u64::from_le_bytes(bytes);
            if n > u32::MAX as u64 {
                *consumed += 9;
                Ok(n)
            } else {
                Err(Error::NonMinimalVarInt)
            }
        }
        Some(0xFEu8) => {
            let bytes: [u8; 4] = slice
                .get(1..5)
                .ok_or(Error::MoreBytesNeeded)?
                .try_into()
                .expect("static bounds check");
            let n = u32::from_le_bytes(bytes) as u64;
            if n > u16::MAX as u64 {
                *consumed += 5;
                Ok(n)
            } else {
                Err(Error::NonMinimalVarInt)
            }
        }
        Some(0xFDu8) => {
            let bytes: [u8; 2] = slice
                .get(1..3)
                .ok_or(Error::MoreBytesNeeded)?
                .try_into()
                .expect("static bounds check");
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
    use super::scan_len;
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

    #[test]
    fn test_scan_len() {
        let mut consumed = 0;
        assert_eq!(scan_len(&[], &mut consumed), Err(Error::MoreBytesNeeded));

        consumed = 0;
        assert_eq!(scan_len(&[10u8], &mut consumed), Ok(10));
        assert_eq!(consumed, 1);

        consumed = 0;
        assert_eq!(scan_len(&[0xFCu8], &mut consumed), Ok(0xFC));
        assert_eq!(consumed, 1);

        consumed = 0;
        assert_eq!(
            scan_len(&[0xFDu8, 0xFC, 0], &mut consumed),
            Err(Error::NonMinimalVarInt)
        );

        consumed = 0;
        assert_eq!(scan_len(&[0xFDu8, 0xFD, 0x00], &mut consumed), Ok(0xFD));
        assert_eq!(consumed, 3);

        consumed = 0;
        assert_eq!(scan_len(&[0xFDu8, 0xFD, 0x33], &mut consumed), Ok(0x33FD));
        assert_eq!(consumed, 3);

        consumed = 0;
        assert_eq!(scan_len(&[0xFDu8, 0xFF, 0xF], &mut consumed), Ok(0xFFF));
        assert_eq!(consumed, 3);

        consumed = 0;
        assert_eq!(scan_len(&[10u8, 0u8], &mut consumed), Ok(10));
        assert_eq!(consumed, 1);

        consumed = 0;
        assert_eq!(
            scan_len(&[0xFDu8], &mut consumed),
            Err(Error::MoreBytesNeeded)
        );

        consumed = 0;
        assert_eq!(
            scan_len(&[0xFDu8, 0xFD], &mut consumed),
            Err(Error::MoreBytesNeeded)
        );

        consumed = 0;
        assert_eq!(
            scan_len(&[0xFEu8], &mut consumed),
            Err(Error::MoreBytesNeeded)
        );

        consumed = 0;
        assert_eq!(
            scan_len(&[0xFEu8, 0xF, 0xF, 0xF, 0xF], &mut consumed),
            Ok(0xF0F0F0F)
        );
        assert_eq!(consumed, 5);

        consumed = 0;
        assert_eq!(
            scan_len(
                &[0xFFu8, 0xE0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0, 0],
                &mut consumed
            ),
            Ok(0xF0F0F0F0F0E0)
        );
        assert_eq!(consumed, 9);
    }

    #[test]
    fn test_len_slice() {
        let slices = [
            &[0x01u8][..],                                               // small value
            &[0xFC][..],                                                 // max small value
            &[0xFD, 0xFD, 0x00][..],                                     // minimal FD encoding
            &[0xFD, 0xFF, 0xFF][..],                                     // max FD value
            &[0xFE, 0x00, 0x00, 0x01, 0x00][..],                         // minimal FE encoding
            &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF][..],                         // max FE value
            &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00][..], // minimal FF encoding
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF][..], // max FF value
            &[0xFE, 0x01, 0x02, 0x03, 0x04][..],                         // random asymmetry
            &[0xFE, 0x01, 0x02, 0x03, 0x04, 0x05][..], // slice longer than strictly needed
        ];
        let expected = [
            1u64,
            0xFC,
            0xFD,
            0xFFFF,
            0x00010000,
            0xFFFFFFFF,
            0x0000000100000000,
            0xFFFFFFFFFFFFFFFF,
            0x04030201,
            0x04030201,
        ];
        let expected_len = [1, 1, 3, 3, 5, 5, 9, 9, 5, 5];
        for (i, (s, (e, l))) in slices
            .iter()
            .zip(expected.into_iter().zip(expected_len.into_iter()))
            .enumerate()
        {
            let mut consumed = 0;
            assert_eq!(scan_len(&s[..], &mut consumed), Ok(e), "fails {i}");
            assert_eq!(consumed, l);

            assert_eq!(parse_len(&s[..]), Ok(Len { consumed: l, n: e }));
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Len>(), 16);
    }
}
