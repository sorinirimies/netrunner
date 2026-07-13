//! Animated intro screen with tachyonfx effects
//!
//! Displays the Netrunner logo with cyberpunk glow animations when the app starts.

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::logo::{NetrunnerLogo, NetrunnerLogoSize};

// Cyberpunk color palette for border effects
const BORDER_COLORS: [Color; 6] = [
    Color::Rgb(0, 255, 255),   // Cyan bright
    Color::Rgb(100, 255, 255), // Cyan lighter
    Color::Rgb(0, 200, 200),   // Cyan dim
    Color::Rgb(255, 0, 255),   // Magenta
    Color::Rgb(0, 255, 150),   // Green neon
    Color::Rgb(255, 255, 0),   // Yellow
];

/// Display the animated intro screen with glowing logo
pub fn show_intro() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the intro animation
    let result = run_intro_animation(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_intro_animation(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let duration_ms = 3000; // 3 second intro
    let start = Instant::now();
    let mut frame_count = 0u32;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            // Create layout
            let chunks = Layout::vertical([
                Constraint::Length(2),
                Constraint::Min(10),
                Constraint::Length(5),
            ])
            .split(area);

            // Render logo in the center with animated glow
            let logo_area = center_rect(chunks[1], 80, 15);

            // Calculate animation progress
            let elapsed_ms = start.elapsed().as_millis() as u32;
            let progress = (elapsed_ms as f64 / duration_ms as f64).min(1.0);

            // Render the base logo with pulsing glow effect
            let logo = NetrunnerLogo::new(NetrunnerLogoSize::Medium);
            frame.render_widget(logo, logo_area);

            // Add animated color-cycling border effect
            if progress > 0.3 {
                draw_animated_border(frame, logo_area, frame_count, progress);
            }

            // Render tagline with animation
            render_tagline(frame, chunks[2], progress, frame_count);

            // Render skip hint
            render_skip_hint(frame, chunks[0]);
        })?;

        frame_count = frame_count.wrapping_add(1);

        // Check for input to skip
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Auto-exit after duration
        if start.elapsed().as_millis() >= duration_ms as u128 {
            break;
        }

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    Ok(())
}

fn draw_animated_border(frame: &mut ratatui::Frame, area: Rect, frame_count: u32, progress: f64) {
    // Speed: cells per frame (higher = faster)
    let speed = 0.5;
    let color_cycle_idx = (frame_count as f64 * speed) as usize;

    // Function to get color at a specific position along the perimeter
    let get_color = |idx: usize| -> Color {
        let color_idx = (color_cycle_idx + idx) % (BORDER_COLORS.len() * 10);
        let base_idx = color_idx / 10;
        let sub_idx = color_idx % 10;

        // Interpolate between colors for smooth transitions
        let current_color = BORDER_COLORS[base_idx % BORDER_COLORS.len()];
        let next_color = BORDER_COLORS[(base_idx + 1) % BORDER_COLORS.len()];

        if sub_idx < 5 {
            current_color
        } else {
            blend_colors(current_color, next_color, (sub_idx - 5) as f64 / 5.0)
        }
    };

    // Apply fade-in effect based on progress
    let alpha = ((progress - 0.3) / 0.7).min(1.0);

    let mut cell_index = 0;

    // Top border (left to right)
    if area.y > 0 {
        for x in area.x..area.x + area.width {
            let color = apply_fade(get_color(cell_index), alpha);
            if let Some(cell) = frame.buffer_mut().cell_mut((x, area.y.saturating_sub(1))) {
                cell.set_style(Style::default().fg(color)).set_symbol("▀");
            }
            cell_index += 1;
        }
    }

    // Right border (top to bottom)
    if area.x + area.width < frame.area().width {
        for y in area.y..area.y + area.height {
            let color = apply_fade(get_color(cell_index), alpha);
            if let Some(cell) = frame.buffer_mut().cell_mut((area.x + area.width, y)) {
                cell.set_style(Style::default().fg(color)).set_symbol("█");
            }
            cell_index += 1;
        }
    }

    // Bottom border (right to left)
    if area.y + area.height < frame.area().height {
        for x in (area.x..area.x + area.width).rev() {
            let color = apply_fade(get_color(cell_index), alpha);
            if let Some(cell) = frame.buffer_mut().cell_mut((x, area.y + area.height)) {
                cell.set_style(Style::default().fg(color)).set_symbol("▄");
            }
            cell_index += 1;
        }
    }

    // Left border (bottom to top)
    if area.x > 0 {
        for y in (area.y..area.y + area.height).rev() {
            let color = apply_fade(get_color(cell_index), alpha);
            if let Some(cell) = frame.buffer_mut().cell_mut((area.x.saturating_sub(1), y)) {
                cell.set_style(Style::default().fg(color)).set_symbol("█");
            }
            cell_index += 1;
        }
    }
}

