use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use vn_clock_core::models::{ClockColor, ClockEvent, ClockOutputEvent};
use vn_clock_core::input_flow::{InputFlow, InputFlowResult};
use vn_clock_core::persistence;
use crate::models::{App, InputMode, Section};
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
                                InputMode::InputFlow => handle_input_flow_input(&mut app, key.code),
                                InputMode::EventManagement => handle_event_management_input(&mut app, key.code),
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
            handle_output_event(&mut app, event);
        }
    }
}

fn handle_output_event(app: &mut App, event: ClockOutputEvent) {
    match event {
        ClockOutputEvent::Ding => {
            app.io.play_ding();
        }
        ClockOutputEvent::Log(_entry) => {
            // Logs are already added to app.core.logs by handle_event, 
            // but we could use this for extra UI notifications if needed.
        }
        ClockOutputEvent::Paused(_) | ClockOutputEvent::TimeSet(_) | ClockOutputEvent::SpeedSet(_) => {
            // These are state changes that the UI reflects in its next draw
        }
    }
}

fn handle_normal_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char(' ') => app.core.handle_event(ClockEvent::TogglePause),
        KeyCode::Char('v') => {
            app.input_mode = InputMode::InputFlow;
            app.input_flow.start_editing_speed();
        }
        KeyCode::Char('e') => {
            app.input_mode = InputMode::EventManagement;
            app.selected_event = 0;
        }
        KeyCode::Char('r') => {
            app.core.handle_event(ClockEvent::Reset);
        }
        KeyCode::Char('t') => {
            app.input_mode = InputMode::InputFlow;
            app.input_flow.start_editing_time();
            if !app.core.paused() {
                app.core.handle_event(ClockEvent::TogglePause);
            }
        }
        KeyCode::Char('S') => {
            app.input_mode = InputMode::SavingConfig;
            app.input_flow.buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&[persistence::CONFIG_EXTENSION]);
        }
        KeyCode::Char('L') => {
            app.input_mode = InputMode::LoadingConfig;
            app.input_flow.buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&[persistence::CONFIG_EXTENSION]);
        }
        KeyCode::Char('s') => {
            app.input_mode = InputMode::SavingState;
            app.input_flow.buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&[persistence::STATE_EXTENSION]);
        }
        KeyCode::Char('l') => {
            app.input_mode = InputMode::LoadingState;
            app.input_flow.buffer.clear();
            app.selected_file = None;
            app.files = get_files_with_extensions(&[persistence::STATE_EXTENSION]);
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

fn handle_event_management_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('a') => {
            app.input_mode = InputMode::InputFlow;
            app.input_flow.start_adding_event();
        }
        KeyCode::Char('d') => {
            if !app.core.events().is_empty() && app.selected_event < app.core.events().len() {
                let event_id = app.core.events()[app.selected_event].id;
                app.core.handle_event(ClockEvent::RemoveTimedEvent(event_id));
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

fn handle_input_flow_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            let was_event_mgmt = matches!(app.input_flow.flow, InputFlow::AddingEvent(_));
            match app.input_flow.handle_input(&app.input_flow.buffer.clone()) {
                InputFlowResult::Completed(event) => {
                    app.input_flow.buffer.clear();
                    app.core.handle_event(event);
                    app.input_mode = if was_event_mgmt {
                        InputMode::EventManagement
                    } else {
                        InputMode::Normal
                    };
                }
                InputFlowResult::NextStep => {
                    app.input_flow.buffer.clear();
                }
                InputFlowResult::Error(err) => {
                    app.core.add_log(err, ClockColor::Red);
                }
            }
        }
        KeyCode::Char(c) => app.input_flow.buffer.push(c),
        KeyCode::Backspace => {
            app.input_flow.buffer.pop();
        }
        KeyCode::Esc => {
            let was_event_mgmt = matches!(app.input_flow.flow, InputFlow::AddingEvent(_));
            app.input_flow.cancel();
            app.input_mode = if was_event_mgmt { InputMode::EventManagement } else { InputMode::Normal };
        }
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
                    app.input_flow.buffer = app.files[selected].clone();
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
                app.input_flow.buffer = app.files[next].clone();
            }
        }
        KeyCode::Enter => {
            let filename = app.input_flow.buffer.trim().to_string();
            if filename.is_empty() {
                return;
            }

            match app.input_mode {
                InputMode::LoadingConfig => {
                    let filename = persistence::ensure_extension(&filename, persistence::CONFIG_EXTENSION);
                    if let Ok(json) = std::fs::read_to_string(&filename) {
                        if let Err(e) = app.core.load_config_json(&json) {
                            app.core.add_log(e, ClockColor::Red);
                        } else {
                            app.core.add_log(format!("Configuration loaded from {}", filename), ClockColor::White);
                        }
                    } else {
                        app.core.add_log(format!("Failed to read file {}", filename), ClockColor::Red);
                    }
                }
                InputMode::SavingConfig => {
                    let filename = persistence::ensure_extension(&filename, persistence::CONFIG_EXTENSION);
                    if std::path::Path::new(&filename).exists() {
                        app.input_mode = InputMode::ConfirmOverwriteConfig(filename);
                        return;
                    }
                    save_config(app, &filename);
                }
                InputMode::LoadingState => {
                    let filename = persistence::ensure_extension(&filename, persistence::STATE_EXTENSION);
                    if let Ok(json) = std::fs::read_to_string(&filename) {
                        if let Err(e) = app.core.load_state_json(&json) {
                            app.core.add_log(e, ClockColor::Red);
                        } else {
                            app.core.add_log(format!("State loaded from {}", filename), ClockColor::White);
                        }
                    } else {
                        app.core.add_log(format!("Failed to read file {}", filename), ClockColor::Red);
                    }
                }
                InputMode::SavingState => {
                    let filename = persistence::ensure_extension(&filename, persistence::STATE_EXTENSION);
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
            app.input_flow.buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_flow.buffer.pop();
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
    match app.core.get_config_json() {
        Ok(json) => {
            if std::fs::write(filename, json).is_ok() {
                app.core.add_log(format!("Configuration saved to {}", filename), ClockColor::White);
            } else {
                app.core.add_log("Failed to write config file".to_string(), ClockColor::Red);
            }
        }
        Err(e) => app.core.add_log(e, ClockColor::Red),
    }
}

fn save_state(app: &mut App, filename: &str) {
    match app.core.get_state_json() {
        Ok(json) => {
            if std::fs::write(filename, json).is_ok() {
                app.core.add_log(format!("State saved to {}", filename), ClockColor::White);
            } else {
                app.core.add_log("Failed to write state file".to_string(), ClockColor::Red);
            }
        }
        Err(e) => app.core.add_log(e, ClockColor::Red),
    }
}
