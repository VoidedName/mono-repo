use anyhow::Result;
use chrono::{Duration, NaiveTime, Timelike};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};
use rodio::{source::Source, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TimedEvent {
    time: NaiveTime,
    name: String,
    auto_pause: bool,
    repeat_interval: Option<Duration>,
    repeat_until: Option<NaiveTime>,
    #[serde(with = "color_serde")]
    color: Color,
}

mod color_serde {
    use ratatui::style::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match color {
            Color::Reset => "Reset",
            Color::Black => "Black",
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Yellow => "Yellow",
            Color::Blue => "Blue",
            Color::Magenta => "Magenta",
            Color::Cyan => "Cyan",
            Color::Gray => "Gray",
            Color::DarkGray => "DarkGray",
            Color::LightRed => "LightRed",
            Color::LightGreen => "LightGreen",
            Color::LightYellow => "LightYellow",
            Color::LightBlue => "LightBlue",
            Color::LightMagenta => "LightMagenta",
            Color::LightCyan => "LightCyan",
            Color::White => "White",
            _ => "White",
        };
        s.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "Reset" => Color::Reset,
            "Black" => Color::Black,
            "Red" => Color::Red,
            "Green" => Color::Green,
            "Yellow" => Color::Yellow,
            "Blue" => Color::Blue,
            "Magenta" => Color::Magenta,
            "Cyan" => Color::Cyan,
            "Gray" => Color::Gray,
            "DarkGray" => Color::DarkGray,
            "LightRed" => Color::LightRed,
            "LightGreen" => Color::LightGreen,
            "LightYellow" => Color::LightYellow,
            "LightBlue" => Color::LightBlue,
            "LightMagenta" => Color::LightMagenta,
            "LightCyan" => Color::LightCyan,
            "White" => Color::White,
            _ => Color::White,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct LogEntry {
    message: String,
    #[serde(with = "color_serde")]
    color: Color,
}

#[derive(Serialize, Deserialize)]
struct ClockConfig {
    initial_time: NaiveTime,
    target_speed: f64,
    events: Vec<TimedEvent>,
}

#[derive(Serialize, Deserialize)]
struct ClockState {
    clock_time: NaiveTime,
    initial_time: NaiveTime,
    target_speed: f64,
    paused: bool,
    events: Vec<TimedEvent>,
    logs: Vec<LogEntry>,
}

#[derive(PartialEq)]
enum Section {
    Config,
    Log,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    EditingTime,
    EditingSpeed,
    EventManagement,
    AddingEventName,
    AddingEventTime,
    AddingEventAutoPause,
    AddingEventRepeatInterval,
    AddingEventRepeatUntil,
    SavingConfig,
    LoadingConfig,
    SavingState,
    LoadingState,
    ConfirmOverwriteConfig,
    ConfirmOverwriteState,
    Help,
}

struct App {
    clock_time: NaiveTime,
    initial_time: NaiveTime,
    speed: f64,
    paused: bool,
    events: Vec<TimedEvent>,
    logs: Vec<LogEntry>,
    last_tick: Instant,
    sink: Arc<Mutex<Option<Sink>>>,
    input_mode: InputMode,
    input_buffer: String,
    target_speed: f64,
    selected_event: usize,
    // Temporary state for adding a new event
    temp_event_name: String,
    temp_event_time: Option<NaiveTime>,
    temp_event_auto_pause: bool,
    temp_event_repeat_interval: Option<Duration>,
    // File explorer state
    files: Vec<String>,
    selected_file: Option<usize>,
    config_scroll: usize,
    log_scroll: usize,
    help_scroll: usize,
    selected_section: Section,
}

impl App {
    fn new() -> Self {
        Self {
            clock_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            initial_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            speed: 0.0,
            paused: true,
            events: vec![],
            logs: vec![LogEntry {
                message: "Welcome to Digital Clock!".to_string(),
                color: Color::White,
            }],
            last_tick: Instant::now(),
            sink: Arc::new(Mutex::new(None)),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            target_speed: 1.0,
            selected_event: 0,
            temp_event_name: String::new(),
            temp_event_time: None,
            temp_event_auto_pause: false,
            temp_event_repeat_interval: None,
            files: vec![],
            selected_file: None,
            config_scroll: 0,
            log_scroll: 0,
            help_scroll: 0,
            selected_section: Section::Log,
        }
    }

    fn add_log(&mut self, message: String, color: Color) {
        self.logs.push(LogEntry { message, color });
    }

    fn get_random_color(&self) -> Color {
        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
        ];
        // Use events length as a pseudo-random seed
        colors[self.events.len() % colors.len()]
    }

    fn refresh_files(&mut self, suffix: &str) {
        self.files.clear();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(suffix) {
                                self.files.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        self.files.sort();
        self.selected_file = None;
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick);
        self.last_tick = now;

        if self.paused {
            self.speed = 0.0;
            return;
        }

        self.speed = self.target_speed;

        let seconds_to_add = delta.as_secs_f64() * self.speed;
        let old_time = self.clock_time;
        
        let duration_to_add = Duration::nanoseconds((seconds_to_add * 1_000_000_000.0) as i64);
        
        // NaiveTime doesn't easily support adding Duration that might wrap around days in a way we want for a simple clock
        // but for this purpose, we can use overflow.
        let new_time_total_nanos = (self.clock_time.num_seconds_from_midnight() as i64 * 1_000_000_000 + self.clock_time.nanosecond() as i64) + duration_to_add.num_nanoseconds().unwrap();
        
        let secs = (new_time_total_nanos / 1_000_000_000) % (24 * 3600);
        let nanos = new_time_total_nanos % 1_000_000_000;
        
        self.clock_time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nanos as u32).unwrap();

        self.check_events(old_time, self.clock_time);
    }

    fn check_events(&mut self, old_time: NaiveTime, new_time: NaiveTime) {
        let t1 = old_time.num_seconds_from_midnight() as i64;
        let mut t2 = new_time.num_seconds_from_midnight() as i64;

        if t2 < t1 {
            // Clock wrapped around midnight during this tick
            t2 += 86400;
        }

        // Clone events to avoid borrowing issues while iterating and modifying self (logs, paused)
        let events = self.events.clone();

        for event in events {
            let base_t = event.time.num_seconds_from_midnight() as i64;

            let mut trigger_times = vec![base_t];
            if let Some(interval) = event.repeat_interval {
                let period = interval.num_seconds();
                if period > 0 {
                    let mut until_t = event.repeat_until
                        .map(|t| t.num_seconds_from_midnight() as i64)
                        .unwrap_or(86399); // Default to end of day if not specified
                    
                    // If until time is before or equal to start time, it refers to the next day
                    if event.repeat_until.is_some() && until_t <= base_t {
                        until_t += 86400;
                    }

                    let mut curr = base_t + period;
                    while curr <= until_t {
                        trigger_times.push(curr);
                        curr += period;
                    }
                }
            }

            for tt in trigger_times {
                // Check if this specific occurrence (tt) falls within our current tick interval (t1, t2]
                // We check tt, tt + 86400, and tt - 86400 to handle any wrap-around scenarios
                let mut triggered = false;
                for offset in &[-86400, 0, 86400] {
                    let adjusted_tt = tt + offset;
                    if adjusted_tt > t1 && adjusted_tt <= t2 {
                        triggered = true;
                        break;
                    }
                }

                if triggered {
                    self.add_log(
                        format!(
                            "[{}] EVENT: {}",
                            self.clock_time.format("%H:%M:%S"),
                            event.name
                        ),
                        event.color,
                    );
                    self.play_ding();
                    if event.auto_pause {
                        self.paused = true;
                    }
                    // Continue to next occurrence in trigger_times (multiple triggers possible in one tick)
                }
            }
        }
    }

    fn play_ding(&self) {
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                // Play a simple beep
                let source = rodio::source::SineWave::new(440.0)
                    .take_duration(std::time::Duration::from_millis(200))
                    .amplify(0.2);
                sink.append(source);
            }
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    // Create app and run it
    let mut app = App::new();
    app.sink = Arc::new(Mutex::new(Some(sink)));
    
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn parse_time(s: &str) -> Option<NaiveTime> {
    let s = s.trim();
    if s == "24:00:00" || s == "24:00" {
        return NaiveTime::from_hms_opt(0, 0, 0);
    }
    if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S") {
        Some(time)
    } else if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M") {
        Some(time)
    } else {
        None
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = std::time::Duration::from_millis(16);
        if event::poll(timeout)? {
            // Process all pending events
            while event::poll(std::time::Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == event::KeyEventKind::Press {
                            match app.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('q') => return Ok(()),
                                    KeyCode::Char(' ') => app.paused = !app.paused,
                                    KeyCode::Char('v') => {
                                        app.input_mode = InputMode::EditingSpeed;
                                        app.input_buffer.clear();
                                    }
                                    KeyCode::Char('e') => {
                                        app.input_mode = InputMode::EventManagement;
                                        app.selected_event = 0;
                                    }
                                    KeyCode::Char('r') => {
                                        app.clock_time = app.initial_time;
                                        app.add_log(format!("Clock reset to {}", app.initial_time), Color::White);
                                    }
                                    KeyCode::Char('t') => {
                                        app.input_mode = InputMode::EditingTime;
                                        app.input_buffer.clear();
                                        app.paused = true;
                                    }
                                    KeyCode::Char('h') => {
                                        app.input_mode = InputMode::Help;
                                        app.help_scroll = 0;
                                    }
                                    KeyCode::Char('S') => {
                                        app.input_mode = InputMode::SavingConfig;
                                        app.input_buffer.clear();
                                        app.refresh_files(".config.json");
                                    }
                                    KeyCode::Char('L') => {
                                        app.input_mode = InputMode::LoadingConfig;
                                        app.refresh_files(".config.json");
                                    }
                                    KeyCode::Char('s') => {
                                        app.input_mode = InputMode::SavingState;
                                        app.input_buffer.clear();
                                        app.refresh_files(".state.json");
                                    }
                                    KeyCode::Char('l') => {
                                        app.input_mode = InputMode::LoadingState;
                                        app.refresh_files(".state.json");
                                    }
                                    KeyCode::Up => {
                                        match app.selected_section {
                                            Section::Config => {
                                                if app.config_scroll > 0 {
                                                    app.config_scroll -= 1;
                                                }
                                            }
                                            Section::Log => {
                                                if app.log_scroll > 0 {
                                                    app.log_scroll -= 1;
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Down => {
                                        match app.selected_section {
                                            Section::Config => {
                                                let config_lines_count = if app.events.is_empty() {
                                                    4 // Initial Time, Target Speed, Events, (None)
                                                } else {
                                                    3 + app.events.len() // Initial Time, Target Speed, Events, + each event
                                                };
                                                let max_scroll = config_lines_count.saturating_sub(1);
                                                if app.config_scroll < max_scroll {
                                                    app.config_scroll += 1;
                                                }
                                            }
                                            Section::Log => {
                                                let max_scroll = app.logs.len().saturating_sub(1);
                                                if app.log_scroll < max_scroll {
                                                    app.log_scroll += 1;
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Left => {
                                        app.selected_section = Section::Config;
                                    }
                                    KeyCode::Right => {
                                        app.selected_section = Section::Log;
                                    }
                                    _ => {}
                                },
                                InputMode::Help => match key.code {
                                    KeyCode::Char('h') | KeyCode::Esc => {
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Up => {
                                        if app.help_scroll > 0 {
                                            app.help_scroll -= 1;
                                        }
                                    }
                                    KeyCode::Down => {
                                        // We need to know the total lines in help text to bound scroll
                                        let help_lines: Vec<&str> = HELP_TEXT.lines().collect();
                                        let total_lines = help_lines.len();
                                        
                                        // Estimate visible height (must match centered_rect(60, 60, ...))
                                        let height = terminal.size()?.height;
                                        let area_height = (height * 60) / 100;
                                        let visible_height = area_height.saturating_sub(2) as usize; // -2 for borders

                                        let max_scroll = total_lines.saturating_sub(visible_height);
                                        if app.help_scroll < max_scroll {
                                            app.help_scroll += 1;
                                        }
                                    }
                                    _ => {}
                                },
                                InputMode::EditingTime => match key.code {
                                    KeyCode::Enter => {
                                        if let Some(time) = parse_time(&app.input_buffer) {
                                            app.clock_time = time;
                                            app.initial_time = time;
                                            app.add_log(format!("Clock set to {}", time), Color::White);
                                        } else {
                                            app.add_log(format!(
                                                "Invalid time format: {}",
                                                app.input_buffer
                                            ), Color::Red);
                                        }
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char(c) => app.input_buffer.push(c),
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                                    _ => {}
                                },
                                InputMode::EditingSpeed => match key.code {
                                    KeyCode::Enter => {
                                        if let Ok(speed) = app.input_buffer.parse::<f64>() {
                                            app.target_speed = speed;
                                            app.add_log(format!("Speed set to {:.2}x", speed), Color::White);
                                        } else {
                                            app.add_log(format!(
                                                "Invalid speed: {}",
                                                app.input_buffer
                                            ), Color::Red);
                                        }
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char(c) => app.input_buffer.push(c),
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                                    _ => {}
                                },
                                InputMode::EventManagement => match key.code {
                                    KeyCode::Char('a') => {
                                        app.input_mode = InputMode::AddingEventName;
                                        app.input_buffer.clear();
                                        app.temp_event_name.clear();
                                        app.temp_event_time = None;
                                        app.temp_event_auto_pause = false;
                                    }
                                    KeyCode::Char('d') => {
                                        if !app.events.is_empty() && app.selected_event < app.events.len() {
                                            let removed = app.events.remove(app.selected_event);
                                            app.add_log(format!("Removed event: {}", removed.name), Color::White);
                                            if app.selected_event >= app.events.len() && !app.events.is_empty() {
                                                app.selected_event = app.events.len() - 1;
                                            }
                                        }
                                    }
                                    KeyCode::Up => {
                                        if app.selected_event > 0 {
                                            app.selected_event -= 1;
                                        }
                                    }
                                    KeyCode::Down => {
                                        if !app.events.is_empty() && app.selected_event < app.events.len() - 1 {
                                            app.selected_event += 1;
                                        }
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                                    _ => {}
                                },
                                InputMode::AddingEventName => match key.code {
                                    KeyCode::Enter => {
                                        app.temp_event_name = app.input_buffer.clone();
                                        app.input_mode = InputMode::AddingEventTime;
                                        app.input_buffer.clear();
                                    }
                                    KeyCode::Char(c) => app.input_buffer.push(c),
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::EventManagement,
                                    _ => {}
                                },
                                InputMode::AddingEventTime => match key.code {
                                    KeyCode::Enter => {
                                        if let Some(time) = parse_time(&app.input_buffer) {
                                            app.temp_event_time = Some(time);
                                            app.input_mode = InputMode::AddingEventAutoPause;
                                            app.input_buffer.clear();
                                        } else {
                                            app.add_log(format!(
                                                "Invalid time format: {}",
                                                app.input_buffer
                                            ), Color::Red);
                                        }
                                    }
                                    KeyCode::Char(c) => app.input_buffer.push(c),
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::EventManagement,
                                    _ => {}
                                },
                                InputMode::AddingEventAutoPause => match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        app.temp_event_auto_pause = true;
                                        app.input_mode = InputMode::AddingEventRepeatInterval;
                                        app.input_buffer.clear();
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') => {
                                        app.temp_event_auto_pause = false;
                                        app.input_mode = InputMode::AddingEventRepeatInterval;
                                        app.input_buffer.clear();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::EventManagement,
                                    _ => {}
                                },
                                InputMode::AddingEventRepeatInterval => match key.code {
                                    KeyCode::Enter => {
                                        if app.input_buffer.is_empty() {
                                            app.temp_event_repeat_interval = None;
                                            if let Some(time) = app.temp_event_time {
                                                let color = app.get_random_color();
                                                app.events.push(TimedEvent {
                                                    time,
                                                    name: app.temp_event_name.clone(),
                                                    auto_pause: app.temp_event_auto_pause,
                                                    repeat_interval: None,
                                                    repeat_until: None,
                                                    color,
                                                });
                                                app.add_log(format!("Added event: {}", app.temp_event_name), color);
                                            }
                                            app.input_mode = InputMode::EventManagement;
                                        } else if let Ok(t) = NaiveTime::parse_from_str(&app.input_buffer, "%H:%M:%S") {
                                            let seconds = t.num_seconds_from_midnight();
                                            if seconds > 0 {
                                                let duration = Duration::seconds(seconds as i64);
                                                app.temp_event_repeat_interval = Some(duration);
                                                app.input_mode = InputMode::AddingEventRepeatUntil;
                                                app.input_buffer.clear();
                                            } else {
                                                app.add_log("Repeat interval must be greater than zero".to_string(), Color::Red);
                                            }
                                        } else {
                                            app.add_log(format!("Invalid interval format (HH:MM:SS): {}", app.input_buffer), Color::Red);
                                        }
                                    }
                                    KeyCode::Char(c) => app.input_buffer.push(c),
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::EventManagement,
                                    _ => {}
                                },
                                InputMode::AddingEventRepeatUntil => match key.code {
                                    KeyCode::Enter => {
                                        let mut repeat_until = None;
                                        let mut format_error = false;
                                        if !app.input_buffer.is_empty() {
                                            if let Some(until) = parse_time(&app.input_buffer) {
                                                repeat_until = Some(until);
                                            } else {
                                                app.add_log(format!("Invalid until format: {}", app.input_buffer), Color::Red);
                                                format_error = true;
                                            }
                                        }

                                        if !format_error {
                                            if let Some(time) = app.temp_event_time {
                                                let color = app.get_random_color();
                                                app.events.push(TimedEvent {
                                                    time,
                                                    name: app.temp_event_name.clone(),
                                                    auto_pause: app.temp_event_auto_pause,
                                                    repeat_interval: app.temp_event_repeat_interval,
                                                    repeat_until,
                                                    color,
                                                });
                                                app.add_log(format!("Added event: {}", app.temp_event_name), color);
                                            }
                                            app.input_mode = InputMode::EventManagement;
                                        }
                                    }
                                    KeyCode::Char(c) => app.input_buffer.push(c),
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::EventManagement,
                                    _ => {}
                                },
                                InputMode::SavingConfig | InputMode::SavingState => match key.code {
                                    KeyCode::Enter => {
                                        let is_config = app.input_mode == InputMode::SavingConfig;
                                        let suffix = if is_config {
                                            ".config.json"
                                        } else {
                                            ".state.json"
                                        };

                                        let filename = if app.input_buffer.ends_with(suffix) {
                                            app.input_buffer.clone()
                                        } else {
                                            format!("{}{}", app.input_buffer, suffix)
                                        };

                                        if std::path::Path::new(&filename).exists() {
                                            app.input_mode = if is_config {
                                                InputMode::ConfirmOverwriteConfig
                                            } else {
                                                InputMode::ConfirmOverwriteState
                                            };
                                        } else {
                                            if is_config {
                                                let config = ClockConfig {
                                                    initial_time: app.initial_time,
                                                    target_speed: app.target_speed,
                                                    events: app.events.clone(),
                                                };
                                                if let Ok(json) = serde_json::to_string_pretty(&config) {
                                                    if fs::write(&filename, json).is_ok() {
                                                        app.add_log(format!("Configuration saved to {}", filename), Color::White);
                                                    }
                                                }
                                            } else {
                                                let state = ClockState {
                                                    clock_time: app.clock_time,
                                                    initial_time: app.initial_time,
                                                    target_speed: app.target_speed,
                                                    paused: app.paused,
                                                    events: app.events.clone(),
                                                    logs: app.logs.clone(),
                                                };
                                                if let Ok(json) = serde_json::to_string_pretty(&state) {
                                                    if fs::write(&filename, json).is_ok() {
                                                        app.add_log(format!("State saved to {}", filename), Color::White);
                                                    }
                                                }
                                            }
                                            app.input_mode = InputMode::Normal;
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        app.input_buffer.push(c);
                                        let is_config = app.input_mode == InputMode::SavingConfig;
                                        let suffix = if is_config { ".config.json" } else { ".state.json" };
                                        let full_name = format!("{}{}", app.input_buffer, suffix);
                                        app.selected_file = app.files.iter().position(|f| f == &full_name);
                                    }
                                    KeyCode::Backspace => {
                                        app.input_buffer.pop();
                                        let is_config = app.input_mode == InputMode::SavingConfig;
                                        let suffix = if is_config { ".config.json" } else { ".state.json" };
                                        let full_name = format!("{}{}", app.input_buffer, suffix);
                                        app.selected_file = app.files.iter().position(|f| f == &full_name);
                                    }
                                    KeyCode::Up => {
                                        if !app.files.is_empty() {
                                            let current = app.selected_file.unwrap_or(0);
                                            let next = if current > 0 { current - 1 } else { 0 };
                                            app.selected_file = Some(next);
                                            let suffix = if let InputMode::SavingConfig = app.input_mode {
                                                ".config.json"
                                            } else {
                                                ".state.json"
                                            };
                                            let name = &app.files[next];
                                            app.input_buffer = name.strip_suffix(suffix).unwrap_or(name).to_string();
                                        }
                                    }
                                    KeyCode::Down => {
                                        if !app.files.is_empty() {
                                            let next = match app.selected_file {
                                                Some(i) if i < app.files.len() - 1 => i + 1,
                                                Some(i) => i,
                                                None => 0,
                                            };
                                            app.selected_file = Some(next);
                                            let suffix = if let InputMode::SavingConfig = app.input_mode {
                                                ".config.json"
                                            } else {
                                                ".state.json"
                                            };
                                            let name = &app.files[next];
                                            app.input_buffer = name.strip_suffix(suffix).unwrap_or(name).to_string();
                                        }
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                                    _ => {}
                                },
                                InputMode::ConfirmOverwriteConfig | InputMode::ConfirmOverwriteState => match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        let is_config = app.input_mode == InputMode::ConfirmOverwriteConfig;
                                        let suffix = if is_config { ".config.json" } else { ".state.json" };
                                        let filename = if app.input_buffer.ends_with(suffix) {
                                            app.input_buffer.clone()
                                        } else {
                                            format!("{}{}", app.input_buffer, suffix)
                                        };

                                        if is_config {
                                            let config = ClockConfig {
                                                initial_time: app.initial_time,
                                                target_speed: app.target_speed,
                                                events: app.events.clone(),
                                            };
                                            if let Ok(json) = serde_json::to_string_pretty(&config) {
                                                if fs::write(&filename, json).is_ok() {
                                                    app.add_log(format!("Configuration saved to {}", filename), Color::White);
                                                }
                                            }
                                        } else {
                                            let state = ClockState {
                                                clock_time: app.clock_time,
                                                initial_time: app.initial_time,
                                                target_speed: app.target_speed,
                                                paused: app.paused,
                                                events: app.events.clone(),
                                                logs: app.logs.clone(),
                                            };
                                            if let Ok(json) = serde_json::to_string_pretty(&state) {
                                                if fs::write(&filename, json).is_ok() {
                                                    app.add_log(format!("State saved to {}", filename), Color::White);
                                                }
                                            }
                                        }
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                        app.input_mode = if app.input_mode == InputMode::ConfirmOverwriteConfig {
                                            InputMode::SavingConfig
                                        } else {
                                            InputMode::SavingState
                                        };
                                    }
                                    _ => {}
                                },
                                InputMode::LoadingConfig | InputMode::LoadingState => match key.code {
                                    KeyCode::Enter => {
                                        if let Some(selected) = app.selected_file {
                                            let filename = &app.files[selected];
                                            if let InputMode::LoadingConfig = app.input_mode {
                                                if let Ok(json) = fs::read_to_string(filename) {
                                                    if let Ok(config) = serde_json::from_str::<ClockConfig>(&json) {
                                                        app.initial_time = config.initial_time;
                                                        app.clock_time = config.initial_time;
                                                        app.target_speed = config.target_speed;
                                                        app.events = config.events;
                                                        app.add_log(format!("Configuration loaded from {}", filename), Color::White);
                                                    }
                                                }
                                            } else {
                                                if let Ok(json) = fs::read_to_string(filename) {
                                                    if let Ok(state) = serde_json::from_str::<ClockState>(&json) {
                                                        app.clock_time = state.clock_time;
                                                        app.initial_time = state.initial_time;
                                                        app.target_speed = state.target_speed;
                                                        app.paused = state.paused;
                                                        app.events = state.events;
                                                        app.logs = state.logs;
                                                        app.add_log(format!("State loaded from {}", filename), Color::White);
                                                    }
                                                }
                                            }
                                        }
                                        app.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Up => {
                                        if !app.files.is_empty() {
                                            let current = app.selected_file.unwrap_or(0);
                                            let next = if current > 0 { current - 1 } else { 0 };
                                            app.selected_file = Some(next);
                                        }
                                    }
                                    KeyCode::Down => {
                                        if !app.files.is_empty() {
                                            let next = match app.selected_file {
                                                Some(i) if i < app.files.len() - 1 => i + 1,
                                                Some(i) => i,
                                                None => 0,
                                            };
                                            app.selected_file = Some(next);
                                        }
                                    }
                                    KeyCode::Esc => app.input_mode = InputMode::Normal,
                                    _ => {}
                                },
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        app.tick();
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    let status = match app.input_mode {
        InputMode::Normal => {
            if app.paused {
                "PAUSED"
            } else {
                "RUNNING"
            }
        }
        InputMode::Help => "HELP - Press 'h' or 'Esc' to close",
        InputMode::EditingTime => "SET TIME (HH:MM:SS)",
        InputMode::EditingSpeed => "SET SPEED (multiplier)",
        InputMode::EventManagement => "EVENT MANAGEMENT",
        InputMode::AddingEventName => "ADD EVENT: NAME",
        InputMode::AddingEventTime => "ADD EVENT: TIME (HH:MM:SS)",
        InputMode::AddingEventAutoPause => "ADD EVENT: AUTO-PAUSE? (y/n)",
        InputMode::AddingEventRepeatInterval => "ADD EVENT: REPEAT INTERVAL (HH:MM:SS, empty to skip)",
        InputMode::AddingEventRepeatUntil => "ADD EVENT: REPEAT UNTIL (HH:MM:SS)",
        InputMode::ConfirmOverwriteConfig => "CONFIRM OVERWRITE? (y/n)",
        InputMode::ConfirmOverwriteState => "CONFIRM OVERWRITE? (y/n)",
        InputMode::SavingConfig => "SAVE CONFIG: ENTER FILENAME",
        InputMode::SavingState => "SAVE STATE: ENTER FILENAME",
        InputMode::LoadingConfig => "LOAD CONFIG: SELECT FILE",
        InputMode::LoadingState => "LOAD STATE: SELECT FILE",
    };
    let clock_text = match app.input_mode {
        InputMode::Normal | InputMode::EventManagement | InputMode::LoadingConfig | InputMode::LoadingState | InputMode::Help | InputMode::ConfirmOverwriteConfig | InputMode::ConfirmOverwriteState => format!(
            "{} | {}",
            app.clock_time.format("%H:%M:%S%.3f"),
            status
        ),
        _ => format!("INPUT: {} | {}", app.input_buffer, status),
    };
    let clock_para = Paragraph::new(clock_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Digital Clock"));
    f.render_widget(clock_para, chunks[0]);

    let lower_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let (config_title, config_block_style) = if app.selected_section == Section::Config {
        ("Configuration Status (SELECTED)", Style::default().bg(Color::Rgb(30, 30, 60)))
    } else {
        ("Configuration Status", Style::default())
    };

    match app.input_mode {
        InputMode::EventManagement => {
            let events: Vec<ListItem> = app
                .events
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let mut text = format!("[{}] {} at {}", i, e.name, e.time.format("%H:%M:%S"));
                    if e.auto_pause {
                        text.push_str(" (Auto-pause)");
                    }
                    if let Some(interval) = e.repeat_interval {
                        let total_secs = interval.num_seconds();
                        let hours = total_secs / 3600;
                        let minutes = (total_secs % 3600) / 60;
                        let seconds = total_secs % 60;
                        text.push_str(&format!(" | Every {:02}:{:02}:{:02}", hours, minutes, seconds));
                        if let Some(until) = e.repeat_until {
                            let until_str = if until == NaiveTime::from_hms_opt(0, 0, 0).unwrap() {
                                "24:00:00".to_string()
                            } else {
                                until.format("%H:%M:%S").to_string()
                            };
                            text.push_str(&format!(" until {}", until_str));
                        }
                    }
                    let style = if i == app.selected_event {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(e.color)
                    };
                    ListItem::new(text).style(style)
                })
                .collect();
            let events_list =
                List::new(events).block(Block::default().borders(Borders::ALL).title("Configured Events").style(config_block_style));
            f.render_widget(events_list, lower_chunks[0]);
        }
        InputMode::LoadingConfig | InputMode::LoadingState | InputMode::SavingConfig | InputMode::SavingState | InputMode::ConfirmOverwriteConfig | InputMode::ConfirmOverwriteState => {
            let files: Vec<ListItem> = app
                .files
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let style = if Some(i) == app.selected_file {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(name.as_str()).style(style)
                })
                .collect();
            let title = match app.input_mode {
                InputMode::LoadingConfig => "Load Configuration",
                InputMode::LoadingState => "Load State",
                InputMode::SavingConfig | InputMode::ConfirmOverwriteConfig => "Save Configuration (Select to overwrite)",
                InputMode::SavingState | InputMode::ConfirmOverwriteState => "Save State (Select to overwrite)",
                _ => "",
            };
            let files_list = List::new(files)
                .block(Block::default().borders(Borders::ALL).title(title).style(config_block_style));
            f.render_widget(files_list, lower_chunks[0]);
        }
        _ => {
            let mut config_lines = vec![
                ListItem::new(format!("Initial Time: {}", app.initial_time.format("%H:%M:%S"))),
                ListItem::new(format!("Target Speed: {:.2}x", app.target_speed)),
                ListItem::new("Events:"),
            ];

            if app.events.is_empty() {
                config_lines.push(ListItem::new("  (None)"));
            } else {
                for (i, e) in app.events.iter().enumerate() {
                    let mut text = format!("  [{}] {} at {}", i, e.name, e.time.format("%H:%M:%S"));
                    if e.auto_pause {
                        text.push_str(" (Auto-pause)");
                    }
                    if let Some(interval) = e.repeat_interval {
                        let total_secs = interval.num_seconds();
                        let hours = total_secs / 3600;
                        let minutes = (total_secs % 3600) / 60;
                        let seconds = total_secs % 60;
                        text.push_str(&format!(" | Every {:02}:{:02}:{:02}", hours, minutes, seconds));
                        if let Some(until) = e.repeat_until {
                            let until_str = if until == NaiveTime::from_hms_opt(0, 0, 0).unwrap() {
                                "24:00:00".to_string()
                            } else {
                                until.format("%H:%M:%S").to_string()
                            };
                            text.push_str(&format!(" until {}", until_str));
                        }
                    }
                    config_lines.push(ListItem::new(text).style(Style::default().fg(e.color)));
                }
            }

            // Adjust scroll
            let max_scroll = config_lines.len().saturating_sub(1);
            let scroll = app.config_scroll.min(max_scroll);
            
            let scroll_indicator = if max_scroll > 0 {
                format!(" (Scroll: {}/{})", scroll, max_scroll)
            } else {
                "".to_string()
            };

            let visible_config: Vec<ListItem> = config_lines.into_iter().skip(scroll).collect();
            
            let config_list = List::new(visible_config)
                .block(Block::default().borders(Borders::ALL).title(format!("{}{}", config_title, scroll_indicator)).style(config_block_style));
            f.render_widget(config_list, lower_chunks[0]);
        }
    }

    let (log_title, log_block_style) = if app.selected_section == Section::Log {
        ("Event Log (SELECTED)", Style::default().bg(Color::Rgb(30, 30, 60)))
    } else {
        ("Event Log", Style::default())
    };

    let max_log_scroll = app.logs.len().saturating_sub(1);
    let log_scroll = app.log_scroll.min(max_log_scroll);
    
    let log_scroll_indicator = if max_log_scroll > 0 {
        format!(" (Scroll: {}/{})", log_scroll, max_log_scroll)
    } else {
        "".to_string()
    };

    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .skip(log_scroll)
        .map(|log| ListItem::new(log.message.as_str()).style(Style::default().fg(log.color)))
        .collect();
    let logs_list = List::new(logs)
        .block(Block::default().borders(Borders::ALL).title(format!("{}{}", log_title, log_scroll_indicator)).style(log_block_style));
    f.render_widget(logs_list, lower_chunks[1]);

    // Footnotes or extra status
    let status_text = format!("Mode: {:<20}", match app.input_mode {
        InputMode::Normal => "Normal",
        InputMode::EditingTime => "Editing Time",
        InputMode::EditingSpeed => "Editing Speed",
        InputMode::EventManagement => "Event Management",
        InputMode::AddingEventName => "Adding Event (Name)",
        InputMode::AddingEventTime => "Adding Event (Time)",
        InputMode::AddingEventAutoPause => "Adding Event (Auto-Pause)",
        InputMode::AddingEventRepeatInterval => "Adding Event (Repeat Interval)",
        InputMode::AddingEventRepeatUntil => "Adding Event (Repeat Until)",
        InputMode::SavingConfig => "Saving Config",
        InputMode::LoadingConfig => "Loading Config",
        InputMode::SavingState => "Saving State",
        InputMode::LoadingState => "Loading State",
        InputMode::ConfirmOverwriteConfig => "Confirm Overwrite (Config)",
        InputMode::ConfirmOverwriteState => "Confirm Overwrite (State)",
        InputMode::Help => "Help Overlay",
    });

    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)].as_ref())
        .split(chunks[2].inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 }));

    let mode_para = Paragraph::new(status_text);
    let help_para = Paragraph::new("press h for help")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Right);

    f.render_widget(Block::default().borders(Borders::ALL).title("Status"), chunks[2]);
    f.render_widget(mode_para, status_layout[0]);
    f.render_widget(help_para, status_layout[1]);

    if let InputMode::Help = app.input_mode {
        render_help_overlay(f, app);
    }

    if let InputMode::ConfirmOverwriteConfig | InputMode::ConfirmOverwriteState = app.input_mode {
        render_confirm_overwrite_overlay(f);
    }
}

fn render_confirm_overwrite_overlay(f: &mut Frame) {
    let block = Block::default()
        .title("Confirm Overwrite")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    
    let text = Paragraph::new("\nOVERWRITE EXISTING FILE?\n\n(y)es / (n)o")
        .alignment(ratatui::layout::Alignment::Center)
        .block(block);
    f.render_widget(text, area);
}

pub const HELP_TEXT: &'static str = "
    SCROLLING:
    - Left/Right: Switch between Config Status and Event Log
    - Up/Down: Scroll selected section

    CLOCK CONTROLS:
    - Space: Pause/Resume
    - v: Set Speed (multiplier)
    - r: Reset to initial time
    - t: Set Time (HH:MM:SS)
    - e: Event Management Mode
    - S: Save Configuration (events + start time + speed)
    - L: Load Configuration
    - s: Save State (current time + events + speed)
    - l: Load State
    - h: Show/Hide Help
    - q: Quit

    EVENT MANAGEMENT:
    - a: Add new event
    - d: Delete selected event
    - Up/Down: Navigate list
    - Esc: Return to Clock view

    MISC:
    - Press 'h' or 'Esc' to close this menu.
    ";

fn render_help_overlay(f: &mut Frame, app: &App) {
    let help_lines: Vec<&str> = HELP_TEXT.lines().collect();
    let total_lines = help_lines.len();
    
    // Calculate visible area height
    let area = centered_rect(60, 60, f.area());
    let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders

    let scroll = app.help_scroll.min(total_lines.saturating_sub(visible_height));
    let visible_help_text = help_lines
        .iter()
        .skip(scroll)
        .take(visible_height)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    let scroll_indicator = if total_lines > visible_height {
        format!(" (Scroll: {}/{})", scroll, total_lines.saturating_sub(visible_height))
    } else {
        "".to_string()
    };

    let block = Block::default()
        .title(format!("Controls Help{}", scroll_indicator))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(Paragraph::new(visible_help_text).block(block), area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
