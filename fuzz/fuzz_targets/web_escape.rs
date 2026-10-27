#![no_main]
use libfuzzer_sys::fuzz_target;
use netrunner_web::escape_html;

// Escaped output must never contain raw markup characters, and every `&` must start one of
// our entities — i.e. the output is inert inside HTML element content and attributes.
fuzz_target!(|s: &str| {
    let out = escape_html(s);
    assert!(!out.contains(['<', '>', '"', '\'']));
    let mut rest = out.as_str();
    while let Some(i) = rest.find('&') {
        rest = &rest[i..];
        assert!(["&amp;", "&lt;", "&gt;", "&quot;", "&#39;"].iter().any(|e| rest.starts_with(e)));
        rest = &rest[1..];
    }
});
