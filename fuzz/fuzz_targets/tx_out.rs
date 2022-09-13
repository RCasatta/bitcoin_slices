#![no_main]
use bitcoin_slices::bsl::TxOut;
use bitcoin_slices::fuzzing::check;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let p = TxOut::parse(data);
    check(data, p);
});
