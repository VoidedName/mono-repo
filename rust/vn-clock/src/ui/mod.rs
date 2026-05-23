use chrono::NaiveTime;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::models::{App, InputMode, Section};

pub const HELP_TEXT: &str = "
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

pub fn ui(f: &mut Frame, app: &App) {
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

pub fn render_confirm_overwrite_overlay(f: &mut Frame) {
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

pub fn render_help_overlay(f: &mut Frame, app: &App) {
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

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
