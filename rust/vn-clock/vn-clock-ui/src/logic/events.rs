use chrono::{Duration, NaiveTime, Timelike};
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use vn_clock_core::models::{ClockConfig, ClockState, TimedEvent, ClockColor, ClockEvent, ClockOutputEvent};
use crate::models::{App, InputMode, Section};
use crate::utils::parse_time;
use crate::ui::ui;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
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
                                InputMode::Normal => handle_normal_input(&mut app, key.code),
                                InputMode::Help => handle_help_input(&mut app, key.code, terminal)?,
                                InputMode::EditingTime => handle_editing_time_input(&mut app, key.code),
                                InputMode::EditingSpeed => handle_editing_speed_input(&mut app, key.code),
                                InputMode::EventManagement => handle_event_management_input(&mut app, key.code),
                                InputMode::AddingEventName => handle_adding_event_name_input(&mut app, key.code),
                                InputMode::AddingEventTime => handle_adding_event_time_input(&mut app, key.code),
                                InputMode::AddingEventAutoPause => handle_adding_event_auto_pause_input(&mut app, key.code),
                                InputMode::AddingEventRepeatInterval => handle_adding_event_repeat_interval_input(&mut app, key.code),
                                InputMode::AddingEventRepeatUntil => handle_adding_event_repeat_until_input(&mut app, key.code),
                                InputMode::LoadingConfig | InputMode::SavingConfig | InputMode::LoadingState | InputMode::SavingState => {
                                    handle_file_input(&mut app, key.code)
                                }
                                InputMode::ConfirmOverwriteConfig(_) | InputMode::ConfirmOverwriteState(_) => {
                                    handle_confirm_overwrite_input(&mut app, key.code)
                                }
                            }

                            if let InputMode::Normal = app.input_mode {
                                if let KeyCode::Char('q') = key.code {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        app.core.tick();
        
        // Handle output events from core
        let output_events = app.core.take_output_events();
        for event in output_events {
            match event {
                ClockOutputEvent::Ding => {
                    app.io.play_ding();
                }
                _ => {} // Other events are currently handled by core updating its own state
            }
        }
    }
}

fn handle_normal_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char(' ') => app.core.handle_event(ClockEvent::TogglePause),
        KeyCode::Char('v') => {
            app.input_mode = InputMode::EditingSpeed;
            app.input_buffer.clear();
        }
        KeyCode::Char('e') => {
            app.input_mode = InputMode::EventManagement;
            app.selected_event = 0;
        }
        KeyCode::Char('r') => {
            app.core.handle_event(ClockEvent::Reset);
        }
        KeyCode::Char('t') => {
            app.input_mode = InputMode::EditingTime;
            app.input_buffer.clear();
            if !app.core.paused() {
                app.core.handle_event(ClockEvent::TogglePause);
            }
        }
        KeyCode::Char('S') => {
            app.input_mode = InputMode::SavingConfig;
            app.input_buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&["clockcfg"]);
        }
        KeyCode::Char('L') => {
            app.input_mode = InputMode::LoadingConfig;
            app.input_buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&["clockcfg"]);
        }
        KeyCode::Char('s') => {
            app.input_mode = InputMode::SavingState;
            app.input_buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&["clockstate"]);
        }
        KeyCode::Char('l') => {
            app.input_mode = InputMode::LoadingState;
            app.input_buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&["clockstate"]);
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
                    let config_lines_count = if app.core.events().is_empty() {
                        4 // Initial Time, Target Speed, Events, (None)
                    } else {
                        3 + app.core.events().len() // Initial Time, Target Speed, Events, + each event
                    };
                    let max_scroll = config_lines_count.saturating_sub(1);
                    if app.config_scroll < max_scroll {
                        app.config_scroll += 1;
                    }
                }
                Section::Log => {
                    let max_scroll = app.core.logs().len().saturating_sub(1);
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
    }
}

