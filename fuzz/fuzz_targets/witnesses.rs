#![no_main]
use bitcoin_slices::bsl::Witnesses;
use bitcoin_slices::Parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 0 {
        let p = Witnesses::parse(&data[1..], data[0] as usize);
        check(&data[1..], p);
    }
});

/// Some checks on a succesfull parse
pub fn check<T: AsRef<[u8]>>(
    data: &[u8],
    p: Result<bitcoin_slices::ParseResult<T>, bitcoin_slices::Error>,
) {
    if let Ok(p) = p {
        let consumed = p.consumed();
        assert_eq!(p.parsed().as_ref().len(), consumed);
        assert_eq!(&data[..consumed], p.parsed().as_ref());
        assert_eq!(&data[consumed..], p.remaining());
    }
}
