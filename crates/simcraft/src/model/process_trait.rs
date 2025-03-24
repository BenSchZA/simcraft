use std::fmt::Debug;

use super::ProcessState;
use crate::simulator::event::Event;
use crate::simulator::SimulationContext;
use crate::utils::errors::SimulationError;

pub trait ProcessClone: Send + Debug {
    fn clone_box(&self) -> Box<dyn Processor + Send>;
}

impl<T> ProcessClone for T
where
    T: 'static + Processor + Clone + Send + Debug,
{
    fn clone_box(&self) -> Box<dyn Processor + Send> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Processor + Send> {
    fn clone(&self) -> Box<dyn Processor + Send> {
        self.clone_box()
    }
}

pub trait SerializableProcess {
    fn get_type(&self) -> &'static str {
        "Process"
    }

    fn serialize(&self) -> serde_yaml::Value {
        serde_yaml::Value::Null
    }
}

pub trait Processor: ProcessClone + SerializableProcess {
    fn id(&self) -> &str;
    fn on_event(
        &mut self,
        event: &Event,
        context: &SimulationContext,
    ) -> Result<Vec<Event>, SimulationError>;
    fn get_state(&self) -> ProcessState;
    fn get_input_ports(&self) -> Vec<String>;
    fn get_output_ports(&self) -> Vec<String>;
}
