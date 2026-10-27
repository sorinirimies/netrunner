#![no_main]
use libfuzzer_sys::fuzz_target;
use netrunner_core::SpeedTestResult;

// The history database stores postcard-encoded results; a corrupt or truncated database
// record must produce an error, never a panic or runaway allocation.
fuzz_target!(|data: &[u8]| {
    if let Ok(result) = postcard::from_bytes::<SpeedTestResult>(data) {
        let bytes = postcard::to_stdvec(&result).expect("encode");
        let _ = postcard::from_bytes::<SpeedTestResult>(&bytes).expect("decode what we encoded");
    }
});
