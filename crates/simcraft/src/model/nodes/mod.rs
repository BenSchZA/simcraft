use serde::{Deserialize, Serialize};

pub mod delay;
pub mod drain;
pub mod pool;
pub mod resource;
pub mod source;
pub mod stepper;

pub use self::delay::Delay;
pub use self::drain::Drain;
pub use self::pool::Pool;
pub use self::source::Source;
pub use self::stepper::Stepper;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    Passive,
    Interactive,
    Automatic,
    Enabling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelayAction {
    Delay,
    Queue,
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
