pub mod color_serde;
use chrono::{Duration, NaiveTime};
use ratatui::style::Color;
use rodio::Sink;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedEvent {
    pub time: NaiveTime,
    pub name: String,
    pub auto_pause: bool,
    pub repeat_interval: Option<Duration>,
    pub repeat_until: Option<NaiveTime>,
    #[serde(with = "color_serde")]
    pub color: Color,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub message: String,
    #[serde(with = "color_serde")]
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct ClockConfig {
    pub initial_time: NaiveTime,
    pub target_speed: f64,
    pub events: Vec<TimedEvent>,
}

#[derive(Serialize, Deserialize)]
pub struct ClockState {
    pub clock_time: NaiveTime,
    pub initial_time: NaiveTime,
    pub target_speed: f64,
    pub paused: bool,
    pub events: Vec<TimedEvent>,
    pub logs: Vec<LogEntry>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Section {
    Config,
    Log,
}

#[derive(PartialEq, Clone, Copy)]
pub enum InputMode {
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

pub struct App {
    pub clock_time: NaiveTime,
    pub initial_time: NaiveTime,
    pub speed: f64,
    pub paused: bool,
    pub events: Vec<TimedEvent>,
    pub logs: Vec<LogEntry>,
    pub last_tick: Instant,
    pub sink: Arc<Mutex<Option<Sink>>>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub target_speed: f64,
    pub selected_event: usize,
    // Temporary state for adding a new event
    pub temp_event_name: String,
    pub temp_event_time: Option<NaiveTime>,
    pub temp_event_auto_pause: bool,
    pub temp_event_repeat_interval: Option<Duration>,
    // File explorer state
    pub files: Vec<String>,
    pub selected_file: Option<usize>,
    pub config_scroll: usize,
    pub log_scroll: usize,
    pub help_scroll: usize,
    pub selected_section: Section,
}
