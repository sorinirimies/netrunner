#![no_main]
use libfuzzer_sys::fuzz_target;
use netrunner_core::Settings;

// Arbitrary settings.json content must never panic, and whatever parses must convert to a
// config and survive a serialise/parse round trip.
fuzz_target!(|data: &[u8]| {
    let Ok(settings) = serde_json::from_slice::<Settings>(data) else {
        return;
    };
    let _ = settings.to_config();
    let json = serde_json::to_string(&settings).expect("settings serialise");
    let back: Settings = serde_json::from_str(&json).expect("settings round-trip");
    assert_eq!(serde_json::to_value(&back).unwrap(), serde_json::to_value(&settings).unwrap());
});
