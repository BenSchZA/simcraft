use super::Processor;
use serde::de;
use serde::Deserializer;
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

pub type ProcessConstructor = fn(serde_yaml::Value) -> Option<Box<dyn Processor>>;
lazy_static! {
    static ref CONSTRUCTORS: Mutex<HashMap<&'static str, ProcessConstructor>> = {
        let mut m = HashMap::new();
        m.insert(
            "Delay",
            super::nodes::Delay::from_value as ProcessConstructor,
        );
        m.insert(
            "Drain",
            super::nodes::Drain::from_value as ProcessConstructor,
        );
        m.insert("Pool", super::nodes::Pool::from_value as ProcessConstructor);
        m.insert(
            "Source",
            super::nodes::Source::from_value as ProcessConstructor,
        );
        m.insert(
            "Stepper",
            super::nodes::Stepper::from_value as ProcessConstructor,
        );
        Mutex::new(m)
    };
    static ref VARIANTS: Vec<&'static str> = {
        CONSTRUCTORS
            .lock()
            .unwrap()
            .iter()
            .map(|(k, _)| k)
            .copied()
            .collect::<Vec<_>>()
    };
}

pub fn register_process(process_type: &'static str, process_constructor: ProcessConstructor) {
    CONSTRUCTORS
        .lock()
        .unwrap()
        .insert(process_type, process_constructor);
}

pub fn create_process<'de, D: Deserializer<'de>>(
    process_type: &str,
    extra_fields: serde_yaml::Value,
) -> Result<Box<dyn Processor>, D::Error> {
    let process = match CONSTRUCTORS.lock().unwrap().get(process_type) {
        Some(constructor) => constructor(extra_fields),
        None => None,
    };

    match process {
        Some(process) => Ok(process),
        None => Err(de::Error::unknown_variant(process_type, &VARIANTS)),
    }
}
