use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Duplicate process ID: {0}")]
    DuplicateProcess(String),
    #[error("Invalid {port_type} port '{port}' for process '{process}'")]
    InvalidPort {
        process: String,
        port: String,
        port_type: String,
    },
    #[error("Invalid dt value: {0}")]
    InvalidDt(f64),
    #[error("No events remaining in queue")]
    NoEvents,
    #[error("{0}")]
    Other(String),
    #[error("Process with ID '{0}' not found")]
    ProcessNotFound(String),
}
