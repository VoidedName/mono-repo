use wasm_bindgen::prelude::*;
use vn_clock_core::models::{CoreApp, ClockEvent, ClockConfig, ClockState};

#[wasm_bindgen]
pub struct WebApp {
    core: CoreApp,
}

#[wasm_bindgen]
impl WebApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
        Self {
            core: CoreApp::new(),
        }
    }

    pub fn tick(&mut self) {
        self.core.tick();
    }

    pub fn handle_event(&mut self, event_json: &str) {
        if let Ok(event) = serde_json::from_str::<ClockEvent>(event_json) {
            self.core.handle_event(event);
        }
    }

    pub fn take_output_events(&mut self) -> JsValue {
        let events = self.core.take_output_events();
        serde_wasm_bindgen::to_value(&events).unwrap()
    }

    pub fn get_clock_time(&self) -> String {
        self.core.clock_time().format("%H:%M:%S%.3f").to_string()
    }

    pub fn is_paused(&self) -> bool {
        self.core.paused()
    }

    pub fn get_logs(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.core.logs()).unwrap()
    }

    pub fn get_events(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.core.events()).unwrap()
    }

    pub fn get_target_speed(&self) -> f64 {
        self.core.target_speed()
    }

    pub fn get_initial_time(&self) -> String {
        self.core.initial_time().format("%H:%M:%S").to_string()
    }

    pub fn get_config_json(&self) -> Option<String> {
        let config = ClockConfig {
            initial_time: self.core.initial_time(),
            target_speed: self.core.target_speed(),
            events: self.core.events().to_vec(),
        };
        serde_json::to_string_pretty(&config).ok()
    }

    pub async fn load_config(&mut self) {
        let file = rfd::AsyncFileDialog::new()
            .add_filter("Clock Config", &["clockcfg"])
            .set_title("Load Configuration")
            .pick_file()
            .await;
        
        if let Some(file) = file {
            let bytes = file.read().await;
            if let Ok(config) = serde_json::from_slice::<ClockConfig>(&bytes) {
                self.core.handle_event(ClockEvent::LoadConfig(config));
            }
        }
    }

    pub fn get_state_json(&self) -> Option<String> {
        let state = ClockState {
            clock_time: self.core.clock_time(),
            initial_time: self.core.initial_time(),
            target_speed: self.core.target_speed(),
            paused: self.core.paused(),
            events: self.core.events().to_vec(),
            logs: self.core.logs().to_vec(),
        };
        serde_json::to_string_pretty(&state).ok()
    }

    pub async fn load_state(&mut self) {
        let file = rfd::AsyncFileDialog::new()
            .add_filter("Clock State", &["clockstate"])
            .set_title("Load State")
            .pick_file()
            .await;
        
        if let Some(file) = file {
            let bytes = file.read().await;
            if let Ok(state) = serde_json::from_slice::<ClockState>(&bytes) {
                self.core.handle_event(ClockEvent::LoadState(state));
            }
        }
    }
}
