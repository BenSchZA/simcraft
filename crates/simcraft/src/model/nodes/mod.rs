use serde::{Deserialize, Serialize};

pub mod pool;
pub mod source;
pub mod stepper;

pub use self::pool::Pool;
pub use self::source::Source;
pub use self::stepper::Stepper;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TriggerMode {
    Passive,
    Interactive,
    Automatic,
    Enabling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Action {
    PullAny,
    PullAll,
    PushAny,
    PushAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Overflow {
    Block,
    Drain,
}
