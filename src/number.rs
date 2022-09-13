use core::convert::TryInto;

use crate::{slice::read_slice, ParseResult, SResult};

pub fn read_u8<'a>(slice: &'a [u8]) -> SResult<u8> {
    let p = read_slice(slice, 1)?;
    Ok(ParseResult::new(p.remaining, p.parsed[0], p.consumed))
}

pub fn read_u16<'a>(slice: &'a [u8]) -> SResult<u16> {
    let p = read_slice(slice, 2)?;
    Ok(ParseResult::new(
        p.remaining,
        u16::from_le_bytes(p.parsed.try_into().expect("slice length is 2")),
        p.consumed,
    ))
}

pub fn read_u32<'a>(slice: &'a [u8]) -> SResult<u32> {
    let p = read_slice(slice, 4)?;
    Ok(ParseResult::new(
        p.remaining,
        u32::from_le_bytes(p.parsed.try_into().expect("slice length is 4")),
        p.consumed,
    ))
}

pub fn read_u64<'a>(slice: &'a [u8]) -> SResult<u64> {
    let p = read_slice(slice, 8)?;
    Ok(ParseResult::new(
        p.remaining,
        u64::from_le_bytes(p.parsed.try_into().expect("slice length is 8")),
        p.consumed,
    ))
}

pub fn read_i32<'a>(slice: &'a [u8]) -> SResult<i32> {
    let p = read_slice(slice, 4)?;
    Ok(ParseResult::new(
        p.remaining,
        i32::from_le_bytes(p.parsed.try_into().expect("slice length is 4")),
        p.consumed,
    ))
}

#[cfg(test)]
mod test {
    use crate::{number::read_u16, ParseResult};

    use super::read_u8;

    #[test]
    fn numbers() {
        assert_eq!(read_u8(&[0u8][..]), Ok(ParseResult::new(&[][..], 0u8, 1)));
        assert_eq!(read_u8(&[1u8][..]), Ok(ParseResult::new(&[][..], 1u8, 1)));
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

        // assert_eq!(serialize(&1u8), vec![1u8]);
        // assert_eq!(serialize(&0u8), vec![0u8]);
        // assert_eq!(serialize(&255u8), vec![255u8]);
        // // u16
        // assert_eq!(serialize(&1u16), vec![1u8, 0]);
        // assert_eq!(serialize(&256u16), vec![0u8, 1]);
        // assert_eq!(serialize(&5000u16), vec![136u8, 19]);
        // // u32
        // assert_eq!(serialize(&1u32), vec![1u8, 0, 0, 0]);
        // assert_eq!(serialize(&256u32), vec![0u8, 1, 0, 0]);
        // assert_eq!(serialize(&5000u32), vec![136u8, 19, 0, 0]);
        // assert_eq!(serialize(&500000u32), vec![32u8, 161, 7, 0]);
        // assert_eq!(serialize(&168430090u32), vec![10u8, 10, 10, 10]);
        // // i32
        // assert_eq!(serialize(&-1i32), vec![255u8, 255, 255, 255]);
        // assert_eq!(serialize(&-256i32), vec![0u8, 255, 255, 255]);
        // assert_eq!(serialize(&-5000i32), vec![120u8, 236, 255, 255]);
        // assert_eq!(serialize(&-500000i32), vec![224u8, 94, 248, 255]);
        // assert_eq!(serialize(&-168430090i32), vec![246u8, 245, 245, 245]);
        // assert_eq!(serialize(&1i32), vec![1u8, 0, 0, 0]);
        // assert_eq!(serialize(&256i32), vec![0u8, 1, 0, 0]);
        // assert_eq!(serialize(&5000i32), vec![136u8, 19, 0, 0]);
        // assert_eq!(serialize(&500000i32), vec![32u8, 161, 7, 0]);
        // assert_eq!(serialize(&168430090i32), vec![10u8, 10, 10, 10]);
        // // u64
        // assert_eq!(serialize(&1u64), vec![1u8, 0, 0, 0, 0, 0, 0, 0]);
        // assert_eq!(serialize(&256u64), vec![0u8, 1, 0, 0, 0, 0, 0, 0]);
        // assert_eq!(serialize(&5000u64), vec![136u8, 19, 0, 0, 0, 0, 0, 0]);
        // assert_eq!(serialize(&500000u64), vec![32u8, 161, 7, 0, 0, 0, 0, 0]);
        // assert_eq!(serialize(&723401728380766730u64), vec![10u8, 10, 10, 10, 10, 10, 10, 10]);
    }
}
