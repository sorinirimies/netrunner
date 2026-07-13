//! Shared, UI-agnostic presentation helpers.
//!
//! The cyberpunk colour palette and the mapping from [`ConnectionQuality`](crate::types::ConnectionQuality) to a
//! colour are the same across the Ratatui TUI and the GPUI desktop app. Rather
//! than duplicate the RGB values in every front-end, we define them **once**
//! here and expose the [`quality_color_fn!`](crate::quality_color_fn) macro so each front-end can
//! generate a mapping function for *its own* colour type without depending on
//! any UI crate.

/// Canonical cyberpunk palette as `(r, g, b)` triples.
///
/// Front-ends convert these into their native colour type.
pub mod palette {
    /// Neon green — excellent.
    pub const GREEN: (u8, u8, u8) = (0x00, 0xff, 0x80);
    /// Cyan — good.
    pub const CYAN: (u8, u8, u8) = (0x00, 0xff, 0xff);
    /// Yellow — average.
    pub const YELLOW: (u8, u8, u8) = (0xff, 0xdc, 0x00);
    /// Orange — poor.
    pub const ORANGE: (u8, u8, u8) = (0xff, 0x8c, 0x00);
    /// Red — very poor.
    pub const RED: (u8, u8, u8) = (0xff, 0x3c, 0x3c);
    /// Dim grey — failed / no data.
    pub const DIM: (u8, u8, u8) = (0x50, 0x50, 0x64);
}

/// Return the canonical `(r, g, b)` colour for a [`ConnectionQuality`](crate::types::ConnectionQuality).
///
/// This is the single source of truth used by [`quality_color_fn!`](crate::quality_color_fn).
pub const fn quality_rgb(quality: crate::types::ConnectionQuality) -> (u8, u8, u8) {
    use crate::types::ConnectionQuality::*;
    match quality {
        Excellent => palette::GREEN,
        Good => palette::CYAN,
        Average => palette::YELLOW,
        Poor => palette::ORANGE,
        VeryPoor => palette::RED,
        Failed => palette::DIM,
    }
}

/// Generate a function mapping [`ConnectionQuality`](crate::types::ConnectionQuality) to a front-end colour type.
///
/// The colour is built from the canonical palette in [`quality_rgb`], so the
/// TUI and GUI stay perfectly in sync. Pass any constructor callable as
/// `ctor(r, g, b)`.
///
/// # Examples
///
/// Ratatui (enum-variant constructor):
///
/// ```ignore
/// use ratatui::style::Color;
/// netrunner_core::quality_color_fn!(
///     /// Colour for a connection quality in the TUI.
///     pub fn quality_color -> Color { Color::Rgb }
/// );
/// ```
///
/// GPUI (via a small helper):
///
/// ```ignore
/// fn rgb8(r: u8, g: u8, b: u8) -> gpui::Rgba {
///     gpui::rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
/// }
/// netrunner_core::quality_color_fn!(pub fn quality_color -> gpui::Rgba { rgb8 });
/// ```
#[macro_export]
macro_rules! quality_color_fn {
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident -> $Color:ty { $ctor:expr }
    ) => {
        $(#[$meta])*
        $vis fn $name(quality: $crate::types::ConnectionQuality) -> $Color {
            let (r, g, b) = $crate::presentation::quality_rgb(quality);
            let ctor = $ctor;
            ctor(r, g, b)
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::types::ConnectionQuality;

    #[test]
    fn quality_rgb_is_stable() {
        assert_eq!(
            super::quality_rgb(ConnectionQuality::Excellent),
            super::palette::GREEN
        );
        assert_eq!(
            super::quality_rgb(ConnectionQuality::Good),
            super::palette::CYAN
        );
        assert_eq!(
            super::quality_rgb(ConnectionQuality::Failed),
            super::palette::DIM
        );
    }

    // Prove the macro expands and produces the canonical colour for an
    // arbitrary target type (here a plain tuple constructor).
    crate::quality_color_fn!(fn tuple_color -> (u8, u8, u8) { |r, g, b| (r, g, b) });

    #[test]
    fn macro_uses_canonical_palette() {
        assert_eq!(
            tuple_color(ConnectionQuality::Average),
            super::palette::YELLOW
        );
        assert_eq!(tuple_color(ConnectionQuality::Poor), super::palette::ORANGE);
        assert_eq!(
            tuple_color(ConnectionQuality::VeryPoor),
            super::palette::RED
        );
    }
}
