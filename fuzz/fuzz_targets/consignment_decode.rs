//! Fuzz target for consignment decoding
//!
//! This fuzz target tests the robustness of consignment decoding
//! against malformed or unexpected input.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to decode consignment from fuzzed data
    // This tests that the decoder handles malformed input gracefully
    let _ = csv_core::consignment::Consignment::decode(data);
});
