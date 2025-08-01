use chrono::{DateTime, Local};
use eframe::egui;
use egui::{Color32, RichText, Rounding, Stroke, Vec2};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Represents the possible states of the timer
#[derive(Debug, Clone, PartialEq)]
enum TimerState {
    Ready,      // Timer is idle and ready to start
    Preparing,  // User is holding space to prepare
    Running,    // Timer is actively counting
    Stopped,    // Timer has stopped after a solve
}

// Defines cube solving events, either standard or custom
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum CubeEvent {
    Standard(StandardEvent),
    Custom(String),
}

// Standard cube events as per WCA regulations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum StandardEvent {
    Cube3x3, Cube2x2, Cube4x4, Cube5x5, Cube6x6, Cube7x7,
    Pyraminx, Megaminx, Skewb, Square1, Clock,
    OneHanded, Blindfolded, FeetSolving,
}

impl std::fmt::Display for StandardEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StandardEvent::Cube3x3 => write!(f, "3x3x3"),
            StandardEvent::Cube2x2 => write!(f, "2x2x2"),
            StandardEvent::Cube4x4 => write!(f, "4x4x4"),
            StandardEvent::Cube5x5 => write!(f, "5x5x5"),
            StandardEvent::Cube6x6 => write!(f, "6x6x6"),
            StandardEvent::Cube7x7 => write!(f, "7x7x7"),
            StandardEvent::Pyraminx => write!(f, "Pyraminx"),
            StandardEvent::Megaminx => write!(f, "Megaminx"),
            StandardEvent::Skewb => write!(f, "Skewb"),
            StandardEvent::Square1 => write!(f, "Square-1"),
            StandardEvent::Clock => write!(f, "Clock"),
            StandardEvent::OneHanded => write!(f, "3x3 OH"),
            StandardEvent::Blindfolded => write!(f, "3x3 BLD"),
            StandardEvent::FeetSolving => write!(f, "3x3 Feet"),
        }
    }
}

impl std::fmt::Display for CubeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CubeEvent::Standard(event) => write!(f, "{}", event),
            CubeEvent::Custom(name) => write!(f, "{}", name),
        }
    }
}

// Stores a single solve record with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeRecord {
    time: Duration,          // Duration of the solve
    event: CubeEvent,       // Event type (e.g., 3x3x3, Pyraminx)
    scramble: String,       // Scramble used for the solve
    timestamp: DateTime<Local>, // Time and date of the solve
    penalty: Option<Penalty>,   // Any penalties applied (e.g., +2, DNF)
    comment: String,        // User comments for the solve
}

// Represents penalties that can be applied to a solve
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Penalty {
    Plus2,  // +2 second penalty
    DNF,    // Did Not Finish
}

// Aggregates statistical data for solves
#[derive(Debug, Clone)]
struct Statistics {
    best: Option<Duration>,         // Fastest solve time
    worst: Option<Duration>,        // Slowest solve time
    current_ao5: Option<Duration>,  // Average of last 5 solves
    current_ao12: Option<Duration>, // Average of last 12 solves
    current_ao100: Option<Duration>, // Average of last 100 solves
    mean: Option<Duration>,         // Mean of all solves
}

// Defines a custom event with user-specified parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomEvent {
    name: String,           // Name of the custom event
    scramble_length: usize, // Length of the scramble
    moves: Vec<String>,    // Available moves for scrambling
}

// Customizable UI theme with color and style settings
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Theme {
    background: [u8; 3],        // Background color
    surface: [u8; 3],           // Surface color for panels
    surface_variant: [u8; 3],   // Variant surface color for hover effects
    text_primary: [u8; 3],      // Primary text color
    text_secondary: [u8; 3],    // Secondary text color
    timer_ready: [u8; 3],       // Timer color when ready
    timer_preparing: [u8; 3],   // Timer color when preparing
    timer_running: [u8; 3],     // Timer color when running
    timer_stopped: [u8; 3],     // Timer color when stopped
    accent_primary: [u8; 3],    // Primary accent color
    accent_secondary: [u8; 3],  // Secondary accent color
    success: [u8; 3],           // Success color (e.g., best time)
    warning: [u8; 3],           // Warning color (e.g., +2 penalty)
    error: [u8; 3],             // Error color (e.g., DNF)
    corner_radius: f32,         // Corner radius for UI elements
    font_size_small: f32,       // Small font size
    font_size_normal: f32,       // Normal font size
    font_size_large: f32,       // Large font size
    font_size_timer: f32,       // Timer font size
    enable_animations: bool,    // Enable/disable animations
    animation_speed: f32,       // Animation speed multiplier
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: [25, 25, 30],
            surface: [35, 35, 42],
            surface_variant: [45, 45, 55],
            text_primary: [240, 240, 245],
            text_secondary: [160, 160, 170],
            timer_ready: [76, 175, 80],
            timer_preparing: [255, 193, 7],
            timer_running: [33, 150, 243],
            timer_stopped: [244, 67, 54],
            accent_primary: [103, 58, 183],
            accent_secondary: [63, 81, 181],
            success: [76, 175, 80],
            warning: [255, 193, 7],
            error: [244, 67, 54],
            corner_radius: 12.0,
            font_size_small: 12.0,
            font_size_normal: 14.0,
            font_size_large: 18.0,
            font_size_timer: 88.0,
            enable_animations: true,
            animation_speed: 1.0,
        }
    }
}

impl Theme {
    fn bg_color(&self) -> Color32 {
        Color32::from_rgb(self.background[0], self.background[1], self.background[2])
    }

    fn surface_color(&self) -> Color32 {
        Color32::from_rgb(self.surface[0], self.surface[1], self.surface[2])
    }

    fn surface_variant_color(&self) -> Color32 {
        Color32::from_rgb(self.surface_variant[0], self.surface_variant[1], self.surface_variant[2])
    }

    fn text_primary_color(&self) -> Color32 {
        Color32::from_rgb(self.text_primary[0], self.text_primary[1], self.text_primary[2])
    }

    fn text_secondary_color(&self) -> Color32 {
        Color32::from_rgb(self.text_secondary[0], self.text_secondary[1], self.text_secondary[2])
    }

    fn accent_primary_color(&self) -> Color32 {
        Color32::from_rgb(self.accent_primary[0], self.accent_primary[1], self.accent_primary[2])
    }

    fn accent_secondary_color(&self) -> Color32 {
        Color32::from_rgb(self.accent_secondary[0], self.accent_secondary[1], self.accent_secondary[2])
    }

    fn timer_color(&self, state: &TimerState) -> Color32 {
        match state {
            TimerState::Ready => Color32::from_rgb(self.timer_ready[0], self.timer_ready[1], self.timer_ready[2]),
            TimerState::Preparing => Color32::from_rgb(self.timer_preparing[0], self.timer_preparing[1], self.timer_preparing[2]),
            TimerState::Running => Color32::from_rgb(self.timer_running[0], self.timer_running[1], self.timer_running[2]),
            TimerState::Stopped => Color32::from_rgb(self.timer_stopped[0], self.timer_stopped[1], self.timer_stopped[2]),
        }
    }

