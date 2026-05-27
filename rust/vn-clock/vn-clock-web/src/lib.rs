//! WebAssembly bridge for the VN Clock, providing a JS-friendly API.

use wasm_bindgen::prelude::*;
use vn_clock_core::models::{CoreApp, ClockEvent};
use vn_clock_core::utils::parse_time;
use vn_clock_core::input_flow::{InputFlowState, InputFlow, InputFlowResult};
use vn_clock_core::persistence;
use serde::Serialize;

/// The main application container for the web environment.
#[wasm_bindgen]
pub struct WebApp {
    core: CoreApp,
    input_flow: InputFlowState,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
        Self {
            core: CoreApp::new(),
            input_flow: InputFlowState::new(),
        }
    }

    /// Starts a specific multi-step input flow from JavaScript.
    pub fn start_input(&mut self, flow_name: &str) {
        match flow_name {
            "EditingTime" => self.input_flow.start_editing_time(),
            "EditingSpeed" => self.input_flow.start_editing_speed(),
            "AddingEvent" => self.input_flow.start_adding_event(),
            _ => self.input_flow.cancel(),
        }
    }

    pub fn cancel_input(&mut self) {
        self.input_flow.cancel();
    }

    /// Returns the internal name of the current input flow step.
    pub fn get_input_step(&self) -> String {
        match &self.input_flow.flow {
            InputFlow::EditingTime => "EditingTime".to_string(),
            InputFlow::EditingSpeed => "EditingSpeed".to_string(),
            InputFlow::AddingEvent(flow) => format!("{:?}", flow.step),
            InputFlow::None => "None".to_string(),
        }
    }

    pub fn get_flow_name(&self) -> String {
        self.input_flow.get_flow_name()
    }

    /// Processes a single input string for the active flow and returns a status object.
    pub fn handle_input(&mut self, input: &str) -> Result<JsValue, JsError> {
        let result = self.input_flow.handle_input(input);
        
        #[derive(Serialize)]
        struct InputResult {
            success: bool,
            error: Option<String>,
            next_step: String,
            prompt: String,
        }

        let (success, error) = match result {
            InputFlowResult::Completed(event) => {
                self.core.handle_event(event);
                (true, None)
            }
            InputFlowResult::NextStep => (true, None),
            InputFlowResult::Error(err) => (false, Some(err)),
        };

        let result = InputResult {
            success,
            error,
            next_step: self.get_input_step(),
            prompt: self.input_flow.get_current_prompt(),
        };

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Validates a time string using the core's parsing logic.
    pub fn validate_time(&self, time_str: &str) -> Option<String> {
        parse_time(time_str).map(|t| t.format("%H:%M:%S").to_string())
    }

    /// Returns the deterministic hex color code for a given event ID.
    pub fn get_event_color_hex(&self, id: u32) -> String {
        vn_clock_core::models::ClockColor::to_hex_dynamic(id)
    }

    /// Converts a `ClockColor` variant (passed from JS) into a hex string.
    pub fn get_color_hex(&self, color: JsValue) -> String {
        match serde_wasm_bindgen::from_value::<vn_clock_core::models::ClockColor>(color) {
            Ok(color) => color.to_hex().to_string(),
            Err(e) => {
                log::error!("Failed to deserialize color: {}", e);
                "#ffffff".to_string()
            }
        }
    }

    /// Advances the core clock logic.
    pub fn tick(&mut self) {
        self.core.tick();
    }

    /// Dispatches a raw JSON event to the core logic.
    pub fn handle_event(&mut self, event_json: &str) -> Result<(), JsError> {
        match serde_json::from_str::<ClockEvent>(event_json) {
            Ok(event) => {
                self.core.handle_event(event);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to deserialize event: {}. JSON: {}", e, event_json);
                log::error!("{}", err_msg);
                Err(JsError::new(&err_msg))
            }
        }
    }

    pub fn take_output_events(&mut self) -> Result<JsValue, JsError> {
        let events = self.core.take_output_events();
        serde_wasm_bindgen::to_value(&events).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn get_clock_time(&self) -> String {
        self.core.clock_time().format("%H:%M:%S%.3f").to_string()
    }

    pub fn is_paused(&self) -> bool {
        self.core.paused()
    }

    pub fn get_logs(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(self.core.logs()).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns a list of all events, formatted for web display.
    pub fn get_events(&self) -> Result<JsValue, JsError> {
        #[derive(Serialize)]
        struct WebTimedEvent {
            id: u32,
            time: String,
            name: String,
            auto_pause: bool,
            repeat_interval: Option<[i64; 2]>,
            repeat_until: Option<String>,
            color: vn_clock_core::models::ClockColor,
            display_string: String,
        }

        let events: Vec<WebTimedEvent> = self.core.events().iter().map(|e| {
            WebTimedEvent {
                id: e.id,
                time: e.config.time.format("%H:%M:%S").to_string(),
                name: e.config.name.clone(),
                auto_pause: e.config.auto_pause,
                repeat_interval: e.config.repeat_interval.map(|d| [d.num_seconds(), 0]),
                repeat_until: e.config.repeat_until.map(|t| t.format("%H:%M:%S").to_string()),
                color: vn_clock_core::models::ClockColor::from_id(e.id),
                display_string: e.to_display_string(),
            }
        }).collect();

        serde_wasm_bindgen::to_value(&events).map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn get_target_speed_string(&self) -> String {
        self.core.get_target_speed_string()
    }

    pub fn get_initial_time_string(&self) -> String {
        self.core.get_initial_time_string()
    }

    pub fn get_config_json(&self) -> Result<String, JsError> {
        self.core.get_config_json().map_err(|e| JsError::new(&e))
    }

    /// Triggers a browser file picker to load a configuration.
    pub async fn load_config(&mut self) -> Result<(), JsError> {
        if let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter("Config", &[persistence::CONFIG_EXTENSION])
            .pick_file()
            .await
        {
            let bytes = file.read().await;
            let json = String::from_utf8(bytes).map_err(|e| JsError::new(&e.to_string()))?;
            self.load_config_json(&json)
        } else {
            Ok(())
        }
    }

    pub fn load_config_json(&mut self, json: &str) -> Result<(), JsError> {
        self.core.load_config_json(json).map_err(|e| JsError::new(&e))
    }

    pub fn get_state_json(&self) -> Result<String, JsError> {
        self.core.get_state_json().map_err(|e| JsError::new(&e))
    }

    /// Triggers a browser file picker to load a session state.
    pub async fn load_state(&mut self) -> Result<(), JsError> {
        if let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter("State", &[persistence::STATE_EXTENSION])
            .pick_file()
            .await
        {
            let bytes = file.read().await;
            let json = String::from_utf8(bytes).map_err(|e| JsError::new(&e.to_string()))?;
            self.load_state_json(&json)
        } else {
            Ok(())
        }
    }

    pub fn load_state_json(&mut self, json: &str) -> Result<(), JsError> {
        self.core.load_state_json(json).map_err(|e| JsError::new(&e))
    }
}
