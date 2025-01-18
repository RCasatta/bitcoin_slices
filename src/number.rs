//! Contains methods to parse numbers (u8,u16,u32,u64,i32) from slices

use core::convert::TryInto;

use crate::{bsl::Len, visit::Parse, Error, ParseResult, SResult, Visit};

/// Converts into and from u8 and implements [`Visit`] and `AsRef<[u8]>`.
#[derive(Debug, PartialEq, Eq)]
#[allow(deprecated)]
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

impl<'a> Parse<'a> for U8 {
    fn parse(slice: &'a [u8]) -> SResult<'a, Self> {
        #[allow(deprecated)]
        let p = crate::slice::read_slice(slice, 1)?;
        Ok(ParseResult::new(p.remaining(), U8([p.parsed()[0]])))
    }
}

macro_rules! impl_number {
    ($primitive:ty, $newtype:ident, $size:expr) => {
        #[doc=concat!("Converts into and from ", stringify!($primitive), " and implements [`Visit`] and `AsRef<[u8]>`." )]
        #[derive(Debug, PartialEq, Eq)]
        pub struct $newtype([u8; $size]);

        impl<'a> Visit<'a> for $newtype {
            fn visit<'b, V: crate::Visitor>(
                slice: &'a [u8],
                _visit: &'b mut V,
            ) -> SResult<'a, Self> {
                #[allow(deprecated)]
                let p = crate::slice::read_slice(slice, $size)?;
                let remaining = p.remaining();
                let arr = p
                    .parsed_owned()
                    .try_into()
                    .expect(concat!("slice length is ", $size));
                Ok(ParseResult::new(remaining, $newtype(arr)))
            }
        }

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

impl U64 {
    /// Convert to `Len`
    pub fn to_len(self) -> Result<Len, Error> {
        let n: u64 = self.into();
        if n > u32::MAX as u64 {
            Ok(Len { n, consumed: 9 })
        } else {
            Err(Error::NonMinimalVarInt)
        }
    }
}

impl U32 {
    /// Convert to `Len`
    pub fn to_len(self) -> Result<Len, Error> {
        let n: u64 = u32::from(self) as u64;
        if n > u16::MAX as u64 {
            Ok(Len { n, consumed: 5 })
        } else {
            Err(Error::NonMinimalVarInt)
        }
    }
}

impl U16 {
    /// Convert to `Len`
    pub fn to_len(self) -> Result<Len, Error> {
        let n: u64 = u16::from(self) as u64;
        if n >= 0xFD {
            Ok(Len { n, consumed: 3 })
        } else {
            Err(Error::NonMinimalVarInt)
        }
    }
}

#[inline(always)]
/// Read a u8 from a slice
pub fn read_u8(slice: &[u8]) -> Result<u8, Error> {
    slice.first().cloned().ok_or(Error::MoreBytesNeeded)
}

#[inline(always)]
/// Read a u16 from a slice
pub fn read_u16(slice: &[u8]) -> Result<u16, Error> {
    let arr: [u8; 2] = slice
        .get(..2)
        .ok_or(Error::MoreBytesNeeded)?
        .try_into()
        .expect("slice length is 2");
    Ok(u16::from_le_bytes(arr))
}

#[inline(always)]
/// Read a u32 from a slice
pub fn read_u32(slice: &[u8]) -> Result<u32, Error> {
    let arr: [u8; 4] = slice
        .get(..4)
        .ok_or(Error::MoreBytesNeeded)?
        .try_into()
        .expect("slice length is 4");
    Ok(u32::from_le_bytes(arr))
}

#[inline(always)]
/// Read a i32 from a slice
pub fn read_i32(slice: &[u8]) -> Result<i32, Error> {
    let arr: [u8; 4] = slice
        .get(..4)
        .ok_or(Error::MoreBytesNeeded)?
        .try_into()
        .expect("slice length is 4");
    Ok(i32::from_le_bytes(arr))
}

#[inline(always)]
/// Read a u64 from a slice
pub fn read_u64(slice: &[u8]) -> Result<u64, Error> {
    let arr: [u8; 8] = slice
        .get(..8)
        .ok_or(Error::MoreBytesNeeded)?
        .try_into()
        .expect("slice length is 8");
    Ok(u64::from_le_bytes(arr))
}

#[cfg(test)]
mod test {
    use crate::ParseResult;

