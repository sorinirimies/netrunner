use colored::*;
use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::RngExt as _;
use std::sync::Arc;
use tokio::sync::RwLock;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use netrunner_core::TestConfig;

// Bandwidth monitor state for real-time graph
#[derive(Clone)]
pub struct BandwidthMonitor {
    pub speed_history: Arc<RwLock<Vec<f64>>>,
    pub current_speed: Arc<RwLock<f64>>,
    pub peak_speed: Arc<RwLock<f64>>,
    pub is_final: Arc<RwLock<bool>>,
    pub throbber_frame: Arc<RwLock<usize>>,
    #[allow(dead_code)]
    pub title: String,
    pub label: String,
}

impl BandwidthMonitor {
    pub fn new(title: String, label: String) -> Self {
        Self {
            speed_history: Arc::new(RwLock::new(Vec::new())),
            current_speed: Arc::new(RwLock::new(0.0)),
            peak_speed: Arc::new(RwLock::new(0.0)),
            is_final: Arc::new(RwLock::new(false)),
            throbber_frame: Arc::new(RwLock::new(0)),
            title,
            label,
        }
    }

    pub async fn update(&self, speed: f64) {
        let mut history = self.speed_history.write().await;
        let mut current = self.current_speed.write().await;
        let mut peak = self.peak_speed.write().await;
        let mut frame = self.throbber_frame.write().await;

        *current = speed;
        *peak = peak.max(speed);
        history.push(speed);

        // Advance throbber animation (10 frames for complete circle)
        *frame = (*frame + 1) % 10;

        // Keep only last 100 samples for graph
        if history.len() > 100 {
            history.remove(0);
        }
    }

    pub async fn mark_final(&self) {
        let mut is_final = self.is_final.write().await;
        *is_final = true;
    }

    pub async fn render_live(&self) -> io::Result<()> {
        let history = self.speed_history.read().await;
        let current = self.current_speed.read().await;
        let peak = self.peak_speed.read().await;
        let is_final = self.is_final.read().await;
        let frame = self.throbber_frame.read().await;

        // Display speed with throbber or checkmark
        let throbber_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let indicator = if *is_final {
            "✓"
        } else {
            &throbber_chars[*frame].to_string()
        };

        println!(
            "{} {}: {}",
            indicator.bright_cyan(),
            self.label.bright_blue().bold(),
            format!("{:.1} Mbps", current).bright_green().bold()
        );
        println!();
        println!(
            "{} {}",
            "Peak:".bright_cyan(),
            format!("{:.1} Mbps", peak).bright_cyan()
        );
        println!();

        // Create filled area graph
        let max_val = if history.is_empty() {
            1.0
        } else {
            history.iter().cloned().fold(0.0f64, f64::max).max(1.0)
        };
        let width = 80; // Full terminal width
        let height = 8; // Height of graph

        // Generate graph lines with filled area
        for row in (0..height).rev() {
            let threshold = (row as f64 / height as f64) * max_val;
            print!("│");

            if history.is_empty() {
                // Show empty graph
                for _ in 0..width {
                    print!(" ");
                }
            } else {
                // Take the most recent samples up to width
                let samples_to_show = history.len().min(width);
                let start_idx = history.len().saturating_sub(width);

                for i in start_idx..history.len() {
                    let speed = history[i];
                    let char = if speed >= threshold { "█" } else { " " };
                    print!("{}", char.bright_yellow());
                }

                // Fill remaining space if we have fewer samples than width
                for _ in 0..(width - samples_to_show) {
                    print!(" ");
                }
            }

            println!();
        }

        // Bottom axis
        print!("└");
        for _ in 0..width {
            print!("─");
        }
        println!();

        std::io::stdout().flush()?;
        Ok(())
    }

    pub async fn render_live_update(&self) -> io::Result<()> {
        let history = self.speed_history.read().await;
        let current = self.current_speed.read().await;
        let peak = self.peak_speed.read().await;
        let is_final = self.is_final.read().await;
        let frame = self.throbber_frame.read().await;

        // Move cursor up 13 lines and clear them
        print!("\x1B[13A"); // Move up 13 lines
        for _ in 0..13 {
            print!("\x1B[2K"); // Clear line
            print!("\x1B[1B"); // Move down 1 line
        }
        print!("\x1B[13A"); // Move back up to start position

        // Display speed with throbber or checkmark
        let throbber_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let indicator = if *is_final {
            "✓"
        } else {
            &throbber_chars[*frame].to_string()
        };

        println!(
            "{} {}: {}",
            indicator.bright_cyan(),
            self.label.bright_blue().bold(),
            format!("{:.1} Mbps", current).bright_green().bold()
        );
        println!();
        println!(
            "{} {}",
            "Peak:".bright_cyan(),
            format!("{:.1} Mbps", peak).bright_cyan()
        );
        println!();

        // Create filled area graph
        let max_val = if history.is_empty() {
            1.0
        } else {
            history.iter().cloned().fold(0.0f64, f64::max).max(1.0)
        };
        let width = 80;
        let height = 8;

        // Generate graph lines with filled area
        for row in (0..height).rev() {
            let threshold = (row as f64 / height as f64) * max_val;
            print!("│");

            if history.is_empty() {
                for _ in 0..width {
                    print!(" ");
                }
            } else {
                let samples_to_show = history.len().min(width);
                let start_idx = history.len().saturating_sub(width);

                for i in start_idx..history.len() {
                    let speed = history[i];
                    let char = if speed >= threshold { "█" } else { " " };
                    print!("{}", char.bright_yellow());
                }

                for _ in 0..(width - samples_to_show) {
                    print!(" ");
                }
            }

            println!();
        }

        // Bottom axis
        print!("└");
        for _ in 0..width {
            print!("─");
        }
        println!();

        std::io::stdout().flush()?;
        Ok(())
    }
}