fn handle_help_input<B: Backend>(app: &mut App, code: KeyCode, terminal: &Terminal<B>) -> io::Result<()> {
    match code {
        KeyCode::Char('h') | KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Up => {
            if app.help_scroll > 0 {
                app.help_scroll -= 1;
            }
        }
        KeyCode::Down => {
            let help_lines: Vec<&str> = crate::ui::HELP_TEXT.lines().collect();
            let total_lines = help_lines.len();

            let height = terminal.size()?.height;
            let area_height = (height * 60) / 100;
            let visible_height = area_height.saturating_sub(2) as usize;

            let max_scroll = total_lines.saturating_sub(visible_height);
            if app.help_scroll < max_scroll {
                app.help_scroll += 1;
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_editing_time_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if let Some(time) = parse_time(&app.input_buffer) {
                app.core.handle_event(ClockEvent::SetTime(time));
            } else {
                app.core.add_log(format!(
                    "Invalid time format: {}",
                    app.input_buffer
                ), ClockColor::Red);
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        _ => {}
    }
}

fn handle_editing_speed_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if let Ok(speed) = app.input_buffer.parse::<f64>() {
                app.core.handle_event(ClockEvent::SetSpeed(speed));
            } else {
                app.core.add_log(format!(
                    "Invalid speed: {}",
                    app.input_buffer
                ), ClockColor::Red);
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        _ => {}
    }
}

fn handle_event_management_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('a') => {
            app.input_mode = InputMode::AddingEventName;
            app.input_buffer.clear();
            app.temp_event_name.clear();
            app.temp_event_time = None;
            app.temp_event_auto_pause = false;
        }
        KeyCode::Char('d') => {
            if !app.core.events().is_empty() && app.selected_event < app.core.events().len() {
                app.core.handle_event(ClockEvent::RemoveTimedEvent(app.selected_event));
                if app.selected_event >= app.core.events().len() && !app.core.events().is_empty() {
                    app.selected_event = app.core.events().len() - 1;
                }
            }
        }
        KeyCode::Up => {
            if app.selected_event > 0 {
                app.selected_event -= 1;
            }
        }
        KeyCode::Down => {
            if !app.core.events().is_empty() && app.selected_event < app.core.events().len() - 1 {
                app.selected_event += 1;
            }
        }
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        _ => {}
    }
}

fn handle_adding_event_name_input(app: &mut App, code: KeyCode) {
    match code {
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
    }
}

fn handle_adding_event_time_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if let Some(time) = parse_time(&app.input_buffer) {
                app.temp_event_time = Some(time);
                app.input_mode = InputMode::AddingEventAutoPause;
                app.input_buffer.clear();
            } else {
                app.core.add_log(format!(
                    "Invalid time format: {}",
                    app.input_buffer
                ), ClockColor::Red);
            }
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Esc => app.input_mode = InputMode::EventManagement,
        _ => {}
    }
}

fn handle_adding_event_auto_pause_input(app: &mut App, code: KeyCode) {
    match code {
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
    }
}

fn handle_adding_event_repeat_interval_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if app.input_buffer.is_empty() {
                app.temp_event_repeat_interval = None;
                if let Some(time) = app.temp_event_time {
                    let color = app.get_random_color();
                    app.core.handle_event(ClockEvent::AddTimedEvent(TimedEvent {
                        time,
                        name: app.temp_event_name.clone(),
                        auto_pause: app.temp_event_auto_pause,
                        repeat_interval: None,
                        repeat_until: None,
                        color,
                    }));
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
                    app.core.add_log("Repeat interval must be greater than zero".to_string(), ClockColor::Red);
                }
            } else {
                app.core.add_log(format!("Invalid interval format (HH:MM:SS): {}", app.input_buffer), ClockColor::Red);
            }
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Esc => app.input_mode = InputMode::EventManagement,
        _ => {}
    }
}

fn handle_adding_event_repeat_until_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            let mut repeat_until = None;
            let mut format_error = false;
            if !app.input_buffer.is_empty() {
                if let Some(until) = parse_time(&app.input_buffer) {
                    repeat_until = Some(until);
                } else {
                    app.core.add_log(format!("Invalid until format: {}", app.input_buffer), ClockColor::Red);
                    format_error = true;
                }
            }

            if !format_error {
                if let Some(time) = app.temp_event_time {
                    let color = app.get_random_color();
                    app.core.handle_event(ClockEvent::AddTimedEvent(TimedEvent {
                        time,
                        name: app.temp_event_name.clone(),
                        auto_pause: app.temp_event_auto_pause,
                        repeat_interval: app.temp_event_repeat_interval,
                        repeat_until,
                        color,
                    }));
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
    }
}

