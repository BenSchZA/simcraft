pub mod connection;
pub mod nodes;
pub mod process;
pub mod process_factory;
pub mod process_repr;
pub mod process_state;
pub mod process_trait;

pub use connection::Connection;
pub use process::Process;
pub use process_repr::ProcessRepr;
pub use process_state::ProcessState;
pub use process_trait::Processor;
pub use process_trait::SerializableProcess;
pub use simcraft_derive::SerializableProcess;
