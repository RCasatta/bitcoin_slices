#![no_main]
use bitcoin_slices::bsl::Witnesses;
use bitcoin_slices::fuzzing::check;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 0 {
        let p = Witnesses::parse(&data[1..], data[0] as usize);
        check(&data[1..], p);
    }
});