fn blend_colors(color1: Color, color2: Color, factor: f64) -> Color {
    match (color1, color2) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f64 + (r2 as f64 - r1 as f64) * factor) as u8;
            let g = (g1 as f64 + (g2 as f64 - g1 as f64) * factor) as u8;
            let b = (b1 as f64 + (b2 as f64 - b1 as f64) * factor) as u8;
            Color::Rgb(r, g, b)
        }
        _ => color1,
    }
}

fn apply_fade(color: Color, alpha: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let r = (r as f64 * alpha) as u8;
            let g = (g as f64 * alpha) as u8;
            let b = (b as f64 * alpha) as u8;
            Color::Rgb(r, g, b)
        }
        _ => color,
    }
}

fn render_tagline(frame: &mut ratatui::Frame, area: Rect, progress: f64, frame_count: u32) {
    // Fade in tagline after 30% progress
    if progress < 0.3 {
        return;
    }

    let pulse = (frame_count as f64 * 0.04).sin() * 0.5 + 0.5;
    let intensity = (pulse * 100.0) as u8 + 155;
    let glow_color = Color::Rgb(0, intensity, intensity);

    let fade_progress = ((progress - 0.3) / 0.7).min(1.0);
    let text_intensity = (fade_progress * 255.0) as u8;
    let text_color = Color::Rgb(0, text_intensity, text_intensity);

    let tagline = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(">>> ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "JACK IN AND TRACE THE NET",
                Style::default().fg(glow_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" <<<", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![Span::styled(
            "Cyberpunk Network Diagnostics",
            Style::default()
                .fg(text_color)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let paragraph = Paragraph::new(tagline).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_skip_hint(frame: &mut ratatui::Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(vec![Span::styled(
        "Press any key to skip...",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM),
    )]))
    .alignment(Alignment::Right);
    frame.render_widget(hint, area);
}

fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Length((area.height.saturating_sub(height)) / 2),
        Constraint::Length(height),
        Constraint::Min(0),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Length((area.width.saturating_sub(width)) / 2),
        Constraint::Length(width),
        Constraint::Min(0),
    ])
    .split(vertical[1])[1]
}

/// Simple intro without animation (fallback)
pub fn show_simple_intro() -> io::Result<()> {
    println!(
        r"
 _   _ ______ _______ _____  _    _ _   _ _   _ ______ _____
| \ | |  ____|__   __|  __ \| |  | | \ | | \ | |  ____|  __ \
|  \| | |__     | |  | |__) | |  | |  \| |  \| | |__  | |__) |
| . ` |  __|    | |  |  _  /| |  | | . ` | . ` |  __| |  _  /
| |\  | |____   | |  | | \ \| |__| | |\  | |\  | |____| | \ \
|_| \_|______|  |_|  |_|  \_\\____/|_| \_|_| \_|______|_|  \_\

>>> JACK IN AND TRACE THE NET <<<
    Cyberpunk Network Diagnostics
"
    );
    std::thread::sleep(Duration::from_millis(1500));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = center_rect(area, 50, 20);
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 20);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 15);
    }
}
