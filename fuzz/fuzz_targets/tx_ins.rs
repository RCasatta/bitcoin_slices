#![no_main]
use bitcoin_slices::bsl::TxIns;
use bitcoin_slices::fuzzing::check;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let p = TxIns::parse(data);
    check(data, p);
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