pub struct UI {
    term: Term,
    multi_progress: MultiProgress,
}

impl UI {
    pub fn new(_config: TestConfig) -> Self {
        Self {
            term: Term::stdout(),
            multi_progress: MultiProgress::new(),
        }
    }

    pub fn clear_screen(&self) -> io::Result<()> {
        self.term.clear_screen()
    }

    pub fn show_welcome_banner(&self) -> io::Result<()> {
        self.term.clear_screen()?;

        let banner = r#"
 _   _ ______ _______ _____  _    _ _   _ _   _ ______ _____
| \ | |  ____|__   __|  __ \| |  | | \ | | \ | |  ____|  __ \
|  \| | |__     | |  | |__) | |  | |  \| |  \| | |__  | |__) |
| . ` |  __|    | |  |  _  /| |  | | . ` | . ` |  __| |  _  /
| |\  | |____   | |  | | \ \| |__| | |\  | |\  | |____| | \ \
|_| \_|______|  |_|  |_|  \_\\____/|_| \_|_| \_|______|_|  \_\

        "#;

        println!("{}", banner.bright_cyan());

        println!("{}", "SYSTEM STATUS".bright_magenta().bold());
        println!("{}", "⟨⟨⟨ NEURAL INTERFACE: ONLINE ⟩⟩⟩".bright_green());
        println!("{}", "⟨⟨⟨ NETWORK SCANNER: INITIALIZED ⟩⟩⟩".bright_green());
        println!("{}", "⟨⟨⟨ QUANTUM DIAGNOSTICS: READY ⟩⟩⟩".bright_green());
        println!();
        println!(
            "{}",
            ">>> JACK IN AND ANALYZE YOUR DIGITAL HIGHWAY <<<"
                .bright_yellow()
                .bold()
        );
        println!(
            "{}",
            ">>> DATA FLOWS | PACKET STREAMS | NEURAL PATHS <<<".bright_blue()
        );
        println!();

        Ok(())
    }

    pub fn create_progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(len));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} {msg} [{bar:40.cyan/blue}] {percent}% [{elapsed_precise}]",
                )
                .unwrap()
                .progress_chars("━━╸─"),
        );
        pb.set_message(message.to_string());
        pb
    }

    pub fn create_speed_test_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_download_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_upload_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_blue} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_ping_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_magenta} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn create_cyberpunk_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_dna_helix_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_rocket_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_yellow} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_wave_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_network_scanner_bar(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_yellow} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn create_pacman_spinner(&self, message: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.bright_yellow} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn show_section_header(&self, title: &str) -> io::Result<()> {
        println!();
        println!(
            "{}",
            format!(">>> {} <<<", title.to_uppercase())
                .bright_magenta()
                .bold()
        );
        Ok(())
    }

    pub fn show_error(&self, message: &str) -> io::Result<()> {
        println!("{} {}", "ERROR:".bright_red().bold(), message.bright_red());
        Ok(())
    }

    pub fn show_info(&self, message: &str) -> io::Result<()> {
        println!("{} {}", "INFO:".bright_blue().bold(), message.bright_blue());
        Ok(())
    }

    pub fn show_typing_effect(&self, text: &str) -> io::Result<()> {
        for char in text.chars() {
            print!("{}", char.to_string().bright_green());
            std::io::stdout().flush()?;
            thread::sleep(Duration::from_millis(50));
        }
        println!();
        Ok(())
    }

    pub fn show_matrix_effect(&self, lines: usize) -> io::Result<()> {
        let matrix_chars = ["0", "1", "⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"];

        for _ in 0..lines {
            print!("{}", "█".bright_green());
            for _ in 0..60 {
                let idx = rand::rng().random_range(0..matrix_chars.len());
                print!("{}", matrix_chars[idx].bright_green());
                thread::sleep(Duration::from_millis(20));
            }
            println!();
        }
        Ok(())
    }

    pub fn show_pulse_text(&self, text: &str, pulses: usize) -> io::Result<()> {
        for _ in 0..pulses {
            print!("\r{}", text.bright_cyan().bold());
            std::io::stdout().flush()?;
            thread::sleep(Duration::from_millis(500));

            print!("\r{}", text.bright_blue());
            std::io::stdout().flush()?;
            thread::sleep(Duration::from_millis(500));
        }
        println!();
        Ok(())
    }

    pub fn show_connection_establishing(&self) -> io::Result<()> {
        let steps = [
            "⟨⟨⟨ INITIALIZING NEURAL INTERFACE ⟩⟩⟩",
            "⟨⟨⟨ SCANNING NETWORK TOPOLOGY ⟩⟩⟩",
            "⟨⟨⟨ ESTABLISHING QUANTUM TUNNEL ⟩⟩⟩",
            "⟨⟨⟨ CALIBRATING DATA STREAMS ⟩⟩⟩",
            "⟨⟨⟨ CONNECTION ESTABLISHED ⟩⟩⟩",
        ];

        for step in steps.iter() {
            println!("{}", step.bright_magenta());
            thread::sleep(Duration::from_millis(800));
        }
        println!();
        Ok(())
    }

    pub fn create_bandwidth_monitor(&self, title: &str, label: &str) -> BandwidthMonitor {
        BandwidthMonitor::new(title.to_string(), label.to_string())
    }
}
