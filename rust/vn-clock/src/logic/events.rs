use chrono::{Duration, NaiveTime, Timelike};
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::Backend;
use ratatui::style::Color;
use ratatui::Terminal;
use std::{fs, io};
use crate::models::{App, ClockConfig, ClockState, InputMode, Section, TimedEvent};
use crate::utils::parse_time;
use crate::ui::{ui, HELP_TEXT};

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
                                InputMode::SavingConfig | InputMode::SavingState => handle_saving_input(&mut app, key.code),
                                InputMode::ConfirmOverwriteConfig | InputMode::ConfirmOverwriteState => handle_confirm_overwrite_input(&mut app, key.code),
                                InputMode::LoadingConfig | InputMode::LoadingState => handle_loading_input(&mut app, key.code),
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
        app.tick();
    }
}

fn handle_normal_input(app: &mut App, code: KeyCode) {
    match code {
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
            let help_lines: Vec<&str> = HELP_TEXT.lines().collect();
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
    }
}

fn handle_editing_speed_input(app: &mut App, code: KeyCode) {
    match code {
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
    }
}

fn handle_saving_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            let is_config = app.input_mode == InputMode::SavingConfig;
            let suffix = if is_config { ".config.json" } else { ".state.json" };
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
                save_to_file(app, &filename, is_config);
                app.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
            update_selected_file(app);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
            update_selected_file(app);
        }
        KeyCode::Up => {
            if !app.files.is_empty() {
                let current = app.selected_file.unwrap_or(0);
                let next = if current > 0 { current - 1 } else { 0 };
                app.selected_file = Some(next);
                update_buffer_from_selected(app);
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
                update_buffer_from_selected(app);
            }
        }
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        _ => {}
    }
}

fn handle_confirm_overwrite_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let is_config = app.input_mode == InputMode::ConfirmOverwriteConfig;
            let suffix = if is_config { ".config.json" } else { ".state.json" };
            let filename = if app.input_buffer.ends_with(suffix) {
                app.input_buffer.clone()
            } else {
                format!("{}{}", app.input_buffer, suffix)
            };
            save_to_file(app, &filename, is_config);
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
    }
}

fn handle_loading_input(app: &mut App, code: KeyCode) {
    match code {
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
    }
}

fn update_selected_file(app: &mut App) {
    let suffix = if let InputMode::SavingConfig = app.input_mode { ".config.json" } else { ".state.json" };
    let full_name = format!("{}{}", app.input_buffer, suffix);
    app.selected_file = app.files.iter().position(|f| f == &full_name);
}

fn update_buffer_from_selected(app: &mut App) {
    if let Some(selected) = app.selected_file {
        let suffix = if let InputMode::SavingConfig = app.input_mode { ".config.json" } else { ".state.json" };
        let name = &app.files[selected];
        app.input_buffer = name.strip_suffix(suffix).unwrap_or(name).to_string();
    }
}

fn save_to_file(app: &mut App, filename: &str, is_config: bool) {
    if is_config {
        let config = ClockConfig {
            initial_time: app.initial_time,
            target_speed: app.target_speed,
            events: app.events.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if fs::write(filename, json).is_ok() {
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
            if fs::write(filename, json).is_ok() {
                app.add_log(format!("State saved to {}", filename), Color::White);
            }
        }
    }
}