    fn success_color(&self) -> Color32 {
        Color32::from_rgb(self.success[0], self.success[1], self.success[2])
    }

    fn warning_color(&self) -> Color32 {
        Color32::from_rgb(self.warning[0], self.warning[1], self.warning[2])
    }

    fn error_color(&self) -> Color32 {
        Color32::from_rgb(self.error[0], self.error[1], self.error[2])
    }

    fn rounding(&self) -> Rounding {
        Rounding::same(self.corner_radius)
    }
}

// Manages UI state with serializable fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UIState {
    show_times_panel: bool,         // Visibility of the times panel
    show_settings: bool,           // Visibility of the settings window
    show_statistics: bool,         // Visibility of the statistics window
    times_panel_width: f32,        // Width of the times panel
    #[serde(skip)]
    selected_time_index: Option<usize>, // Index of the selected time record
    #[serde(skip)]
    editing_comment_index: Option<usize>, // Index of the time being commented
    #[serde(skip)]
    comment_text: String,          // Text for editing comments
    #[serde(skip)]
    confirm_delete_index: Option<usize>, // Index of the time to delete
    #[serde(skip)]
    show_exit_popup: bool,         // Visibility of the exit confirmation popup
    is_first_launch: bool,         // Flag for showing the welcome message
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            show_times_panel: true,
            show_settings: false,
            show_statistics: false,
            times_panel_width: 300.0,
            selected_time_index: None,
            editing_comment_index: None,
            comment_text: String::new(),
            confirm_delete_index: None,
            show_exit_popup: false,
            is_first_launch: true,
        }
    }
}

// Main application struct for the cube timer
struct CubeTimer {
    state: TimerState,              // Current state of the timer
    start_time: Option<Instant>,    // Start time of the current solve
    current_time: Duration,         // Current running time
    last_time: Option<Duration>,    // Last recorded solve time
    current_event: CubeEvent,       // Currently selected event
    available_events: Vec<CubeEvent>, // List of available events
    custom_events: HashMap<String, CustomEvent>, // Custom event definitions
    current_scramble: String,       // Current scramble
    records: Vec<TimeRecord>,       // List of all solve records
    statistics: Statistics,         // Statistical data for solves
    theme: Theme,                   // UI theme settings
    ui_state: UIState,             // UI state settings
    new_custom_event_name: String,  // Name for new custom event
    new_custom_moves: String,      // Moves for new custom event
    space_pressed: bool,            // Space key state
    space_hold_start: Option<Instant>, // Time when space key was pressed
    key_preparation_time: Duration, // Minimum hold time to start timer
    timer_scale: f32,              // Current timer scale for animation
    target_timer_scale: f32,       // Target timer scale for animation
    last_save_time: Instant,
}

impl Default for CubeTimer {
    fn default() -> Self {
        let available_events = vec![
            CubeEvent::Standard(StandardEvent::Cube3x3),
            CubeEvent::Standard(StandardEvent::Cube2x2),
            CubeEvent::Standard(StandardEvent::Cube4x4),
            CubeEvent::Standard(StandardEvent::Cube5x5),
            CubeEvent::Standard(StandardEvent::Cube6x6),
            CubeEvent::Standard(StandardEvent::Cube7x7),
            CubeEvent::Standard(StandardEvent::Pyraminx),
            CubeEvent::Standard(StandardEvent::Megaminx),
            CubeEvent::Standard(StandardEvent::Skewb),
            CubeEvent::Standard(StandardEvent::Square1),
            CubeEvent::Standard(StandardEvent::Clock),
            CubeEvent::Standard(StandardEvent::OneHanded),
            CubeEvent::Standard(StandardEvent::Blindfolded),
            CubeEvent::Standard(StandardEvent::FeetSolving),

        ];

        let current_event = available_events[0].clone();
        let current_scramble = Self::generate_scramble(&current_event);

        Self {
            state: TimerState::Ready,
            start_time: None,
            current_time: Duration::ZERO,
            last_time: None,
            current_event,
            available_events,
            custom_events: HashMap::new(),
            current_scramble,
            records: Vec::new(),
            statistics: Statistics {
                best: None,
                worst: None,
                current_ao5: None,
                current_ao12: None,
                current_ao100: None,
                mean: None,
            },
            theme: Theme::default(),
            ui_state: UIState::default(),
            new_custom_event_name: String::new(),
            new_custom_moves: String::new(),
            space_pressed: false,
            space_hold_start: None,
            key_preparation_time: Duration::from_millis(300),
            timer_scale: 1.0,
            target_timer_scale: 1.0,
            last_save_time: Instant::now(),
        }
    }
}

