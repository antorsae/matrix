//! Matrix-style terminal rain animation.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::io::{stdout, IsTerminal, Write};
use std::time::{Duration, Instant};

// Character sets
const KATAKANA: &str = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン";
const ASCII_CHARS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()_+-=[]{}|;:',.<>?/";

// Speed tiers (cells per second) - 1x/2x/3x for depth perception
const SPEED_TIERS: [f64; 3] = [8.0, 16.0, 24.0];

// Visual parameters
const TRAIL_LENGTH_MIN: usize = 8;
const TRAIL_LENGTH_MAX: usize = 25;
const COLUMN_DENSITY: f64 = 0.65; // 60-70%
const MUTATION_RATE: f64 = 0.10; // 10% per frame

// Timing
const TARGET_FPS: u64 = 30;
const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

// Terminal requirements
const MIN_WIDTH: u16 = 20;
const MIN_HEIGHT: u16 = 10;

// Color definitions for gradient
const COLOR_HEAD: Color = Color::Rgb { r: 255, g: 255, b: 255 }; // Bright white
const COLOR_BRIGHT: Color = Color::Rgb { r: 0, g: 255, b: 0 };   // Bright green
const COLOR_MEDIUM: Color = Color::Rgb { r: 0, g: 215, b: 0 };   // Medium green
const COLOR_DIM: Color = Color::Rgb { r: 0, g: 175, b: 0 };      // Dim green

/// All characters combined into a vector for random selection
fn get_char_set() -> Vec<char> {
    KATAKANA.chars().chain(ASCII_CHARS.chars()).collect()
}

/// Represents a single falling rain column.
struct Column {
    x: u16,
    screen_height: u16,
    y_head: f64,
    speed: f64,
    trail_length: usize,
    characters: Vec<char>,
    active: bool,
}

impl Column {
    fn new(x: u16, screen_height: u16, char_set: &[char]) -> Self {
        let mut rng = rand::thread_rng();
        let speed_tier = rng.gen_range(0..3);
        let trail_length = rng.gen_range(TRAIL_LENGTH_MIN..=TRAIL_LENGTH_MAX);
        let characters: Vec<char> = (0..trail_length)
            .map(|_| char_set[rng.gen_range(0..char_set.len())])
            .collect();

        Column {
            x,
            screen_height,
            y_head: 0.0,
            speed: SPEED_TIERS[speed_tier],
            trail_length,
            characters,
            active: true,
        }
    }

    /// Move column down by delta_time * speed.
    fn update(&mut self, delta_time: f64) {
        self.y_head += self.speed * delta_time;

        // Check if fully off screen (head + trail length past bottom)
        if self.y_head - self.trail_length as f64 > self.screen_height as f64 {
            self.active = false;
        }
    }

    /// Randomly mutate characters based on MUTATION_RATE.
    fn mutate(&mut self, char_set: &[char]) {
        let mut rng = rand::thread_rng();
        for i in 0..self.characters.len() {
            if rng.gen::<f64>() < MUTATION_RATE {
                self.characters[i] = char_set[rng.gen_range(0..char_set.len())];
            }
        }
    }

    /// Return list of (x, y, char, color) for visible cells.
    fn get_visible_cells(&self) -> Vec<(u16, u16, char, Color)> {
        let mut cells = Vec::new();
        let head_y = self.y_head as i32;

        for i in 0..self.trail_length {
            let y = head_y - i as i32;
            if y >= 0 && y < self.screen_height as i32 {
                let char_idx = i % self.characters.len();
                let color = self.get_color_for_position(i);
                cells.push((self.x, y as u16, self.characters[char_idx], color));
            }
        }

        cells
    }

    /// Determine color based on position in trail.
    fn get_color_for_position(&self, pos: usize) -> Color {
        if pos == 0 {
            return COLOR_HEAD; // White head
        }
        let ratio = pos as f64 / self.trail_length as f64;
        if ratio < 0.33 {
            COLOR_BRIGHT // Bright green
        } else if ratio < 0.66 {
            COLOR_MEDIUM // Medium green
        } else {
            COLOR_DIM // Dim green
        }
    }
}

/// Main application controller.
struct MatrixRain {
    height: u16,
    width: u16,
    columns: Vec<Column>,
    column_slots: HashSet<u16>,
    running: bool,
    prev_frame: HashMap<(u16, u16), (char, Color)>,
    char_set: Vec<char>,
}

impl MatrixRain {
    fn new(width: u16, height: u16) -> Self {
        MatrixRain {
            height,
            width,
            columns: Vec::new(),
            column_slots: HashSet::new(),
            running: true,
            prev_frame: HashMap::new(),
            char_set: get_char_set(),
        }
    }

    /// Spawn columns to achieve target density immediately.
    fn spawn_initial_columns(&mut self) {
        let mut rng = rand::thread_rng();
        let target_count = (self.width as f64 * COLUMN_DENSITY) as usize;
        let mut available_slots: Vec<u16> = (0..self.width).collect();

        // Shuffle available slots
        for i in (1..available_slots.len()).rev() {
            let j = rng.gen_range(0..=i);
            available_slots.swap(i, j);
        }

        for &x in available_slots.iter().take(target_count) {
            let mut col = Column::new(x, self.height, &self.char_set);
            // Randomize starting position for varied entry
            col.y_head = rng.gen_range(-(col.trail_length as f64)..self.height as f64);
            self.columns.push(col);
            self.column_slots.insert(x);
        }
    }

