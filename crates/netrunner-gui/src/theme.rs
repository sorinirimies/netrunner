//! Cyberpunk colour palette for the GPUI desktop app, mirroring the TUI theme.

use gpui::{rgb, Rgba};

pub const BG: u32 = 0x0a0a14;
pub const PANEL_BG: u32 = 0x12121f;
pub const PANEL_BORDER: u32 = 0x2a2a45;

pub const CYAN: u32 = 0x00ffff;
pub const MAGENTA: u32 = 0xff00ff;
pub const GREEN: u32 = 0x00ff80;
pub const YELLOW: u32 = 0xffdc00;
pub const BLUE: u32 = 0x3c8cff;
pub const RED: u32 = 0xff3c3c;

pub const TEXT: u32 = 0xe6e6f0;
pub const MUTED: u32 = 0x8080a0;

/// Colour for a download bar based on its share of the peak (hot = fast).
pub fn download_color() -> Rgba {
    rgb(GREEN)
}

/// Colour for an upload bar.
pub fn upload_color() -> Rgba {
    rgb(CYAN)
}

/// Build an `Rgba` from 8-bit RGB components (used by the shared macro).
pub fn rgb8(r: u8, g: u8, b: u8) -> Rgba {
    rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
}

// Generate `quality_color` from the canonical palette in `netrunner_core`, so
// the GUI and TUI use identical colours for each connection quality.
netrunner_core::quality_color_fn!(
    /// GPUI colour for a connection quality.
    pub fn quality_color -> Rgba { rgb8 }
);