impl CubeTimer {
    // Initializes the application with loaded data
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.load_data();
        app.last_save_time = Instant::now();
        app
    }

    // Generates a scramble for the given event
    fn generate_scramble(event: &CubeEvent) -> String {
        let mut rng = rand::thread_rng();

        match event {
            CubeEvent::Standard(StandardEvent::Cube3x3) => {
                Self::generate_cube_scramble(&mut rng, &["R", "U", "F", "L", "D", "B"], 20)
            },
            CubeEvent::Standard(StandardEvent::Cube2x2) => {
                Self::generate_cube_scramble(&mut rng, &["R", "U", "F"], 9)
            },
            CubeEvent::Standard(StandardEvent::Pyraminx) => {
                Self::generate_pyraminx_scramble(&mut rng)
            },
            CubeEvent::Custom(name) => {
                format!("Custom scramble for {}", name)
            },
            _ => {
                Self::generate_cube_scramble(&mut rng, &["R", "U", "F", "L", "D", "B"], 15)
            }
        }
    }

    // Generates a cube scramble with specified moves and length
    fn generate_cube_scramble(rng: &mut impl Rng, moves: &[&str], length: usize) -> String {
        let modifiers = ["", "'", "2"];
        let mut scramble = Vec::new();

        for _ in 0..length {
            let move_idx = rng.gen_range(0..moves.len());
            let mod_idx = rng.gen_range(0..modifiers.len());
            scramble.push(format!("{}{}", moves[move_idx], modifiers[mod_idx]));
        }
        scramble.join(" ")
    }

    // Generates a Pyraminx scramble
    fn generate_pyraminx_scramble(rng: &mut impl Rng) -> String {
        let moves = ["R", "U", "L", "B"];
        let modifiers = ["", "'"];
        let mut scramble = Vec::new();

        for _ in 0..10 {
            let move_idx = rng.gen_range(0..moves.len());
            let mod_idx = rng.gen_range(0..modifiers.len());
            scramble.push(format!("{}{}", moves[move_idx], modifiers[mod_idx]));
        }
        scramble.join(" ")
    }

    // Updates statistics based on recorded times
    fn calculate_statistics(&mut self) {
        let current_event_times: Vec<Duration> = self.records
            .iter()
            .filter(|r| r.event == self.current_event && r.penalty.is_none())
            .map(|r| r.time)
            .collect();

        if current_event_times.is_empty() {
            self.statistics = Statistics {
                best: None,
                worst: None,
                current_ao5: None,
                current_ao12: None,
                current_ao100: None,
                mean: None,
            };
            return;
        }

        self.calculate_basic_stats(&current_event_times);
        self.calculate_averages(&current_event_times);
    }

    // Calculates basic statistics (best, worst, mean)
    fn calculate_basic_stats(&mut self, times: &[Duration]) {
        self.statistics.best = times.iter().min().copied();
        self.statistics.worst = times.iter().max().copied();

        let sum: Duration = times.iter().sum();
        self.statistics.mean = Some(sum / times.len() as u32);
    }

    // Calculates average of 5, 12, and 100 solves
    fn calculate_averages(&mut self, times: &[Duration]) {
        if times.len() >= 5 {
            let last_5: Vec<Duration> = times.iter().rev().take(5).cloned().collect();
            self.statistics.current_ao5 = Self::calculate_average(&last_5);
        }

        if times.len() >= 12 {
            let last_12: Vec<Duration> = times.iter().rev().take(12).cloned().collect();
            self.statistics.current_ao12 = Self::calculate_average(&last_12);
        }

        if times.len() >= 100 {
            let last_100: Vec<Duration> = times.iter().rev().take(100).cloned().collect();
            self.statistics.current_ao100 = Self::calculate_average(&last_100);
        }
    }

    // Calculates the trimmed mean for a set of times
    fn calculate_average(times: &[Duration]) -> Option<Duration> {
        if times.len() < 5 {
            return None;
        }

        let mut sorted = times.to_vec();
        sorted.sort();

        let remove_count = (times.len() as f32 * 0.05).ceil() as usize;
        if remove_count * 2 >= times.len() {
            return None;
        }

        let trimmed = &sorted[remove_count..sorted.len() - remove_count];
        let sum: Duration = trimmed.iter().sum();
        Some(sum / trimmed.len() as u32)
    }

    // Formats a duration into a readable time string
    fn format_time(duration: Duration) -> String {
        let total_millis = duration.as_millis();
        let minutes = total_millis / 60000;
        let seconds = (total_millis % 60000) / 1000;
        let millis = total_millis % 1000;

        if minutes > 0 {
            format!("{}:{:02}.{:03}", minutes, seconds, millis)
        } else {
            format!("{}.{:03}", seconds, millis)
        }
    }

    // Saves all application data to disk
    fn save_data(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("cube-timer");
            if std::fs::create_dir_all(&app_dir).is_err() {
                return;
            }

            self.save_records(&app_dir);
            self.save_theme(&app_dir);
            self.save_custom_events(&app_dir);
            self.save_ui_state(&app_dir);
        }
    }

    // Saves solve records to disk
    fn save_records(&self, app_dir: &std::path::Path) {
        if let Ok(json) = serde_json::to_string(&self.records) {
            let _ = std::fs::write(app_dir.join("records.json"), json);
        }
    }

    // Saves theme settings to disk
    fn save_theme(&self, app_dir: &std::path::Path) {
        if let Ok(json) = serde_json::to_string(&self.theme) {
            let _ = std::fs::write(app_dir.join("theme.json"), json);
        }
    }

    // Saves custom events to disk
    fn save_custom_events(&self, app_dir: &std::path::Path) {
        if let Ok(json) = serde_json::to_string(&self.custom_events) {
            let _ = std::fs::write(app_dir.join("custom_events.json"), json);
        }
    }

    // Saves UI state to disk
    fn save_ui_state(&self, app_dir: &std::path::Path) {
        if let Ok(json) = serde_json::to_string(&self.ui_state) {
            let _ = std::fs::write(app_dir.join("ui_state.json"), json);
        }
    }

    // Loads all application data from disk
    fn load_data(&mut self) {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir,
            None => return,
        };

        let app_dir = config_dir.join("cube-timer");

        self.load_records(&app_dir);
        self.load_theme(&app_dir);
        self.load_custom_events(&app_dir);
        self.load_ui_state(&app_dir);
        self.calculate_statistics();
    }

    // Loads solve records from disk
    fn load_records(&mut self, app_dir: &std::path::Path) {
        if let Ok(data) = std::fs::read_to_string(app_dir.join("records.json")) {
            if let Ok(records) = serde_json::from_str(&data) {
                self.records = records;
            }
        }
    }

    // Loads theme settings from disk
    fn load_theme(&mut self, app_dir: &std::path::Path) {
        if let Ok(data) = std::fs::read_to_string(app_dir.join("theme.json")) {
            if let Ok(theme) = serde_json::from_str(&data) {
                self.theme = theme;
            }
        }
    }

    // Loads custom events from disk
    fn load_custom_events(&mut self, app_dir: &std::path::Path) {
        if let Ok(data) = std::fs::read_to_string(app_dir.join("custom_events.json")) {
            if let Ok(custom_events) = serde_json::from_str(&data) {
                self.custom_events = custom_events;
                for name in self.custom_events.keys() {
                    let custom_event = CubeEvent::Custom(name.clone());
                    if !self.available_events.contains(&custom_event) {
                        self.available_events.push(custom_event);
                    }
                }
            }
        }
    }

    // Loads UI state from disk
    fn load_ui_state(&mut self, app_dir: &std::path::Path) {
        if let Ok(data) = std::fs::read_to_string(app_dir.join("ui_state.json")) {
            if let Ok(ui_state) = serde_json::from_str(&data) {
                self.ui_state = ui_state;
            }
        }
    }

    // Handles space key input for timer control
    fn handle_space_key(&mut self, pressed: bool) {
        let now = Instant::now();

        if pressed && !self.space_pressed {
            self.handle_space_press(now);
        } else if !pressed && self.space_pressed {
            self.handle_space_release(now);
        }
    }

    // Processes space key press
    fn handle_space_press(&mut self, now: Instant) {
        self.space_pressed = true;
        self.space_hold_start = Some(now);

        match self.state {
            TimerState::Ready => {
                self.state = TimerState::Preparing;
                self.target_timer_scale = 0.95;
            }
            TimerState::Running => {
                self.stop_timer(now);
            }
            TimerState::Stopped => {
                self.state = TimerState::Preparing;
                self.target_timer_scale = 0.95;
            }
            _ => {}
        }
    }

    // Processes space key release
    fn handle_space_release(&mut self, now: Instant) {
        self.space_pressed = false;

        match self.state {
            TimerState::Preparing => {
                self.try_start_timer(now);
            }
            TimerState::Stopped => {
                self.state = TimerState::Ready;
            }
            _ => {}
        }

        self.space_hold_start = None;
        self.target_timer_scale = 1.0;
    }

    // Stops the timer and records the time
    fn stop_timer(&mut self, now: Instant) {
        if let Some(start_time) = self.start_time {
            self.current_time = now.duration_since(start_time);
            self.last_time = Some(self.current_time);

            self.save_time_record();
            self.generate_new_scramble();
        }
        self.state = TimerState::Stopped;
        self.start_time = None;
        self.target_timer_scale = 1.0;
    }

    // Saves a new time record
    fn save_time_record(&mut self) {
        let record = TimeRecord {
            time: self.current_time,
            event: self.current_event.clone(),
            scramble: self.current_scramble.clone(),
            timestamp: Local::now(),
            penalty: None,
            comment: String::new(),
        };

        self.records.push(record);
        self.calculate_statistics();
        self.save_data()
    }

    // Generates a new scramble for the current event
    fn generate_new_scramble(&mut self) {
        self.current_scramble = Self::generate_scramble(&self.current_event);
    }

    // Attempts to start the timer based on hold duration
    fn try_start_timer(&mut self, now: Instant) {
        if let Some(hold_start) = self.space_hold_start {
            let hold_duration = now.duration_since(hold_start);

            if hold_duration >= self.key_preparation_time {
                self.start_timer(now);
            } else {
                self.state = TimerState::Ready;
            }
        }
    }

    // Starts the timer
    fn start_timer(&mut self, now: Instant) {
        self.state = TimerState::Running;
        self.start_time = Some(now);
        self.current_time = Duration::ZERO;
        self.target_timer_scale = 1.0;
    }

    // Deletes a time record
    fn delete_time(&mut self, index: usize) {
        if index < self.records.len() {
            self.records.remove(index);
            self.calculate_statistics();
            self.ui_state.confirm_delete_index = None;
            self.save_data(); // Ensure data is saved after deletion
        }
    }

    // Updates the comment for a time record
    fn update_time_comment(&mut self, index: usize, comment: String) {
        if index < self.records.len() {
            self.records[index].comment = comment;
        }
    }

    // Applies a penalty to a time record
    fn apply_penalty(&mut self, index: usize, penalty: Option<Penalty>) {
        if index < self.records.len() {
            self.records[index].penalty = penalty;
            self.calculate_statistics();
            self.save_data();
        }
    }

    // Updates timer state and animations
    fn handle_timer_updates(&mut self, ctx: &egui::Context) {
        if matches!(self.state, TimerState::Running) {
            if let Some(start_time) = self.start_time {
                self.current_time = Instant::now().duration_since(start_time);
            }
            ctx.request_repaint();
        }

        // Smooth animations
        if self.theme.enable_animations {
            let dt = ctx.input(|i| i.predicted_dt).min(1.0 / 30.0);
            let lerp_factor = (self.theme.animation_speed * dt * 10.0).min(1.0);

            self.timer_scale = self.timer_scale + (self.target_timer_scale - self.timer_scale) * lerp_factor;

            if (self.timer_scale - self.target_timer_scale).abs() > 0.001 {
                ctx.request_repaint();
            }
        } else {
            self.timer_scale = self.target_timer_scale;
        }
    }

    // Handles keyboard input
    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.handle_space_key(true);
            } else if i.key_released(egui::Key::Space) {
                self.handle_space_key(false);
            }
        });
    }

    // Applies theme settings to the UI
    fn setup_theme(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(self.theme.text_primary_color());
        visuals.panel_fill = self.theme.bg_color();
        visuals.window_fill = self.theme.surface_color();
        visuals.window_rounding = self.theme.rounding();
        visuals.menu_rounding = self.theme.rounding();
        visuals.button_frame = true;
        visuals.collapsing_header_frame = true;
        ctx.set_visuals(visuals);
        ctx.set_pixels_per_point(1.5);
    }

    // Renders the times panel on the left side
    fn render_times_panel(&mut self, ctx: &egui::Context) {
        if !self.ui_state.show_times_panel {
            return;
        }

        let panel_width = self.ui_state.times_panel_width;

        egui::SidePanel::left("times_panel")
            .resizable(true)
            .default_width(panel_width)
            .min_width(250.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                self.render_times_panel_header(ui);
                self.render_times_panel_stats(ui);
                ui.separator();
                self.render_times_list(ui);
                self.ui_state.times_panel_width = ui.min_size().x;
            });
    }

    // Renders the header of the times panel
    fn render_times_panel_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Times").size(self.theme.font_size_large).color(self.theme.text_primary_color()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    self.ui_state.show_times_panel = false;
                }
            });
        });
    }

    // Renders statistics in the times panel
    fn render_times_panel_stats(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Event:").size(self.theme.font_size_normal).color(self.theme.text_secondary_color()));
            ui.label(RichText::new(format!("{}", self.current_event)).size(self.theme.font_size_normal).color(self.theme.accent_primary_color()));
        });

        ui.horizontal_wrapped(|ui| {
            if let Some(best) = self.statistics.best {
                self.render_stat_chip(ui, "Best", &Self::format_time(best), self.theme.success_color());
            }
            if let Some(ao5) = self.statistics.current_ao5 {
                self.render_stat_chip(ui, "Ao5", &Self::format_time(ao5), self.theme.accent_primary_color());
            }
            if let Some(ao12) = self.statistics.current_ao12 {
                self.render_stat_chip(ui, "Ao12", &Self::format_time(ao12), self.theme.accent_secondary_color());
            }
        });
    }

    // Renders the list of times
    fn render_times_list(&mut self, ui: &mut egui::Ui) {
        let current_event = self.current_event.clone();
        let current_event_records: Vec<(usize, TimeRecord)> = self.records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.event == current_event)
            .map(|(i, r)| (i, r.clone()))
            .rev()
            .collect();

        if current_event_records.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No times yet").size(self.theme.font_size_normal).color(self.theme.text_secondary_color()));
            });
        } else {
            let total_records = current_event_records.len();
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (display_index, (actual_index, record)) in current_event_records.iter().enumerate() {
                        let solve_number = total_records - display_index;
                        self.render_time_entry(ui, solve_number, *actual_index, record);
                    }
                });
        }
    }

    // Renders a statistics chip
    fn render_stat_chip(&self, ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
        let chip_rect = ui.allocate_response(Vec2::new(80.0, 24.0), egui::Sense::hover()).rect;

        ui.painter().rect_filled(
            chip_rect,
            Rounding::same(12.0),
            color.gamma_multiply(0.1)
        );

        ui.painter().rect_stroke(
            chip_rect,
            Rounding::same(12.0),
            Stroke::new(1.0, color.gamma_multiply(0.3))
        );

        let text_pos = chip_rect.center() - Vec2::new(0.0, self.theme.font_size_small / 2.0);
        ui.painter().text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            format!("{}: {}", label, value),
            egui::FontId::proportional(self.theme.font_size_small),
            color
        );
    }

    // Renders a single time entry
    fn render_time_entry(&mut self, ui: &mut egui::Ui, display_index: usize, actual_index: usize, record: &TimeRecord) {
        let is_selected = self.ui_state.selected_time_index == Some(actual_index);
        let is_editing = self.ui_state.editing_comment_index == Some(actual_index);

        let entry_response = ui.allocate_response(
            Vec2::new(ui.available_width(), 60.0),
            egui::Sense::click()
        );

        self.render_time_entry_background(ui, &entry_response, is_selected);
        self.handle_time_entry_click(&entry_response, actual_index, is_selected);
        self.render_time_entry_content(ui, &entry_response, display_index, actual_index, record, is_editing);
    }

    // Renders the background of a time entry
    fn render_time_entry_background(&self, ui: &mut egui::Ui, entry_response: &egui::Response, is_selected: bool) {
        let bg_color = if is_selected {
            self.theme.accent_primary_color().gamma_multiply(0.1)
        } else if entry_response.hovered() {
            self.theme.surface_variant_color()
        } else {
            self.theme.surface_color()
        };

        ui.painter().rect_filled(
            entry_response.rect,
            self.theme.rounding(),
            bg_color
        );

        if is_selected {
            ui.painter().rect_stroke(
                entry_response.rect,
                self.theme.rounding(),
                Stroke::new(2.0, self.theme.accent_primary_color())
            );
        }
    }

    // Handles click events on time entries
    fn handle_time_entry_click(&mut self, entry_response: &egui::Response, actual_index: usize, is_selected: bool) {
        if entry_response.clicked() {
            self.ui_state.selected_time_index = if is_selected { None } else { Some(actual_index) };
        }
    }

    // Renders the content of a time entry
    fn render_time_entry_content(&mut self, ui: &mut egui::Ui, entry_response: &egui::Response, display_index: usize, actual_index: usize, record: &TimeRecord, is_editing: bool) {
        ui.allocate_ui_at_rect(entry_response.rect.shrink(8.0), |ui| {
            self.render_time_entry_main_row(ui, display_index, actual_index, record);
            self.render_comment_and_penalty_editor(ui, actual_index, record, is_editing);
        });
        ui.add_space(4.0);
    }

    // Renders the main row of a time entry
    fn render_time_entry_main_row(&mut self, ui: &mut egui::Ui, display_index: usize, actual_index: usize, record: &TimeRecord) {
        ui.horizontal(|ui| {
            self.render_time_entry_info(ui, display_index, record);
            self.render_time_entry_buttons(ui, actual_index, record);
        });
    }

    // Renders time entry information
    fn render_time_entry_info(&self, ui: &mut egui::Ui, display_index: usize, record: &TimeRecord) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("#{}", display_index))
                    .size(self.theme.font_size_small)
                    .color(self.theme.text_secondary_color()));

                let (time_color, time_text) = self.get_time_display_info(record);

                ui.label(RichText::new(time_text)
                    .size(self.theme.font_size_normal)
                    .color(time_color));
            });

            ui.label(RichText::new(record.timestamp.format("%H:%M:%S").to_string())
                .size(self.theme.font_size_small)
                .color(self.theme.text_secondary_color()));
        });
    }

    // Gets display information for a time record
    fn get_time_display_info(&self, record: &TimeRecord) -> (Color32, String) {
        let time_color = match record.penalty {
            Some(Penalty::DNF) => self.theme.error_color(),
            Some(Penalty::Plus2) => self.theme.warning_color(),
            None => self.theme.text_primary_color(),
        };

        let time_text = match record.penalty {
            Some(Penalty::DNF) => "DNF".to_string(),
            Some(Penalty::Plus2) => format!("{}+", Self::format_time(record.time)),
            None => Self::format_time(record.time),
        };

        (time_color, time_text)
    }

    // Renders buttons for a time entry
    fn render_time_entry_buttons(&mut self, ui: &mut egui::Ui, actual_index: usize, record: &TimeRecord) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("🗑").clicked() {
                self.ui_state.confirm_delete_index = Some(actual_index);
            }

            let comment_button_text = if record.comment.is_empty() { "💬" } else { "📝" };
            if ui.small_button(comment_button_text).clicked() {
                self.handle_comment_button_click(actual_index, record);
            }
        });
    }

    // Handles comment button clicks
    fn handle_comment_button_click(&mut self, actual_index: usize, record: &TimeRecord) {
        let is_editing = self.ui_state.editing_comment_index == Some(actual_index);

        if is_editing {
            self.ui_state.editing_comment_index = None;
            self.update_time_comment(actual_index, self.ui_state.comment_text.clone());
            self.ui_state.comment_text.clear();
        } else {
            self.ui_state.editing_comment_index = Some(actual_index);
            self.ui_state.comment_text = record.comment.clone();
        }
    }

    // Renders the comment section with penalty buttons
    fn render_comment_and_penalty_editor(&mut self, ui: &mut egui::Ui, actual_index: usize, record: &TimeRecord, is_editing: bool) {
        if !record.comment.is_empty() && !is_editing {
            ui.label(RichText::new(&record.comment)
                .size(self.theme.font_size_small)
                .color(self.theme.text_secondary_color())
                .italics());
        }

        if is_editing {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.ui_state.comment_text);
                if ui.small_button("✓").clicked() {
                    self.update_time_comment(actual_index, self.ui_state.comment_text.clone());
                    self.ui_state.editing_comment_index = None;
                    self.ui_state.comment_text.clear();
                    self.save_data();
                }
                if ui.small_button("✕").clicked() {
                    self.ui_state.editing_comment_index = None;
                    self.ui_state.comment_text.clear();
                }
            });

            ui.horizontal(|ui| {
                let plus2_color = if record.penalty == Some(Penalty::Plus2) { self.theme.warning_color() } else { self.theme.text_primary_color().gamma_multiply(0.5) };
                if ui.add(egui::Button::new(RichText::new("+2").color(plus2_color))).clicked() {
                    if record.penalty == Some(Penalty::Plus2) {
                        self.apply_penalty(actual_index, None);
                    } else {
                        self.apply_penalty(actual_index, Some(Penalty::Plus2));
                    }
                }

                let dnf_color = if record.penalty == Some(Penalty::DNF) { self.theme.error_color() } else { self.theme.text_primary_color().gamma_multiply(0.5) };
                if ui.add(egui::Button::new(RichText::new("DNF").color(dnf_color))).clicked() {
                    if record.penalty == Some(Penalty::DNF) {
                        self.apply_penalty(actual_index, None);
                    } else {
                        self.apply_penalty(actual_index, Some(Penalty::DNF));
                    }
                }
            });
        }
    }

    // Renders the main content area
    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if !self.ui_state.show_times_panel {
                if ui.button("📊 Times").clicked() {
                    self.ui_state.show_times_panel = true;
                }
            }

            ui.separator();
            self.render_enhanced_event_selector(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙ Settings").clicked() {
                    self.ui_state.show_settings = !self.ui_state.show_settings;
                }
                if ui.button("📈 Stats").clicked() {
                    self.ui_state.show_statistics = !self.ui_state.show_statistics;
                }
            });
        });

        ui.separator();
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            self.render_enhanced_scramble(ui);
            ui.add_space(60.0);
            self.render_enhanced_timer(ui);
            ui.add_space(30.0);
            self.render_enhanced_state_indicator(ui);
            ui.add_space(40.0);
            self.render_enhanced_quick_stats(ui);
        });
    }

    // Renders the event selector
    fn render_enhanced_event_selector(&mut self, ui: &mut egui::Ui) {
        let current_event = self.current_event.clone();
        let available_events = self.available_events.clone();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Event:").size(self.theme.font_size_normal).color(self.theme.text_secondary_color()));

            egui::ComboBox::from_id_source("event_selector")
                .selected_text(RichText::new(format!("{}", current_event))
                    .size(self.theme.font_size_normal)
                    .color(self.theme.accent_primary_color()))
                .show_ui(ui, |ui| {
                    for event in &available_events {
                        if ui.selectable_value(&mut self.current_event, event.clone(),
                                               RichText::new(format!("{}", event)).size(self.theme.font_size_normal)).clicked() {
                            self.generate_new_scramble();
                            self.calculate_statistics();
                        }
                    }
                });
        });
    }

    // Renders the scramble display
    fn render_enhanced_scramble(&self, ui: &mut egui::Ui) {
        let scramble_rect = ui.allocate_response(
            Vec2::new(ui.available_width().min(800.0), 80.0),
            egui::Sense::hover()
        ).rect;

        ui.painter().rect_filled(
            scramble_rect,
            self.theme.rounding(),
            self.theme.surface_color()
        );

        ui.painter().rect_stroke(
            scramble_rect,
            self.theme.rounding(),
            Stroke::new(1.0, self.theme.accent_primary_color().gamma_multiply(0.3))
        );

        ui.allocate_ui_at_rect(scramble_rect.shrink(16.0), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(&self.current_scramble)
                    .size(self.theme.font_size_large)
                    .color(self.theme.text_primary_color())
                    .family(egui::FontFamily::Monospace));
            });
        });
    }

    // Renders the timer display
    fn render_enhanced_timer(&self, ui: &mut egui::Ui) {
        let timer_text = self.get_timer_text();
        let timer_color = self.get_timer_color();
        let scaled_size = self.theme.font_size_timer * self.timer_scale;

        let timer_response = ui.allocate_response(
            Vec2::new(ui.available_width(), scaled_size + 40.0),
            egui::Sense::hover()
        );

        if matches!(self.state, TimerState::Running) {
            let glow_rect = timer_response.rect.expand(20.0);
            ui.painter().rect_filled(
                glow_rect,
                Rounding::same(30.0),
                timer_color.gamma_multiply(0.1)
            );
        }

        ui.allocate_ui_at_rect(timer_response.rect, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(timer_text)
                    .size(scaled_size)
                    .color(timer_color)
                    .family(egui::FontFamily::Monospace));
            });
        });
    }

    // Gets the timer display text
    fn get_timer_text(&self) -> String {
        if matches!(self.state, TimerState::Running) {
            Self::format_time(self.current_time)
        } else if let Some(last_time) = self.last_time {
            Self::format_time(last_time)
        } else {
            "0.000".to_string()
        }
    }

    // Determines the timer text color based on the state and hold time
    fn get_timer_color(&self) -> Color32 {
        if let TimerState::Preparing = self.state {
            if let Some(hold_start) = self.space_hold_start {
                if hold_start.elapsed() >= self.key_preparation_time {
                    return self.theme.success_color();
                }
            }
        }
        self.theme.timer_color(&self.state)
    }

    // Renders the timer state indicator
    fn render_enhanced_state_indicator(&self, ui: &mut egui::Ui) {
        let (state_text, state_color) = match self.state {
            TimerState::Ready => ("Press and hold SPACE to start", self.theme.text_secondary_color()),
            TimerState::Preparing => {
                if let Some(hold_start) = self.space_hold_start {
                    if hold_start.elapsed() >= self.key_preparation_time {
                        ("Release to Start", self.theme.success_color())
                    } else {
                        ("Hold SPACE...", self.theme.timer_color(&TimerState::Preparing))
                    }
                } else {
                    ("Hold SPACE...", self.theme.timer_color(&TimerState::Preparing))
                }
            },
            TimerState::Running => ("RUNNING - Press SPACE to stop", self.theme.timer_color(&TimerState::Running)),
            TimerState::Stopped => ("Press SPACE for next solve", self.theme.success_color()),
        };

        ui.label(RichText::new(state_text)
            .size(self.theme.font_size_normal)
            .color(state_color));
    }

    // Renders quick statistics cards
    fn render_enhanced_quick_stats(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 20.0;

            if let Some(best) = self.statistics.best {
                self.render_stat_card(ui, "Best", &Self::format_time(best), self.theme.success_color());
            }
            if let Some(ao5) = self.statistics.current_ao5 {
                self.render_stat_card(ui, "Ao5", &Self::format_time(ao5), self.theme.accent_primary_color());
            }
            if let Some(ao12) = self.statistics.current_ao12 {
                self.render_stat_card(ui, "Ao12", &Self::format_time(ao12), self.theme.accent_secondary_color());
            }
            if let Some(mean) = self.statistics.mean {
                self.render_stat_card(ui, "Mean", &Self::format_time(mean), self.theme.text_secondary_color());
            }
        });
    }

    // Renders a statistics card
    fn render_stat_card(&self, ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
        let card_rect = ui.allocate_response(Vec2::new(100.0, 60.0), egui::Sense::hover()).rect;

        ui.painter().rect_filled(
            card_rect,
            self.theme.rounding(),
            self.theme.surface_color()
        );

        ui.painter().rect_stroke(
            card_rect,
            self.theme.rounding(),
            Stroke::new(2.0, color.gamma_multiply(0.5))
        );

        ui.allocate_ui_at_rect(card_rect.shrink(8.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(label)
                    .size(self.theme.font_size_small)
                    .color(self.theme.text_secondary_color()));
                ui.label(RichText::new(value)
                    .size(self.theme.font_size_normal)
                    .color(color));
            });
        });
    }

    // Renders all modal windows
    fn render_windows(&mut self, ctx: &egui::Context) {
        self.render_settings_window(ctx);
        self.render_statistics_window(ctx);
        self.render_delete_confirmation(ctx);
        self.render_exit_confirmation(ctx);
        self.render_welcome_popup(ctx);
    }

    // Renders the welcome popup for first-time users
    fn render_welcome_popup(&mut self, ctx: &egui::Context) {
        if !self.ui_state.is_first_launch {
            return;
        }

        let mut is_open = true;
        egui::Window::new("👋 Welcome!")
            .open(&mut is_open)
            .default_width(400.0)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(RichText::new("Welcome to CubeTimer Pro! Here's a quick guide to get you started:").size(self.theme.font_size_normal));
                ui.add_space(10.0);

                ui.label(RichText::new("To solve:").strong().size(self.theme.font_size_normal));
                ui.label("Hold the SPACE bar to prepare the timer. The text will turn green. Release the SPACE bar to start the timer, and press it again to stop.");
                ui.add_space(10.0);

                ui.label(RichText::new("Buttons:").strong().size(self.theme.font_size_normal));
                ui.label("📊 Times: Opens the panel on the left to view your solve history and statistics.");
                ui.label("📈 Stats: Opens a separate window to view a graph of your solve times.");
                ui.label("⚙ Settings: Opens a window to customize the app's theme and other options.");
                ui.add_space(10.0);

                ui.centered_and_justified(|ui| {
                    if ui.button(RichText::new("Got it!").strong()).clicked() {
                        self.ui_state.is_first_launch = false;
                        self.save_data();
                    }
                });
            });

        if !is_open {
            self.ui_state.is_first_launch = false;
            self.save_data();
        }
    }

    // Renders the settings window
    fn render_settings_window(&mut self, ctx: &egui::Context) {
        if !self.ui_state.show_settings {
            return;
        }

        let mut show_settings = self.ui_state.show_settings;
        egui::Window::new("⚙ Settings")
            .open(&mut show_settings)
            .default_width(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(5.0);

                    // Theme Colors Section
                    egui::CollapsingHeader::new(RichText::new("🎨 Theme Colors").strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.columns(2, |columns| {
                                columns[0].vertical(|ui| {
                                    ui.add_space(5.0);
                                    ui.label("Background:");
                                    ui.color_edit_button_srgb(&mut self.theme.background);
                                    ui.add_space(10.0);
                                    ui.label("Surface:");
                                    ui.color_edit_button_srgb(&mut self.theme.surface);
                                    ui.add_space(10.0);
                                    ui.label("Surface Variant:");
                                    ui.color_edit_button_srgb(&mut self.theme.surface_variant);
                                    ui.add_space(10.0);
                                    ui.label("Primary Text:");
                                    ui.color_edit_button_srgb(&mut self.theme.text_primary);
                                    ui.add_space(10.0);
                                    ui.label("Secondary Text:");
                                    ui.color_edit_button_srgb(&mut self.theme.text_secondary);
                                    ui.add_space(10.0);
                                });

                                columns[1].vertical(|ui| {
                                    ui.add_space(5.0);
                                    ui.label("Primary Accent:");
                                    ui.color_edit_button_srgb(&mut self.theme.accent_primary);
                                    ui.add_space(10.0);
                                    ui.label("Secondary Accent:");
                                    ui.color_edit_button_srgb(&mut self.theme.accent_secondary);
                                    ui.add_space(10.0);
                                    ui.label("Success:");
                                    ui.color_edit_button_srgb(&mut self.theme.success);
                                    ui.add_space(10.0);
                                    ui.label("Warning:");
                                    ui.color_edit_button_srgb(&mut self.theme.warning);
                                    ui.add_space(10.0);
                                    ui.label("Error:");
                                    ui.color_edit_button_srgb(&mut self.theme.error);
                                    ui.add_space(10.0);
                                });
                            });

                            ui.separator();
                            ui.label(RichText::new("Timer Colors").strong());
                            ui.columns(4, |columns| {
                                columns[0].vertical(|ui| {
                                    ui.label("Ready:");
                                    ui.color_edit_button_srgb(&mut self.theme.timer_ready);
                                });
                                columns[1].vertical(|ui| {
                                    ui.label("Preparing:");
                                    ui.color_edit_button_srgb(&mut self.theme.timer_preparing);
                                });
                                columns[2].vertical(|ui| {
                                    ui.label("Running:");
                                    ui.color_edit_button_srgb(&mut self.theme.timer_running);
                                });
                                columns[3].vertical(|ui| {
                                    ui.label("Stopped:");
                                    ui.color_edit_button_srgb(&mut self.theme.timer_stopped);
                                });
                            });
                        });
                    ui.add_space(10.0);
                    ui.separator();

                    // UI Settings Section
                    egui::CollapsingHeader::new(RichText::new("⚙ UI Settings").strong())
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(5.0);
                            ui.label("Corner Radius:");
                            ui.add(egui::Slider::new(&mut self.theme.corner_radius, 0.0..=24.0));
                            ui.add_space(10.0);

                            ui.label("Font Sizes:");
                            ui.horizontal(|ui| {
                                ui.label("Small:");
                                ui.add(egui::Slider::new(&mut self.theme.font_size_small, 8.0..=16.0));
                                ui.label("Normal:");
                                ui.add(egui::Slider::new(&mut self.theme.font_size_normal, 10.0..=20.0));
                                ui.label("Large:");
                                ui.add(egui::Slider::new(&mut self.theme.font_size_large, 14.0..=28.0));
                            });
                            ui.add_space(10.0);

                            ui.checkbox(&mut self.theme.enable_animations, "Enable animations");
                            ui.add_space(10.0);
                            if self.theme.enable_animations {
                                ui.label("Animation Speed:");
                                ui.add(egui::Slider::new(&mut self.theme.animation_speed, 0.5..=2.0));
                            }
                        });
                    ui.add_space(10.0);
                    ui.separator();

                    // Events Section
                    egui::CollapsingHeader::new(RichText::new("🎲 Custom Events").strong())
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add_space(5.0);
                            ui.label("Create New Custom Event:");
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.new_custom_event_name);
                                ui.label("Moves (comma-separated):");
                                ui.text_edit_singleline(&mut self.new_custom_moves);
                            });

                            if ui.button("Add Custom Event").clicked() {
                                self.add_custom_event();
                            }

                            ui.separator();

                            ui.label("Existing Custom Events:");
                            let custom_event_names: Vec<String> = self.custom_events.keys().cloned().collect();
                            for name in custom_event_names {
                                ui.horizontal(|ui| {
                                    ui.label(&name);
                                    if ui.button("Remove").clicked() {
                                        self.remove_custom_event(&name);
                                    }
                                });
                            }
                        });
                    ui.add_space(10.0);
                    ui.separator();
                });
            });
        self.ui_state.show_settings = show_settings;
    }

    // Renders the statistics window
        fn render_statistics_window(&mut self, ctx: &egui::Context) {
            if !self.ui_state.show_statistics {
                return;
            }
    
            let mut show_stats = self.ui_state.show_statistics;
            egui::Window::new("📈 Statistics")
                .open(&mut show_stats)
                .default_width(1000.0)
                .default_height(800.0)
                .resizable(true)
                .show(ctx, |ui| {
                    let current_event_records: Vec<(usize, TimeRecord)> = self.records
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.event == self.current_event)
                        .map(|(i, r)| (i, r.clone()))
                        .collect();
    
                    if current_event_records.len() < 2 {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Need at least 2 solves to show statistics").size(self.theme.font_size_normal).color(self.theme.text_secondary_color()));
                        });
                        return;
                    }
    
                    // Prepare plot data
                    let mut solve_points: Vec<egui_plot::PlotPoint> = Vec::new();
                    let mut ao5_points: Vec<egui_plot::PlotPoint> = Vec::new();
                    let mut ao12_points: Vec<egui_plot::PlotPoint> = Vec::new();
    
                    let mut current_times_for_avg: Vec<Duration> = Vec::new();
                    for (i, (_, record)) in current_event_records.iter().enumerate() {
                        let solve_time_ms = record.time.as_millis() as f64;
                        solve_points.push(egui_plot::PlotPoint::new(i as f64, solve_time_ms));
    
                        current_times_for_avg.push(record.time);
    
                        if current_times_for_avg.len() >= 5 {
                            let last_5: Vec<Duration> = current_times_for_avg.iter().rev().take(5).cloned().collect();
                            if let Some(ao5) = Self::calculate_average(&last_5) {
                                ao5_points.push(egui_plot::PlotPoint::new(i as f64, ao5.as_millis() as f64));
                            }
                        }
    
                        if current_times_for_avg.len() >= 12 {
                            let last_12: Vec<Duration> = current_times_for_avg.iter().rev().take(12).cloned().collect();
                            if let Some(ao12) = Self::calculate_average(&last_12) {
                                ao12_points.push(egui_plot::PlotPoint::new(i as f64, ao12.as_millis() as f64));
                            }
                        }
                    }
    
                    // Convert PlotPoint vectors to [f64; 2] vectors for PlotPoints
                    let solve_coords: Vec<[f64; 2]> = solve_points.iter()
                        .map(|point| [point.x, point.y])
                        .collect();
                    let ao5_coords: Vec<[f64; 2]> = ao5_points.iter()
                        .map(|point| [point.x, point.y])
                        .collect();
                    let ao12_coords: Vec<[f64; 2]> = ao12_points.iter()
                        .map(|point| [point.x, point.y])
                        .collect();
    
                    let solve_line = Line::new(PlotPoints::from(solve_coords))
                        .color(self.theme.accent_primary_color())
                        .name("Solve Times");
                    let ao5_line = Line::new(PlotPoints::from(ao5_coords))
                        .color(self.theme.success_color())
                        .name("Ao5");
                    let ao12_line = Line::new(PlotPoints::from(ao12_coords))
                        .color(self.theme.accent_secondary_color())
                        .name("Ao12");
    
                    let plot = Plot::new("time_graph")
                        .view_aspect(2.0)
                        .show_axes([false, true])
                        .legend(Legend::default())
                        .set_margin_fraction(Vec2::new(0.05, 0.05));
    
                    plot.show(ui, |plot_ui| {
                        plot_ui.line(solve_line);
                        plot_ui.line(ao5_line);
                        plot_ui.line(ao12_line);
                    });
                });
    
            self.ui_state.show_statistics = show_stats;
        }

    // Renders the delete confirmation popup
    fn render_delete_confirmation(&mut self, ctx: &egui::Context) {
        if self.ui_state.confirm_delete_index.is_none() {
            return;
        }

        let mut show_popup = true;
        egui::Window::new("Confirm Delete")
            .open(&mut show_popup)
            .default_width(300.0)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(RichText::new("Are you sure you want to delete this time?").size(self.theme.font_size_normal).color(self.theme.warning_color()));

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Yes, delete").clicked() {
                        if let Some(index) = self.ui_state.confirm_delete_index {
                            self.delete_time(index);
                        }
                    }
                    if ui.button("No, cancel").clicked() {
                        self.ui_state.confirm_delete_index = None;
                    }
                });
            });

        if !show_popup {
            self.ui_state.confirm_delete_index = None;
        }
    }

    // Renders the exit confirmation popup
    fn render_exit_confirmation(&mut self, ctx: &egui::Context) {
        if !self.ui_state.show_exit_popup {
            return;
        }
    
        let mut show_popup = self.ui_state.show_exit_popup;
        
        let response = egui::Window::new("Exit Application")
            .open(&mut show_popup)
            .default_width(300.0)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(RichText::new("Do you want to save your data before exiting?").size(self.theme.font_size_normal));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save & Exit").strong()).clicked() {
                        self.save_data();
                        std::process::exit(0);
                    }
                    if ui.button("Exit without saving").clicked() {
                        std::process::exit(0);
                    }
                    if ui.button("Cancel").clicked() {
                        self.ui_state.show_exit_popup = false;
                    }
                });
            });
        
        // Update the popup state after the window is shown
        self.ui_state.show_exit_popup = show_popup;
    }

    // Adds a new custom event
    fn add_custom_event(&mut self) {
        if self.new_custom_event_name.trim().is_empty() || self.new_custom_moves.trim().is_empty() {
            return;
        }

        let moves: Vec<String> = self.new_custom_moves.split(',').map(|s| s.trim().to_string()).collect();
        let custom_event = CustomEvent {
            name: self.new_custom_event_name.clone(),
            scramble_length: 20, // Default scramble length for custom events for now
            moves,
        };

        self.custom_events.insert(self.new_custom_event_name.clone(), custom_event);
        self.available_events.push(CubeEvent::Custom(self.new_custom_event_name.clone()));

        self.new_custom_event_name.clear();
        self.new_custom_moves.clear();
    }

    // Removes a custom event
    fn remove_custom_event(&mut self, name: &str) {
        self.custom_events.remove(name);
        self.available_events.retain(|event| {
            if let CubeEvent::Custom(custom_name) = event {
                custom_name != name
            } else {
                true
            }
        });
        if let Some(CubeEvent::Custom(current_name)) = Some(self.current_event.clone()) {
            if current_name == name {
                self.current_event = self.available_events[0].clone();
                self.generate_new_scramble();
            }
        }
    }
}
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_title("CubeTimer Pro - Cool Speedcubing Timer"),
        ..Default::default()
    };

    eframe::run_native(
        "CubeTimer Pro",
        options,
        Box::new(|cc| Box::new(CubeTimer::new(cc))),
    )
}

impl eframe::App for CubeTimer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_timer_updates(ctx);
        self.handle_input(ctx);
        self.setup_theme(ctx);

        let now = Instant::now();
        if now.duration_since(self.last_save_time) > Duration::from_secs(120) {
            self.save_data();
            self.last_save_time = now;
        }

        self.render_times_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_content(ui);
        });

        self.render_windows(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.ui_state.show_exit_popup = true;
    }
}