    /// Spawn a new column at random available position.
    fn spawn_new_column(&mut self) {
        let mut rng = rand::thread_rng();
        let available: Vec<u16> = (0..self.width)
            .filter(|x| !self.column_slots.contains(x))
            .collect();

        if !available.is_empty() {
            let x = available[rng.gen_range(0..available.len())];
            let mut col = Column::new(x, self.height, &self.char_set);
            col.y_head = 0.0; // Start from top
            self.columns.push(col);
            self.column_slots.insert(x);
        }
    }

    /// Update all columns and manage spawning.
    fn update(&mut self, delta_time: f64) {
        // Update existing columns
        for col in &mut self.columns {
            col.update(delta_time);
            col.mutate(&self.char_set);
        }

        // Remove inactive columns and free their slots
        let inactive_x: Vec<u16> = self
            .columns
            .iter()
            .filter(|c| !c.active)
            .map(|c| c.x)
            .collect();

        for x in inactive_x {
            self.column_slots.remove(&x);
        }
        self.columns.retain(|c| c.active);

        // Spawn replacements to maintain density
        let target_count = (self.width as f64 * COLUMN_DENSITY) as usize;
        while self.column_slots.len() < target_count {
            self.spawn_new_column();
        }
    }

    /// Render frame with differential updates.
    fn render(&mut self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        // Build current frame state
        let mut current_frame: HashMap<(u16, u16), (char, Color)> = HashMap::new();

        for col in &self.columns {
            for (x, y, ch, color) in col.get_visible_cells() {
                // Avoid bottom-right corner (terminal quirk)
                if x < self.width && y < self.height {
                    if x == self.width - 1 && y == self.height - 1 {
                        continue;
                    }
                    current_frame.insert((x, y), (ch, color));
                }
            }
        }

        // Clear cells that were drawn last frame but not this frame
        for pos in self.prev_frame.keys() {
            if !current_frame.contains_key(pos) {
                let (x, y) = *pos;
                execute!(stdout, MoveTo(x, y), Print(" "))?;
            }
        }

        // Draw new/changed cells
        for (pos, (ch, color)) in &current_frame {
            if !self.prev_frame.contains_key(pos) || self.prev_frame.get(pos) != Some(&(*ch, *color))
            {
                let (x, y) = *pos;
                execute!(
                    stdout,
                    MoveTo(x, y),
                    SetForegroundColor(*color),
                    Print(ch)
                )?;
            }
        }

        self.prev_frame = current_frame;
        stdout.flush()?;
        Ok(())
    }

    /// Handle keyboard input.
    fn handle_input(&mut self) -> std::io::Result<()> {
        // Poll for events with zero timeout (non-blocking)
        if poll(Duration::ZERO)? {
            if let Event::Key(key_event) = read()? {
                match key_event.code {
                    KeyCode::Char('q') => self.running = false,
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.running = false
                    }
                    KeyCode::Esc => self.running = false,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Main loop with frame pacing.
    fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout();

        // Initialize
        self.spawn_initial_columns();

        let mut last_time = Instant::now();

        while self.running {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(last_time).as_secs_f64();

            // Check for quit key
            self.handle_input()?;

            // Update simulation
            self.update(delta_time);

            // Render
            self.render(&mut stdout)?;

            // Frame pacing
            let elapsed = current_time.elapsed();
            if elapsed < FRAME_TIME {
                std::thread::sleep(FRAME_TIME - elapsed);
            }

            last_time = current_time;
        }

        Ok(())
    }
}

fn main() {
    // Check TTY requirement
    if !stdout().is_terminal() {
        eprintln!("Error: Requires TTY.");
        std::process::exit(1);
    }

    // Get terminal size and validate
    let (width, height) = match terminal::size() {
        Ok(size) => size,
        Err(e) => {
            eprintln!(
                "Error: Cannot get terminal size. \
                 Ensure you're running in a supported terminal.\n\
                 Details: {}",
                e
            );
            std::process::exit(1);
        }
    };

    if width < MIN_WIDTH || height < MIN_HEIGHT {
        eprintln!(
            "Error: Terminal too small: {}x{}. Minimum size: {}x{}.",
            width, height, MIN_WIDTH, MIN_HEIGHT
        );
        std::process::exit(1);
    }

    // Setup terminal
    if let Err(e) = setup_terminal() {
        eprintln!("Error: Failed to setup terminal: {}", e);
        std::process::exit(1);
    }

    // Run the animation
    let mut app = MatrixRain::new(width, height);
    let result = app.run();

    // Cleanup terminal (always try to restore state)
    let _ = cleanup_terminal();

    // Handle any errors from the main loop
    if let Err(e) = result {
        eprintln!("Error during execution: {}", e);
        std::process::exit(1);
    }
}

fn setup_terminal() -> std::io::Result<()> {
    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        Hide,
        Clear(ClearType::All)
    )?;
    Ok(())
}

fn cleanup_terminal() -> std::io::Result<()> {
    execute!(stdout(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
