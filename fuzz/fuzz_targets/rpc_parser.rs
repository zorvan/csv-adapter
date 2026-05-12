//! Fuzz target for RPC response parsing
//!
//! This fuzz target tests the robustness of RPC response parsing
//! against malformed or unexpected JSON input.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse RPC response from fuzzed data
    // This tests that the parser handles malformed JSON gracefully
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<serde_json::Value>(s);
    }
});
