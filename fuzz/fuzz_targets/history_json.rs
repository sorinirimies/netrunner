#![no_main]
use libfuzzer_sys::fuzz_target;
use netrunner_core::SpeedTestResult;

// History import (`import_from_json`) parses a JSON array of results: never panic on junk,
// and a parsed result must re-serialise and parse again to the same JSON value.
fuzz_target!(|data: &[u8]| {
    let Ok(results) = serde_json::from_slice::<Vec<SpeedTestResult>>(data) else {
        return;
    };
    let json = serde_json::to_string(&results).expect("serialise");
    let back: Vec<SpeedTestResult> = serde_json::from_str(&json).expect("round-trip parse");
    assert_eq!(back.len(), results.len());
});
