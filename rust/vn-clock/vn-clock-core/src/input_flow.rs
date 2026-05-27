//! Multi-step input flow state machine for the VN Clock.

use crate::models::{ClockEvent, TimedEventConfig};
use crate::utils::parse_time;
use chrono::{Duration, Timelike, NaiveTime};
use serde::{Serialize, Deserialize};

/// Represents the current step in the "Adding Event" process.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AddingEventStep {
    Name,
    Time,
    AutoPause,
    RepeatInterval,
    RepeatUntil,
}

/// State for the "Adding Event" multi-step flow.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AddingEventFlow {
    pub step: AddingEventStep,
    pub name: String,
    pub time: Option<NaiveTime>,
    pub auto_pause: bool,
    pub repeat_interval: Option<Duration>,
}

impl AddingEventFlow {
    /// Converts the collected flow data into a `TimedEventConfig`.
    pub fn finalize(&self, repeat_until: Option<NaiveTime>) -> TimedEventConfig {
        TimedEventConfig {
            time: self.time.unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            name: self.name.clone(),
            auto_pause: self.auto_pause,
            repeat_interval: self.repeat_interval,
            repeat_until,
        }
    }
}

/// The result of processing a single step in an input flow.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InputFlowResult {
    /// The current flow has been completed successfully and produced an event.
    Completed(ClockEvent),
    /// The input was accepted, and the flow has transitioned to the next step.
    NextStep,
    /// The input was rejected due to a validation error.
    Error(String),
}

/// Represents the active input process.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InputFlow {
    None,
    EditingTime,
    EditingSpeed,
    AddingEvent(AddingEventFlow),
}

/// Orchestrates multi-step user inputs and their transitions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InputFlowState {
    pub flow: InputFlow,
    /// Current input text buffer.
    pub buffer: String,
}

impl InputFlowState {
    /// Creates a new `InputFlowState` in the `None` state.
    pub fn new() -> Self {
        Self {
            flow: InputFlow::None,
            buffer: String::new(),
        }
    }

    /// Returns the prompt text for the current active flow step.
    pub fn get_current_prompt(&self) -> String {
        match &self.flow {
            InputFlow::EditingTime => "SET TIME (HH:MM:SS)".to_string(),
            InputFlow::EditingSpeed => "SET SPEED (multiplier)".to_string(),
            InputFlow::AddingEvent(flow) => match flow.step {
                AddingEventStep::Name => "ADD EVENT: NAME".to_string(),
                AddingEventStep::Time => "ADD EVENT: TIME (HH:MM:SS)".to_string(),
                AddingEventStep::AutoPause => "ADD EVENT: AUTO-PAUSE? (y/n)".to_string(),
                AddingEventStep::RepeatInterval => {
                    "ADD EVENT: REPEAT INTERVAL (HH:MM:SS, empty to skip)".to_string()
                }
                AddingEventStep::RepeatUntil => "ADD EVENT: REPEAT UNTIL (HH:MM:SS)".to_string(),
            },
            InputFlow::None => "INPUT".to_string(),
        }
    }

    /// Returns a human-readable name for the active flow.
    pub fn get_flow_name(&self) -> String {
        match &self.flow {
            InputFlow::EditingTime => "Editing Time".to_string(),
            InputFlow::EditingSpeed => "Editing Speed".to_string(),
            InputFlow::AddingEvent(_) => "Adding Event".to_string(),
            InputFlow::None => "Normal".to_string(),
        }
    }

    /// Transitions to the "Editing Time" flow.
    pub fn start_editing_time(&mut self) {
        self.flow = InputFlow::EditingTime;
        self.buffer.clear();
    }

    /// Transitions to the "Editing Speed" flow.
    pub fn start_editing_speed(&mut self) {
        self.flow = InputFlow::EditingSpeed;
        self.buffer.clear();
    }

    /// Transitions to the "Adding Event" flow.
    pub fn start_adding_event(&mut self) {
        self.flow = InputFlow::AddingEvent(AddingEventFlow {
            step: AddingEventStep::Name,
            name: String::new(),
            time: None,
            auto_pause: false,
            repeat_interval: None,
        });
        self.buffer.clear();
    }

    /// Cancels any active flow and clears the buffer.
    pub fn cancel(&mut self) {
        self.flow = InputFlow::None;
        self.buffer.clear();
    }

