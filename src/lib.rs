//! # Bitcoin Slices
//!
//! ZERO allocations parse library for Bitcoin data structures available in the [`bsl`] module.
//!
//! Data can be accessed via the [`crate::Visitor`]s during parsing.

#![cfg_attr(bench, feature(test))]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

#[cfg(bench)]
extern crate test;

pub mod bsl;
mod error;
pub mod number;
mod parse_result;
mod slice;
mod visit;

pub use error::Error;
pub use parse_result::ParseResult;
pub use visit::{EmptyVisitor, Visit, Visitor};

type SResult<'a, T> = Result<ParseResult<'a, T>, Error>;

#[cfg(any(test, bench))]
pub mod test_common {
    use hex_lit::hex;

    use crate::ParseResult;

    pub const GENESIS_TX: [u8; 204] = hex!("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000");
    pub const GENESIS_BLOCK_HEADER: [u8; 80] = hex!("0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c");
    pub const GENESIS_BLOCK: [u8;285] = hex!("0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000");

    impl<'a, T: AsRef<[u8]>> ParseResult<'a, T> {
        pub fn new_exact(parsed: T) -> Self {
            ParseResult::new(&[], parsed)
        }
    }

    pub fn reverse(arr: [u8; 32]) -> [u8; 32] {
        let mut ret = arr;
        ret.reverse();
        ret
    }
}

/// Common functions used in fuzzing
#[cfg(fuzzing)]
pub mod fuzzing {
    use crate::{Error, ParseResult};

    /// Some checks on a succesfull parse
    pub fn check<T: AsRef<[u8]>>(data: &[u8], p: Result<ParseResult<T>, Error>) {
        if let Ok(p) = p {
            let consumed = p.consumed();
            assert_eq!(p.parsed().as_ref().len(), consumed);
            assert_eq!(&data[..consumed], p.parsed().as_ref());
            assert_eq!(&data[consumed..], p.remaining());
        }
    }
}
