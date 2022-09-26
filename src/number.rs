//! Contains methods to read numbers (u8,u16,u32,u64,i32) from slices

use core::convert::TryInto;

use crate::{slice::read_slice, ParseResult, SResult};

///
#[derive(Debug, PartialEq, Eq)]
pub struct U8([u8; 1]);

impl AsRef<[u8]> for U8 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<U8> for u8 {
    fn from(u: U8) -> Self {
        u.0[0]
    }
}

impl From<&U8> for u8 {
    fn from(u: &U8) -> Self {
        u.0[0]
    }
}

impl From<u8> for U8 {
    fn from(u: u8) -> Self {
        U8([u])
    }
}

/// Read an u8 from the slice
pub fn read_u8(slice: &[u8]) -> SResult<U8> {
    let p = read_slice(slice, 1)?;
    Ok(ParseResult::new(p.remaining(), U8([p.parsed()[0]])))
}

/// Read an u16 from the slice
pub fn read_u16(slice: &[u8]) -> SResult<U16> {
    let p = read_slice(slice, 2)?;
    Ok(ParseResult::new(
        p.remaining(),
        U16((*p.parsed()).try_into().expect("slice length is 2")),
    ))
}

/// Read an u32 from the slice
pub fn read_u32(slice: &[u8]) -> SResult<U32> {
    let p = read_slice(slice, 4)?;
    Ok(ParseResult::new(
        p.remaining(),
        U32((*p.parsed()).try_into().expect("slice length is 4")),
    ))
}

/// Read an u64 from the slice
pub fn read_u64(slice: &[u8]) -> SResult<U64> {
    let p = read_slice(slice, 8)?;
    Ok(ParseResult::new(
        p.remaining(),
        U64((*p.parsed()).try_into().expect("slice length is 8")),
    ))
}

/// Read an i32 from the slice
pub fn read_i32(slice: &[u8]) -> SResult<I32> {
    let p = read_slice(slice, 4)?;
    Ok(ParseResult::new(
        p.remaining(),
        I32((*p.parsed()).try_into().expect("slice length is 4")),
    ))
}

macro_rules! impl_number {
    ($primitive:ty, $newtype:ident, $size:expr) => {
        ///
        #[derive(Debug, PartialEq, Eq)]
        pub struct $newtype([u8; $size]);

        impl From<$newtype> for $primitive {
            fn from(u: $newtype) -> Self {
                <$primitive>::from_le_bytes(u.0)
            }
        }

        impl From<&$newtype> for $primitive {
            fn from(u: &$newtype) -> Self {
                <$primitive>::from_le_bytes(u.0)
            }
        }

        impl From<$primitive> for $newtype {
            fn from(u: $primitive) -> Self {
                $newtype(u.to_le_bytes())
            }
        }

        impl AsRef<[u8]> for $newtype {
            fn as_ref(&self) -> &[u8] {
                &self.0[..]
            }
        }
    };
}
impl_number!(u16, U16, 2);
impl_number!(u32, U32, 4);
impl_number!(i32, I32, 4);
impl_number!(u64, U64, 8);

#[cfg(test)]
mod test {
    use crate::ParseResult;

    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(
            read_u8(&[0u8][..]),
            Ok(ParseResult::new(&[][..], 0u8.into()))
        );
        assert_eq!(
            read_u8(&[1u8][..]),
            Ok(ParseResult::new(&[][..], 1u8.into()))
        );
        assert_eq!(
            read_u8(&[255u8][..]),
            Ok(ParseResult::new(&[][..], 255u8.into()))
        );
        assert_eq!(
            read_u8(&[1u8, 2][..]),
            Ok(ParseResult::new(&[2u8][..], 1u8.into()))
        );

        assert_eq!(
            read_u16(&[1u8, 0][..]),
            Ok(ParseResult::new(&[][..], 1u16.into()))
        );
        assert_eq!(
            read_u16(&[0u8, 1][..]),
            Ok(ParseResult::new(&[][..], 256u16.into()))
        );
        assert_eq!(
            read_u16(&[136u8, 19][..]),
            Ok(ParseResult::new(&[][..], 5000u16.into()))
        );
        assert_eq!(
            read_u16(&[136u8, 19, 2][..]),
            Ok(ParseResult::new(&[2u8][..], 5000u16.into()))
        );

        assert_eq!(
            read_u32(&[1u8, 0, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 1u32.into()))
        );
        assert_eq!(
            read_u32(&[0u8, 1, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 256u32.into()))
        );
        assert_eq!(
            read_u32(&[136u8, 19, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 5000u32.into()))
        );
        assert_eq!(
            read_u32(&[32u8, 161, 7, 0][..]),
            Ok(ParseResult::new(&[][..], 500000u32.into()))
        );
        assert_eq!(
            read_u32(&[10u8, 10, 10, 10][..]),
            Ok(ParseResult::new(&[][..], 168430090u32.into()))
        );

        assert_eq!(
            read_i32(&[255u8, 255, 255, 255][..]),
            Ok(ParseResult::new(&[][..], (-1i32).into()))
        );
        assert_eq!(
            read_i32(&[0u8, 255, 255, 255][..]),
            Ok(ParseResult::new(&[][..], (-256i32).into()))
        );
        assert_eq!(
            read_i32(&[120u8, 236, 255, 255][..]),
            Ok(ParseResult::new(&[][..], (-5000i32).into()))
        );
        assert_eq!(
            read_i32(&[32u8, 161, 7, 0][..]),
            Ok(ParseResult::new(&[][..], 500000i32.into()))
        );

        assert_eq!(
            read_u64(&[1u8, 0, 0, 0, 0, 0, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 1u64.into()))
        );
        assert_eq!(
            read_u64(&[10u8, 10, 10, 10, 10, 10, 10, 10][..]),
            Ok(ParseResult::new(&[][..], 723401728380766730u64.into()))
        );
    }
}
