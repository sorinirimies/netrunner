//! Netrunner Logo Widget
//!
//! A cyberpunk-styled logo widget that draws "NETRUNNER" using geometric shapes and lines.
//! Inspired by the Ratatui logo implementation with a futuristic aesthetic.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

/// Size variants for the Netrunner logo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum NetrunnerLogoSize {
    /// Tiny logo (3 lines high)
    Tiny,
    /// Small logo (5 lines high)
    Small,
    /// Medium logo (7 lines high) - default
    #[default]
    Medium,
}

/// The Netrunner logo widget with cyberpunk aesthetic
#[derive(Debug, Clone, Copy, Default)]
pub struct NetrunnerLogo {
    size: NetrunnerLogoSize,
}

impl NetrunnerLogo {
    /// Creates a new Netrunner logo with the specified size
    pub const fn new(size: NetrunnerLogoSize) -> Self {
        Self { size }
    }
}

impl Widget for NetrunnerLogo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.size {
            NetrunnerLogoSize::Tiny => render_tiny(area, buf),
            NetrunnerLogoSize::Small => render_small(area, buf),
            NetrunnerLogoSize::Medium => render_medium(area, buf),
        }
    }
}

// Cyberpunk color palette
const CYAN: Color = Color::Rgb(0, 255, 255);
const CYAN_BRIGHT: Color = Color::Rgb(100, 255, 255);
const CYAN_DIM: Color = Color::Rgb(0, 200, 200);
const MAGENTA: Color = Color::Rgb(255, 0, 255);
const YELLOW: Color = Color::Rgb(255, 255, 0);
const GREEN_NEON: Color = Color::Rgb(0, 255, 150);

