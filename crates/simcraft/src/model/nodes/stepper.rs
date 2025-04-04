use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::TriggerMode;
use crate::{
    model::{
        process_state::{ProcessState, StepperState},
        ProcessContext, Processor, SerializableProcess,
    },
    simulator::{Event, EventPayload},
    utils::SimulationError,
};

#[derive(Builder, Debug, Clone, Serialize, Deserialize, SerializableProcess)]
#[serde(default, rename_all = "camelCase")]
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
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let new_events: Vec<Event> = match event.payload {
            EventPayload::SimulationStart | EventPayload::Step => vec![Event::new(
                &self.id,
                "broadcast",
                context.current_time() + self.dt,
                EventPayload::Step,
            )
            .with_ports("step", "")],
            EventPayload::SimulationEnd => vec![],
            _ => vec![],
        };

        Ok(new_events)
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

    fn reset(&mut self) {
        self.state = StepperState::default();
    }
}
