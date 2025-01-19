#![no_main]
use bitcoin_slices::bsl::{parse_len, scan_len};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(len) = parse_len(data) {
        let mut consumed = 0;
        let scan_len = scan_len(data, &mut consumed).unwrap();
        assert_eq!(len.n(), scan_len);
        assert_eq!(len.consumed(), consumed);
    }
});
