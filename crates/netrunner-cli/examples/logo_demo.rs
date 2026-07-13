//! # Netrunner Logo Demo
//!
//! A simple example demonstrating the Netrunner logo widget with cyberpunk aesthetics.
//!
//! Usage:
//!   cargo run --example logo_demo [size]
//!
//! Arguments:
//!   size: tiny, small, medium (default: medium)
//!
//! Examples:
//!   cargo run --example logo_demo
//!   cargo run --example logo_demo small
//!   cargo run --example logo_demo tiny

use std::env::args;
use std::io;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use netrunner_cli::{NetrunnerLogo, NetrunnerLogoSize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    Terminal,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = match args().nth(1).as_deref() {
        Some("small") => NetrunnerLogoSize::Small,
        Some("tiny") => NetrunnerLogoSize::Tiny,
        _ => NetrunnerLogoSize::default(),
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run(&mut terminal, size);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    println!();
    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    size: NetrunnerLogoSize,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            use Constraint::{Fill, Length};
            let [top, bottom] = Layout::vertical([Length(1), Fill(1)]).areas(frame.area());
            frame.render_widget(">>> Powered by <<<", top);
            frame.render_widget(NetrunnerLogo::new(size), bottom);
        })?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}
