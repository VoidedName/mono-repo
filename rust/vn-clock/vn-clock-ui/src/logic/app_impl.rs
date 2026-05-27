use crate::models::{App, InputMode, Section, AppIo};
use vn_clock_core::models::CoreApp;
use vn_clock_core::input_flow::InputFlowState;
use std::sync::{Arc, Mutex};

impl App {
    pub fn new() -> Self {
        Self {
            core: CoreApp::new(),
            io: AppIo {
                sink: Arc::new(Mutex::new(None)),
            },
            input_mode: InputMode::Normal,
            input_flow: InputFlowState::new(),
            selected_event: 0,
            config_scroll: 0,
            log_scroll: 0,
            help_scroll: 0,
            selected_section: Section::Log,
            files: Vec::new(),
            selected_file: None,
        }
    }
}
