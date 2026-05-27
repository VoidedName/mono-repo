pub mod color_serde;
use rodio::Sink;
use std::sync::{Arc, Mutex};
use vn_clock_core::models::CoreApp;
use vn_clock_core::input_flow::InputFlowState;

/// UI sections that can be focused and scrolled.
#[derive(PartialEq, Clone, Copy)]
pub enum Section {
    Config,
    Log,
}

/// The current interactive mode of the TUI application.
#[derive(PartialEq, Clone)]
pub enum InputMode {
    Normal,
    InputFlow,
    EventManagement,
    Help,
    LoadingConfig,
    SavingConfig,
    LoadingState,
    SavingState,
    ConfirmOverwriteConfig(String),
    ConfirmOverwriteState(String),
}

/// Main state for the TUI application.
pub struct App {
    pub core: CoreApp,
    pub io: AppIo,
    pub input_mode: InputMode,
    pub input_flow: InputFlowState,
    /// Index of the selected event in Event Management mode.
    pub selected_event: usize,
    pub config_scroll: usize,
    pub log_scroll: usize,
    pub help_scroll: usize,
    pub selected_section: Section,
    /// List of files found in the current directory for the file explorer.
    pub files: Vec<String>,
    pub selected_file: Option<usize>,
}

/// Platform-specific IO implementation for the TUI (Audio).
#[derive(Clone)]
pub struct AppIo {
    pub sink: Arc<Mutex<Option<Sink>>>,
}

impl AppIo {
    /// Plays a high-pitched "ding" sound using `rodio`.
    pub fn play_ding(&self) {
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                use rodio::source::Source;
                let source = rodio::source::SineWave::new(440.0)
                    .take_duration(std::time::Duration::from_millis(500))
                    .amplify(0.5);
                sink.append(source);
            }
        }
    }
}