    /// Validates the provided input and advances the flow state.
    pub fn handle_input(&mut self, input: &str) -> InputFlowResult {
        let val = input.trim();
        match &mut self.flow {
            InputFlow::EditingTime => {
                if let Some(time) = parse_time(val) {
                    self.flow = InputFlow::None;
                    InputFlowResult::Completed(ClockEvent::SetTime(time))
                } else {
                    InputFlowResult::Error("Invalid time format!".to_string())
                }
            }
            InputFlow::EditingSpeed => {
                if let Ok(speed) = val.parse::<f64>() {
                    if speed >= 0.0 {
                        self.flow = InputFlow::None;
                        InputFlowResult::Completed(ClockEvent::SetSpeed(speed))
                    } else {
                        InputFlowResult::Error("Speed cannot be negative".to_string())
                    }
                } else {
                    InputFlowResult::Error("Invalid speed!".to_string())
                }
            }
            InputFlow::AddingEvent(flow) => match flow.step {
                AddingEventStep::Name => {
                    if val.is_empty() {
                        InputFlowResult::Error("Name cannot be empty".to_string())
                    } else {
                        flow.name = val.to_string();
                        flow.step = AddingEventStep::Time;
                        InputFlowResult::NextStep
                    }
                }
                AddingEventStep::Time => {
                    if let Some(time) = parse_time(val) {
                        flow.time = Some(time);
                        flow.step = AddingEventStep::AutoPause;
                        InputFlowResult::NextStep
                    } else {
                        InputFlowResult::Error("Invalid time format!".to_string())
                    }
                }
                AddingEventStep::AutoPause => {
                    let lower = val.to_lowercase();
                    if lower == "y" || lower == "n" {
                        flow.auto_pause = lower == "y";
                        flow.step = AddingEventStep::RepeatInterval;
                        InputFlowResult::NextStep
                    } else {
                        InputFlowResult::Error("Please enter y or n".to_string())
                    }
                }
                AddingEventStep::RepeatInterval => {
                    if val.is_empty() {
                        let event = flow.finalize(None);
                        self.flow = InputFlow::None;
                        InputFlowResult::Completed(ClockEvent::AddTimedEvent(event))
                    } else if let Some(t) = parse_time(val) {
                        let seconds = t.num_seconds_from_midnight();
                        if seconds > 0 {
                            flow.repeat_interval = Some(Duration::seconds(seconds as i64));
                            flow.step = AddingEventStep::RepeatUntil;
                            InputFlowResult::NextStep
                        } else {
                            InputFlowResult::Error("Repeat interval must be > 0".to_string())
                        }
                    } else {
                        InputFlowResult::Error("Invalid interval format!".to_string())
                    }
                }
                AddingEventStep::RepeatUntil => {
                    let mut repeat_until = None;
                    if !val.is_empty() {
                        if let Some(until) = parse_time(val) {
                            repeat_until = Some(until);
                        } else {
                            return InputFlowResult::Error("Invalid until format!".to_string());
                        }
                    }
                    let event = flow.finalize(repeat_until);
                    self.flow = InputFlow::None;
                    InputFlowResult::Completed(ClockEvent::AddTimedEvent(event))
                }
            },
            InputFlow::None => InputFlowResult::Error("No active input flow".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editing_time_flow() {
        let mut state = InputFlowState::new();
        state.start_editing_time();
        
        let result = state.handle_input("12:30:00");
        if let InputFlowResult::Completed(ClockEvent::SetTime(t)) = result {
            assert_eq!(t, NaiveTime::from_hms_opt(12, 30, 0).unwrap());
        } else {
            panic!("Expected Completed(SetTime) result, got {:?}", result);
        }
        assert_eq!(state.flow, InputFlow::None);
    }

    #[test]
    fn test_editing_speed_flow() {
        let mut state = InputFlowState::new();
        state.start_editing_speed();
        
        let result = state.handle_input("2.5");
        if let InputFlowResult::Completed(ClockEvent::SetSpeed(s)) = result {
            assert_eq!(s, 2.5);
        } else {
            panic!("Expected Completed(SetSpeed) result, got {:?}", result);
        }
        
        state.start_editing_speed();
        let result = state.handle_input("-1");
        if let InputFlowResult::Error(err) = result {
            assert_eq!(err, "Speed cannot be negative");
        } else {
            panic!("Expected Error result, got {:?}", result);
        }
    }

    #[test]
    fn test_adding_event_flow() {
        let mut state = InputFlowState::new();
        state.start_adding_event();
        
        // Name
        let result = state.handle_input("Test Event");
        assert!(matches!(result, InputFlowResult::NextStep));
        
        // Time
        let result = state.handle_input("10:00:00");
        assert!(matches!(result, InputFlowResult::NextStep));
        
        // Auto-pause
        let result = state.handle_input("y");
        assert!(matches!(result, InputFlowResult::NextStep));
        
        // Repeat Interval
        let result = state.handle_input("00:05:00");
        assert!(matches!(result, InputFlowResult::NextStep));
        
        // Repeat Until
        let result = state.handle_input("11:00:00");
        if let InputFlowResult::Completed(ClockEvent::AddTimedEvent(config)) = result {
            assert_eq!(config.name, "Test Event");
            assert_eq!(config.time, NaiveTime::from_hms_opt(10, 0, 0).unwrap());
            assert!(config.auto_pause);
            assert_eq!(config.repeat_interval.unwrap(), Duration::minutes(5));
            assert_eq!(config.repeat_until.unwrap(), NaiveTime::from_hms_opt(11, 0, 0).unwrap());
        } else {
            panic!("Expected Completed(AddTimedEvent) result, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_time_consistency() {
        assert_eq!(parse_time("24"), Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
        assert_eq!(parse_time("24:00"), Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
        assert_eq!(parse_time("24:00:00"), Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
        
        assert_eq!(parse_time("22"), Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()));
        assert_eq!(parse_time("2"), Some(NaiveTime::from_hms_opt(2, 0, 0).unwrap()));
        assert_eq!(parse_time("02"), Some(NaiveTime::from_hms_opt(2, 0, 0).unwrap()));
        
        assert_eq!(parse_time("25"), None);
    }
}
