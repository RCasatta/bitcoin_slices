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
        None => return Err(Error::Needed(1)),
    })
}

#[inline(always)]
pub fn parse_len_capped(slice: &[u8]) -> (u32, u8, u8) {
    match slice.first() {
        Some(0xFFu8) => (0, 0, 1),
        Some(0xFEu8) if slice.len() >= 5 => {
            let val = u32::from_le_bytes((&slice[1..5]).try_into().unwrap());
            // let non_minimal = if val >= u16::MAX as u32 { 0 } else { 1 };
            let non_minimal = 0u8;
            (val, 5, non_minimal)
        }
        Some(0xFDu8) if slice.len() >= 3 => {
            let val = u16::from_le_bytes((&slice[1..3]).try_into().unwrap()) as u32;
            // let non_minimal = if val >= 0xFD as u32 { 0 } else { 1 };
            let non_minimal = 0u8;
            (val, 3, non_minimal)
        }
        Some(x) => (*x as u32, 1, 0),
        _ => (0, 0, 1),
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
        assert_eq!(parse_len(&[]), Err(Error::Needed(1)));

        check(&[10u8], 1, 10);
        check(&[0xFCu8], 1, 0xFC);

        assert_eq!(parse_len(&[0xFDu8, 0xFC, 0]), Err(Error::NonMinimalVarInt));

        check(&[0xFDu8, 0xFD, 0x00], 3, 0xFD);
        check(&[0xFDu8, 0xFD, 0x33], 3, 0x33FD);
        check(&[0xFDu8, 0xFF, 0xF], 3, 0xFFF);
        check(&[10u8, 0u8], 1, 10);

        assert_eq!(parse_len(&[0xFDu8]), Err(Error::Needed(2)));
        assert_eq!(parse_len(&[0xFDu8, 0xFD]), Err(Error::Needed(1)));
        assert_eq!(parse_len(&[0xFEu8]), Err(Error::Needed(4)));
        check(&[0xFEu8, 0xF, 0xF, 0xF, 0xF], 5, 0xF0F0F0F);
        check(
            &[0xFFu8, 0xE0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0, 0],
            9,
            0xF0F0F0F0F0E0,
        );

        assert_eq!(parse_len(&[0xFFu8]), Err(Error::Needed(8)));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Len>(), 16);
    }

    // #[test]
    // fn test_non_minimal_u16() {
    //     println!("{:#b}", 0xFD);

    //     println!("{:#b}", 0xFE);
    //     println!("{:#b}", 0xFFu8);
    //     println!("");
    //     println!("{:#010b}", 0xFF & 0x03);
    //     println!("{:#010b}", 0xFE & 0x03);

    //     let non_minimal_expected = |val: u16| if val >= 0x000000FD { 0u16 } else { 1 };
    //     let non_minimal_new = |val: u16| {
    //         let val = val as u8;
    //         (!val & FF)
    //     };
    //     for i in 0..u16::MAX {
    //         let new = non_minimal_new(i);
    //         assert_eq!(
    //             non_minimal_expected(i) > 0,
    //             new > 0,
    //             "{i} {i:#x} {i:#b} - {new}"
    //         );
    //     }
    // }
}
