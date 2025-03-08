use simcraft::{
    model::{connection::Connection, nodes::Stepper, Process},
    simulator::{Simulate, Simulation},
    utils::SimulationError,
};

pub fn setup() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn create_stepper() -> Process {
    Process::new(Box::new(
        Stepper::builder()
            .id("stepper".to_string())
            .build()
            .unwrap(),
    ))
}

pub fn create_stepped_simulation(
    mut processes: Vec<Process>,
    connections: Vec<Connection>,
) -> Result<Simulation, SimulationError> {
    processes.insert(0, create_stepper());
    Simulation::new(processes, connections)
}
