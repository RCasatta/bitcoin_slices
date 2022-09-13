#![no_main]
use bitcoin_slices::bsl::TxIn;
use bitcoin_slices::fuzzing::check;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let p = TxIn::parse(data);
    check(data, p);
});
