use core::convert::TryInto;

use crate::{slice::read_slice, ParseResult, SResult};

pub fn read_u8(slice: &[u8]) -> SResult<u8> {
    let p = read_slice(slice, 1)?;
    Ok(ParseResult::new(p.remaining, p.parsed[0], p.consumed))
}

pub fn read_u16(slice: &[u8]) -> SResult<u16> {
    let p = read_slice(slice, 2)?;
    Ok(ParseResult::new(
        p.remaining,
        u16::from_le_bytes(p.parsed.try_into().expect("slice length is 2")),
        p.consumed,
    ))
}

pub fn read_u32(slice: &[u8]) -> SResult<u32> {
    let p = read_slice(slice, 4)?;
    Ok(ParseResult::new(
        p.remaining,
        u32::from_le_bytes(p.parsed.try_into().expect("slice length is 4")),
        p.consumed,
    ))
}

pub fn read_u64(slice: &[u8]) -> SResult<u64> {
    let p = read_slice(slice, 8)?;
    Ok(ParseResult::new(
        p.remaining,
        u64::from_le_bytes(p.parsed.try_into().expect("slice length is 8")),
        p.consumed,
    ))
}

pub fn read_i32(slice: &[u8]) -> SResult<i32> {
    let p = read_slice(slice, 4)?;
    Ok(ParseResult::new(
        p.remaining,
        i32::from_le_bytes(p.parsed.try_into().expect("slice length is 4")),
        p.consumed,
    ))
}

#[cfg(test)]
mod test {
    use crate::{
        number::{read_i32, read_u16, read_u32, read_u64},
        ParseResult,
    };

    use super::read_u8;

    #[test]
    fn numbers() {
        assert_eq!(read_u8(&[0u8][..]), Ok(ParseResult::new(&[][..], 0u8, 1)));
        assert_eq!(read_u8(&[1u8][..]), Ok(ParseResult::new(&[][..], 1u8, 1)));
        assert_eq!(
            read_u8(&[255u8][..]),
            Ok(ParseResult::new(&[][..], 255u8, 1))
        );
        assert_eq!(
            read_u8(&[1u8, 2][..]),
            Ok(ParseResult::new(&[2u8][..], 1u8, 1))
        );

        assert_eq!(
            read_u16(&[1u8, 0][..]),
            Ok(ParseResult::new(&[][..], 1u16, 2))
        );
        assert_eq!(
            read_u16(&[0u8, 1][..]),
            Ok(ParseResult::new(&[][..], 256u16, 2))
        );
        assert_eq!(
            read_u16(&[136u8, 19][..]),
            Ok(ParseResult::new(&[][..], 5000u16, 2))
        );
        assert_eq!(
            read_u16(&[136u8, 19, 2][..]),
            Ok(ParseResult::new(&[2u8][..], 5000u16, 2))
        );

        assert_eq!(
            read_u32(&[1u8, 0, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 1u32, 4))
        );
        assert_eq!(
            read_u32(&[0u8, 1, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 256u32, 4))
        );
        assert_eq!(
            read_u32(&[136u8, 19, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 5000u32, 4))
        );
        assert_eq!(
            read_u32(&[32u8, 161, 7, 0][..]),
            Ok(ParseResult::new(&[][..], 500000u32, 4))
        );
        assert_eq!(
            read_u32(&[10u8, 10, 10, 10][..]),
            Ok(ParseResult::new(&[][..], 168430090u32, 4))
        );

        assert_eq!(
            read_i32(&[255u8, 255, 255, 255][..]),
            Ok(ParseResult::new(&[][..], -1i32, 4))
        );
        assert_eq!(
            read_i32(&[0u8, 255, 255, 255][..]),
            Ok(ParseResult::new(&[][..], -256i32, 4))
        );
        assert_eq!(
            read_i32(&[120u8, 236, 255, 255][..]),
            Ok(ParseResult::new(&[][..], -5000i32, 4))
        );
        assert_eq!(
            read_i32(&[32u8, 161, 7, 0][..]),
            Ok(ParseResult::new(&[][..], 500000i32, 4))
        );

        assert_eq!(
            read_u64(&[1u8, 0, 0, 0, 0, 0, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 1u64, 8))
        );
        assert_eq!(
            read_u64(&[10u8, 10, 10, 10, 10, 10, 10, 10][..]),
            Ok(ParseResult::new(&[][..], 723401728380766730u64, 8))
        );
    }
}
