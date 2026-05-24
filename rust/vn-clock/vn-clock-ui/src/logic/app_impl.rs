use vn_clock_core::models::ClockColor;
use crate::models::{App, InputMode, Section, AppIo};
use vn_clock_core::models::CoreApp;
use std::sync::{Arc, Mutex};

impl App {
    pub fn new() -> Self {
        Self {
            core: CoreApp::new(),
            io: AppIo {
                sink: Arc::new(Mutex::new(None)),
            },
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            selected_event: 0,
            temp_event_name: String::new(),
            temp_event_time: None,
            temp_event_auto_pause: false,
            temp_event_repeat_interval: None,
            config_scroll: 0,
            log_scroll: 0,
            help_scroll: 0,
            selected_section: Section::Log,
            files: Vec::new(),
            selected_file: None,
        }
    }

    pub fn get_random_color(&self) -> ClockColor {
        let colors = [
            ClockColor::Red,
            ClockColor::Green,
            ClockColor::Yellow,
            ClockColor::Blue,
            ClockColor::Magenta,
            ClockColor::Cyan,
            ClockColor::LightRed,
            ClockColor::LightGreen,
            ClockColor::LightYellow,
            ClockColor::LightBlue,
            ClockColor::LightMagenta,
            ClockColor::LightCyan,
        ];
        colors[self.core.events.len() % colors.len()]
    }
}