fn handle_file_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up => {
            if !app.files.is_empty() {
                let current = app.selected_file.unwrap_or(0);
                if current > 0 {
                    app.selected_file = Some(current - 1);
                } else {
                    app.selected_file = Some(app.files.len() - 1);
                }
                if let Some(selected) = app.selected_file {
                    app.input_buffer = app.files[selected].clone();
                }
            }
        }
        KeyCode::Down => {
            if !app.files.is_empty() {
                let next = match app.selected_file {
                    Some(current) => (current + 1) % app.files.len(),
                    None => 0,
                };
                app.selected_file = Some(next);
                app.input_buffer = app.files[next].clone();
            }
        }
        KeyCode::Enter => {
            let filename = app.input_buffer.trim().to_string();
            if filename.is_empty() {
                return;
            }

            match app.input_mode {
                InputMode::LoadingConfig => {
                    let filename = if filename.ends_with(".clockcfg") {
                        filename
                    } else {
                        format!("{}.clockcfg", filename)
                    };
                    if let Ok(bytes) = std::fs::read(&filename) {
                        if let Ok(config) = serde_json::from_slice::<ClockConfig>(&bytes) {
                            app.core.handle_event(ClockEvent::LoadConfig(config));
                            app.core.add_log(format!("Configuration loaded from {}", filename), ClockColor::White);
                        } else {
                            app.core.add_log("Failed to parse config".to_string(), ClockColor::Red);
                        }
                    } else {
                        app.core.add_log(format!("Failed to read file {}", filename), ClockColor::Red);
                    }
                }
                InputMode::SavingConfig => {
                    let filename = if filename.ends_with(".clockcfg") {
                        filename
                    } else {
                        format!("{}.clockcfg", filename)
                    };
                    if std::path::Path::new(&filename).exists() {
                        app.input_mode = InputMode::ConfirmOverwriteConfig(filename);
                        return;
                    }
                    save_config(app, &filename);
                }
                InputMode::LoadingState => {
                    let filename = if filename.ends_with(".clockstate") {
                        filename
                    } else {
                        format!("{}.clockstate", filename)
                    };
                    if let Ok(bytes) = std::fs::read(&filename) {
                        if let Ok(state) = serde_json::from_slice::<ClockState>(&bytes) {
                            app.core.handle_event(ClockEvent::LoadState(state));
                            app.core.add_log(format!("State loaded from {}", filename), ClockColor::White);
                        } else {
                            app.core.add_log("Failed to parse state".to_string(), ClockColor::Red);
                        }
                    } else {
                        app.core.add_log(format!("Failed to read file {}", filename), ClockColor::Red);
                    }
                }
                InputMode::SavingState => {
                    let filename = if filename.ends_with(".clockstate") {
                        filename
                    } else {
                        format!("{}.clockstate", filename)
                    };
                    if std::path::Path::new(&filename).exists() {
                        app.input_mode = InputMode::ConfirmOverwriteState(filename);
                        return;
                    }
                    save_state(app, &filename);
                }
                _ => {}
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

fn get_files_with_extensions(extensions: &[&str]) -> Vec<String> {
    std::fs::read_dir(".")
        .map(|rd| {
            rd.filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        if extensions.contains(&ext) {
                            return path.file_name().map(|n| n.to_string_lossy().to_string());
                        }
                    }
                    None
                })
            })
            .collect()
        })
        .unwrap_or_default()
}

fn handle_confirm_overwrite_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let mode = app.input_mode.clone();
            match mode {
                InputMode::ConfirmOverwriteConfig(filename) => {
                    save_config(app, &filename);
                }
                InputMode::ConfirmOverwriteState(filename) => {
                    save_state(app, &filename);
                }
                _ => {}
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            match app.input_mode {
                InputMode::ConfirmOverwriteConfig(_) => app.input_mode = InputMode::SavingConfig,
                InputMode::ConfirmOverwriteState(_) => app.input_mode = InputMode::SavingState,
                _ => app.input_mode = InputMode::Normal,
            }
        }
        _ => {}
    }
}

fn save_config(app: &mut App, filename: &str) {
    let config = ClockConfig {
        initial_time: app.core.initial_time(),
        target_speed: app.core.target_speed(),
        events: app.core.events().to_vec(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        if std::fs::write(filename, json).is_ok() {
            app.core.add_log(format!("Configuration saved to {}", filename), ClockColor::White);
        } else {
            app.core.add_log("Failed to save config".to_string(), ClockColor::Red);
        }
    }
}

fn save_state(app: &mut App, filename: &str) {
    let state = ClockState {
        clock_time: app.core.clock_time(),
        initial_time: app.core.initial_time(),
        target_speed: app.core.target_speed(),
        paused: app.core.paused(),
        events: app.core.events().to_vec(),
        logs: app.core.logs().to_vec(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        if std::fs::write(filename, json).is_ok() {
            app.core.add_log(format!("State saved to {}", filename), ClockColor::White);
        } else {
            app.core.add_log("Failed to save state".to_string(), ClockColor::Red);
        }
    }
}