    use super::*;
    #[test]
    fn test_read_functions() {
        assert_eq!(read_u8(&[1u8]), Ok(1u8));
        assert_eq!(read_u8(&[255u8]), Ok(255u8));
        assert_eq!(read_u8(&[1u8, 2]), Ok(1u8));
        assert_eq!(read_u8(&[]), Err(Error::MoreBytesNeeded));

        assert_eq!(read_u32(&[1u8, 0, 0, 0]), Ok(1u32));
        assert_eq!(read_u32(&[0u8, 1, 0, 0]), Ok(256u32));
        assert_eq!(read_u32(&[136u8, 19, 0, 0]), Ok(5000u32));
        assert_eq!(read_u32(&[32u8, 161, 7, 0]), Ok(500000u32));
        assert_eq!(read_u32(&[10u8, 10, 10, 10]), Ok(168430090u32));
        assert_eq!(read_u32(&[1u8]), Err(Error::MoreBytesNeeded));
        assert_eq!(read_u32(&[1u8, 2]), Err(Error::MoreBytesNeeded));
        assert_eq!(read_u32(&[1u8, 2, 3]), Err(Error::MoreBytesNeeded));

        assert_eq!(read_i32(&[255u8, 255, 255, 255]), Ok(-1i32));
        assert_eq!(read_i32(&[0u8, 255, 255, 255]), Ok(-256i32));
        assert_eq!(read_i32(&[120u8, 236, 255, 255]), Ok(-5000i32));
        assert_eq!(read_i32(&[32u8, 161, 7, 0]), Ok(500000i32));
        assert_eq!(read_i32(&[1u8]), Err(Error::MoreBytesNeeded));
        assert_eq!(read_i32(&[1u8, 2]), Err(Error::MoreBytesNeeded));
        assert_eq!(read_i32(&[1u8, 2, 3]), Err(Error::MoreBytesNeeded));

        assert_eq!(read_u64(&[1u8, 0, 0, 0, 0, 0, 0, 0]), Ok(1u64));
        assert_eq!(
            read_u64(&[10u8, 10, 10, 10, 10, 10, 10, 10]),
            Ok(723401728380766730u64)
        );
        assert_eq!(read_u64(&[1u8]), Err(Error::MoreBytesNeeded));
        assert_eq!(read_u64(&[1u8, 2, 3, 4]), Err(Error::MoreBytesNeeded));
        assert_eq!(
            read_u64(&[1u8, 2, 3, 4, 5, 6, 7]),
            Err(Error::MoreBytesNeeded)
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            U8::parse(&[0u8][..]),
            Ok(ParseResult::new(&[][..], 0u8.into()))
        );
        assert_eq!(
            U8::parse(&[1u8][..]),
            Ok(ParseResult::new(&[][..], 1u8.into()))
        );
        assert_eq!(
            U8::parse(&[255u8][..]),
            Ok(ParseResult::new(&[][..], 255u8.into()))
        );
        assert_eq!(
            U8::parse(&[1u8, 2][..]),
            Ok(ParseResult::new(&[2u8][..], 1u8.into()))
        );

        assert_eq!(
            U16::parse(&[1u8, 0][..]),
            Ok(ParseResult::new(&[][..], 1u16.into()))
        );
        assert_eq!(
            U16::parse(&[0u8, 1][..]),
            Ok(ParseResult::new(&[][..], 256u16.into()))
        );
        assert_eq!(
            U16::parse(&[136u8, 19][..]),
            Ok(ParseResult::new(&[][..], 5000u16.into()))
        );
        assert_eq!(
            U16::parse(&[136u8, 19, 2][..]),
            Ok(ParseResult::new(&[2u8][..], 5000u16.into()))
        );

        assert_eq!(
            U32::parse(&[1u8, 0, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 1u32.into()))
        );
        assert_eq!(
            U32::parse(&[0u8, 1, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 256u32.into()))
        );
        assert_eq!(
            U32::parse(&[136u8, 19, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 5000u32.into()))
        );
        assert_eq!(
            U32::parse(&[32u8, 161, 7, 0][..]),
            Ok(ParseResult::new(&[][..], 500000u32.into()))
        );
        assert_eq!(
            U32::parse(&[10u8, 10, 10, 10][..]),
            Ok(ParseResult::new(&[][..], 168430090u32.into()))
        );

        assert_eq!(
            I32::parse(&[255u8, 255, 255, 255][..]),
            Ok(ParseResult::new(&[][..], (-1i32).into()))
        );
        assert_eq!(
            I32::parse(&[0u8, 255, 255, 255][..]),
            Ok(ParseResult::new(&[][..], (-256i32).into()))
        );
        assert_eq!(
            I32::parse(&[120u8, 236, 255, 255][..]),
            Ok(ParseResult::new(&[][..], (-5000i32).into()))
        );
        assert_eq!(
            I32::parse(&[32u8, 161, 7, 0][..]),
            Ok(ParseResult::new(&[][..], 500000i32.into()))
        );

        assert_eq!(
            U64::parse(&[1u8, 0, 0, 0, 0, 0, 0, 0][..]),
            Ok(ParseResult::new(&[][..], 1u64.into()))
        );
        assert_eq!(
            U64::parse(&[10u8, 10, 10, 10, 10, 10, 10, 10][..]),
            Ok(ParseResult::new(&[][..], 723401728380766730u64.into()))
        );
    }
}
