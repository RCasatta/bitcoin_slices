#![no_main]
use bitcoin_slices::bsl::Block;
use bitcoin_slices::fuzzing::check;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let p = Block::parse(data);
    check(data, p);
});
