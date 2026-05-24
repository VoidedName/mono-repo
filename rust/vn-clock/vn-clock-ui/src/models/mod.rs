pub mod color_serde;
use chrono::{Duration, NaiveTime};
use rodio::Sink;
use std::sync::{Arc, Mutex};
use vn_clock_core::models::CoreApp;

#[derive(PartialEq, Clone, Copy)]
pub enum Section {
    Config,
    Log,
}

#[derive(PartialEq, Clone)]
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
    Help,
    LoadingConfig,
    SavingConfig,
    LoadingState,
    SavingState,
    ConfirmOverwriteConfig(String),
    ConfirmOverwriteState(String),
}

pub struct App {
    pub core: CoreApp,
    pub io: AppIo,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub selected_event: usize,
    // Temporary state for adding a new event
    pub temp_event_name: String,
    pub temp_event_time: Option<NaiveTime>,
    pub temp_event_auto_pause: bool,
    pub temp_event_repeat_interval: Option<Duration>,
    pub config_scroll: usize,
    pub log_scroll: usize,
    pub help_scroll: usize,
    pub selected_section: Section,
    pub files: Vec<String>,
    pub selected_file: Option<usize>,
}

#[derive(Clone)]
pub struct AppIo {
    pub sink: Arc<Mutex<Option<Sink>>>,
}

impl AppIo {
    pub fn play_ding(&self) {
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                use rodio::source::Source;
                let source = rodio::source::SineWave::new(440.0)
                    .take_duration(std::time::Duration::from_millis(200))
                    .amplify(0.2);
                sink.append(source);
            }
        }
    }
}
