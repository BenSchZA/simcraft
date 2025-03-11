use serde::{Deserialize, Serialize};

pub mod pool;
pub mod source;
pub mod stepper;
pub mod drain;

pub use self::pool::Pool;
pub use self::source::Source;
pub use self::stepper::Stepper;
pub use self::drain::Drain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    Passive,
    Interactive,
    Automatic,
    Enabling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    PullAny,
    PullAll,
    PushAny,
    PushAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Overflow {
    Block,
    Drain,
}
