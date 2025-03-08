use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        process_state::{ProcessState, StepperState},
        process_trait::Processor,
    },
    simulator::{Event, EventPayload, SimulationContext},
    utils::SimulationError,
};

use super::TriggerMode;

#[derive(Builder, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Stepper {
    #[builder(setter(into))]
    id: String,
    #[builder(setter(skip))]
    state: StepperState,
    #[builder(setter(skip))]
    dt: f64,
    trigger_mode: TriggerMode,
}

impl Default for Stepper {
    fn default() -> Self {
        Self {
            id: String::new(),
            state: StepperState::default(),
            dt: 1.0,
            trigger_mode: TriggerMode::Automatic,
        }
    }
}

impl Stepper {
    pub fn builder() -> StepperBuilder {
        StepperBuilder::default()
    }

    pub fn set_dt(&mut self, dt: f64) -> Result<(), SimulationError> {
        // TODO Perform this validation in builder?
        if dt <= 0.0 {
            return Err(SimulationError::InvalidDt(dt));
        }
        self.dt = dt;
        Ok(())
    }
}

impl Processor for Stepper {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &mut SimulationContext,
    ) -> Result<Vec<Event>, SimulationError> {
        match event.payload {
            EventPayload::SimulationStart => {
                // Start stepping
                let new_events = vec![Event {
                    time: context.current_time(),
                    source_id: self.id.clone(),
                    source_port: Some("step".to_string()),
                    target_id: "broadcast".to_string(),
                    target_port: None,
                    payload: EventPayload::Step,
                }];
                Ok(new_events)
            }
            EventPayload::Step => {
                // Continue stepping
                let new_events = vec![Event {
                    time: context.current_time() + self.dt,
                    source_id: self.id.clone(),
                    source_port: Some("step".to_string()),
                    target_id: "broadcast".to_string(),
                    target_port: None,
                    payload: EventPayload::Step,
                }];
                Ok(new_events)
            }
            EventPayload::SimulationEnd => Ok(vec![]),
            _ => Ok(vec![]),
        }
    }

    fn get_state(&self) -> ProcessState {
        ProcessState::Stepper(self.state.clone())
    }

    fn get_input_ports(&self) -> Vec<String> {
        vec!["step".to_string()]
    }

    fn get_output_ports(&self) -> Vec<String> {
        vec!["step".to_string()]
    }
}