fn render_medium(area: Rect, buf: &mut Buffer) {
    let height = 7;
    let width = 70;

    if area.width < width || area.height < height {
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    // Line 0: Top bars and accents
    draw_horizontal_line(buf, x, y, 8, CYAN_BRIGHT, "▀");
    draw_horizontal_line(buf, x + 10, y, 8, CYAN, "▀");
    draw_horizontal_line(buf, x + 20, y, 8, MAGENTA, "▀");
    draw_horizontal_line(buf, x + 30, y, 8, CYAN, "▀");
    draw_horizontal_line(buf, x + 40, y, 8, YELLOW, "▀");
    draw_horizontal_line(buf, x + 50, y, 8, CYAN_BRIGHT, "▀");
    draw_horizontal_line(buf, x + 60, y, 8, GREEN_NEON, "▀");

    // Line 1: N E T R U N N E R
    let y1 = y + 1;
    // N
    draw_vertical_line(buf, x, y1, 5, CYAN_BRIGHT, "█");
    draw_diagonal_line(buf, x + 1, y1, 4, CYAN_BRIGHT, "▀");
    draw_vertical_line(buf, x + 5, y1, 5, CYAN_BRIGHT, "█");

    // E
    draw_vertical_line(buf, x + 8, y1, 5, CYAN, "█");
    draw_horizontal_line(buf, x + 9, y1, 3, CYAN, "▀");
    draw_horizontal_line(buf, x + 9, y1 + 2, 2, CYAN, "█");
    draw_horizontal_line(buf, x + 9, y1 + 4, 3, CYAN, "▄");

    // T
    draw_horizontal_line(buf, x + 14, y1, 5, MAGENTA, "▀");
    draw_vertical_line(buf, x + 16, y1 + 1, 4, MAGENTA, "█");

    // R
    draw_vertical_line(buf, x + 21, y1, 5, CYAN, "█");
    draw_horizontal_line(buf, x + 22, y1, 3, CYAN, "▀");
    draw_cell(buf, x + 25, y1 + 1, CYAN, "▄");
    draw_horizontal_line(buf, x + 22, y1 + 2, 2, CYAN, "▀");
    draw_diagonal_line(buf, x + 24, y1 + 3, 2, CYAN, "▄");

    // U
    draw_vertical_line(buf, x + 28, y1, 4, YELLOW, "█");
    draw_horizontal_line(buf, x + 29, y1 + 4, 3, YELLOW, "▄");
    draw_vertical_line(buf, x + 32, y1, 4, YELLOW, "█");

    // N
    draw_vertical_line(buf, x + 35, y1, 5, CYAN_BRIGHT, "█");
    draw_diagonal_line(buf, x + 36, y1, 4, CYAN_BRIGHT, "▀");
    draw_vertical_line(buf, x + 40, y1, 5, CYAN_BRIGHT, "█");

    // N
    draw_vertical_line(buf, x + 43, y1, 5, CYAN, "█");
    draw_diagonal_line(buf, x + 44, y1, 4, CYAN, "▀");
    draw_vertical_line(buf, x + 48, y1, 5, CYAN, "█");

    // E
    draw_vertical_line(buf, x + 51, y1, 5, GREEN_NEON, "█");
    draw_horizontal_line(buf, x + 52, y1, 3, GREEN_NEON, "▀");
    draw_horizontal_line(buf, x + 52, y1 + 2, 2, GREEN_NEON, "█");
    draw_horizontal_line(buf, x + 52, y1 + 4, 3, GREEN_NEON, "▄");

    // R
    draw_vertical_line(buf, x + 57, y1, 5, MAGENTA, "█");
    draw_horizontal_line(buf, x + 58, y1, 3, MAGENTA, "▀");
    draw_cell(buf, x + 61, y1 + 1, MAGENTA, "▄");
    draw_horizontal_line(buf, x + 58, y1 + 2, 2, MAGENTA, "▀");
    draw_diagonal_line(buf, x + 60, y1 + 3, 2, MAGENTA, "▄");

    // Line 6: Bottom accent line with glitch effect
    let y6 = y + 6;
    draw_horizontal_line(buf, x, y6, 10, CYAN_DIM, "▄");
    draw_horizontal_line(buf, x + 15, y6, 8, CYAN_BRIGHT, "▄");
    draw_horizontal_line(buf, x + 28, y6, 10, MAGENTA, "▄");
    draw_horizontal_line(buf, x + 43, y6, 12, CYAN, "▄");
    draw_horizontal_line(buf, x + 58, y6, 8, GREEN_NEON, "▄");

    // Add glitch markers
    draw_cell(buf, x + 12, y1, MAGENTA, "▓");
    draw_cell(buf, x + 26, y1 + 3, CYAN_BRIGHT, "▒");
    draw_cell(buf, x + 41, y1 + 1, YELLOW, "░");
    draw_cell(buf, x + 55, y1 + 4, CYAN, "▓");
}

fn render_small(area: Rect, buf: &mut Buffer) {
    let height = 5;
    let width = 50;

    if area.width < width || area.height < height {
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    // Top accent
    draw_horizontal_line(buf, x, y, 10, CYAN_BRIGHT, "▀");
    draw_horizontal_line(buf, x + 15, y, 10, MAGENTA, "▀");
    draw_horizontal_line(buf, x + 30, y, 10, GREEN_NEON, "▀");

    let y1 = y + 1;

    // N E T
    draw_vertical_line(buf, x, y1, 3, CYAN_BRIGHT, "█");
    draw_diagonal_line(buf, x + 1, y1, 2, CYAN_BRIGHT, "▀");
    draw_vertical_line(buf, x + 3, y1, 3, CYAN_BRIGHT, "█");

    draw_vertical_line(buf, x + 6, y1, 3, CYAN, "█");
    draw_horizontal_line(buf, x + 7, y1, 2, CYAN, "▀");
    draw_horizontal_line(buf, x + 7, y1 + 2, 2, CYAN, "▄");

    draw_horizontal_line(buf, x + 11, y1, 3, MAGENTA, "▀");
    draw_vertical_line(buf, x + 12, y1 + 1, 2, MAGENTA, "█");

    // R U N
    draw_vertical_line(buf, x + 16, y1, 3, CYAN, "█");
    draw_horizontal_line(buf, x + 17, y1, 2, CYAN, "▀");
    draw_diagonal_line(buf, x + 18, y1 + 1, 2, CYAN, "▄");

    draw_vertical_line(buf, x + 21, y1, 2, YELLOW, "█");
    draw_horizontal_line(buf, x + 22, y1 + 2, 2, YELLOW, "▄");
    draw_vertical_line(buf, x + 24, y1, 2, YELLOW, "█");

    draw_vertical_line(buf, x + 27, y1, 3, CYAN_BRIGHT, "█");
    draw_diagonal_line(buf, x + 28, y1, 2, CYAN_BRIGHT, "▀");
    draw_vertical_line(buf, x + 30, y1, 3, CYAN_BRIGHT, "█");

    // N E R
    draw_vertical_line(buf, x + 33, y1, 3, CYAN, "█");
    draw_diagonal_line(buf, x + 34, y1, 2, CYAN, "▀");
    draw_vertical_line(buf, x + 36, y1, 3, CYAN, "█");

    draw_vertical_line(buf, x + 39, y1, 3, GREEN_NEON, "█");
    draw_horizontal_line(buf, x + 40, y1, 2, GREEN_NEON, "▀");
    draw_horizontal_line(buf, x + 40, y1 + 2, 2, GREEN_NEON, "▄");

    draw_vertical_line(buf, x + 44, y1, 3, MAGENTA, "█");
    draw_horizontal_line(buf, x + 45, y1, 2, MAGENTA, "▀");
    draw_diagonal_line(buf, x + 46, y1 + 1, 2, MAGENTA, "▄");

    // Bottom accent
    draw_horizontal_line(buf, x, y + 4, 15, CYAN_DIM, "▄");
    draw_horizontal_line(buf, x + 20, y + 4, 15, CYAN_BRIGHT, "▄");
    draw_horizontal_line(buf, x + 40, y + 4, 10, MAGENTA, "▄");
}

fn render_tiny(area: Rect, buf: &mut Buffer) {
    let height = 3;
    let width = 35;

    if area.width < width || area.height < height {
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    // Top line with accent
    draw_horizontal_line(buf, x, y, 12, CYAN_BRIGHT, "▀");
    draw_horizontal_line(buf, x + 15, y, 12, MAGENTA, "▀");

    // NETRUNNER in compact form
    let y1 = y + 1;

    // Simplified letters using blocks
    buf.set_string(x, y1, "█▀█", Style::default().fg(CYAN_BRIGHT));
    buf.set_string(x + 4, y1, "█▀", Style::default().fg(CYAN));
    buf.set_string(x + 7, y1, "▀█▀", Style::default().fg(MAGENTA));
    buf.set_string(x + 11, y1, "█▀▄", Style::default().fg(CYAN));
    buf.set_string(x + 15, y1, "█─█", Style::default().fg(YELLOW));
    buf.set_string(x + 19, y1, "█▀█", Style::default().fg(CYAN_BRIGHT));
    buf.set_string(x + 23, y1, "█▀█", Style::default().fg(CYAN));
    buf.set_string(x + 27, y1, "█▀", Style::default().fg(GREEN_NEON));
    buf.set_string(x + 30, y1, "█▀▄", Style::default().fg(MAGENTA));

    // Bottom accent
    draw_horizontal_line(buf, x, y + 2, 35, CYAN_DIM, "▄");
}

// Helper functions for drawing primitives
fn draw_cell(buf: &mut Buffer, x: u16, y: u16, color: Color, symbol: &str) {
    if x < buf.area.right() && y < buf.area.bottom() {
        buf.set_string(x, y, symbol, Style::default().fg(color));
    }
}

fn draw_horizontal_line(buf: &mut Buffer, x: u16, y: u16, length: u16, color: Color, symbol: &str) {
    for i in 0..length {
        draw_cell(buf, x + i, y, color, symbol);
    }
}

fn draw_vertical_line(buf: &mut Buffer, x: u16, y: u16, length: u16, color: Color, symbol: &str) {
    for i in 0..length {
        draw_cell(buf, x, y + i, color, symbol);
    }
}

fn draw_diagonal_line(buf: &mut Buffer, x: u16, y: u16, length: u16, color: Color, symbol: &str) {
    for i in 0..length {
        draw_cell(buf, x + i, y + i, color, symbol);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_creation() {
        let logo = NetrunnerLogo::new(NetrunnerLogoSize::Medium);
        assert_eq!(logo.size, NetrunnerLogoSize::Medium);
    }

    #[test]
    fn test_default_size() {
        let logo = NetrunnerLogo::default();
        assert_eq!(logo.size, NetrunnerLogoSize::Medium);
    }
}
