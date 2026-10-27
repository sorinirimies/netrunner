#![no_main]
use libfuzzer_sys::fuzz_target;
use netrunner_core::{quality_rgb, ConnectionQuality};
use std::str::FromStr;

// Quality rating must be total over all floats (NaN, infinities, negatives) and its
// textual form must round-trip.
fuzz_target!(|v: (f64, f64, f64, &str)| {
    let (down, up, ping, text) = v;
    let q = ConnectionQuality::from_speed_and_ping(down, up, ping);
    let _ = quality_rgb(q);
    assert_eq!(ConnectionQuality::from_str(&q.to_string()), Ok(q));
    let _ = ConnectionQuality::from_str(text);
});
