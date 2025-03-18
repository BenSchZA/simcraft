/// DSL macros for the simulation framework
///
/// This module provides macros for easily defining simulation models.
use crate::model::{nodes::stepper::Stepper, process::Process};

/// Creates a simulation model with the given processes and connections.
///
/// # Example
///
/// ```
/// use simcraft::dsl::*;
/// use crate::simcraft::simulator::Simulate;
///
/// let sim = simulation! {
///     processes {
///         source "source1" {}
///         pool "pool1" {}
///     }
///     connections {
///         "source1.out" -> "pool1.in" {
///             id: "conn1",
///             flow_rate: 1.0
///         }
///     }
/// };
/// ```
#[macro_export]
macro_rules! simulation {
    (
        processes {
            $($process_def:tt)*
        }
        connections {
            $($connection_def:tt)*
        }
    ) => {{
        let mut processes = vec![];

        // Add a stepper process by default
        processes.push($crate::dsl::create_stepper());

        // Add user-defined processes
        processes_internal!(processes, $($process_def)*);

        // Create connections
        let connections = connections_internal!($($connection_def)*);

        // Create the simulation
        $crate::simulator::Simulation::new(processes, connections)
    }};
}

/// Internal macro for defining processes
#[macro_export]
#[doc(hidden)]
macro_rules! processes_internal {
    // Base case: no more processes
    ($processes:ident, ) => {};

    // Source process with no attributes
    ($processes:ident, source $id:tt {} $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Source::builder();
            builder.id($id);
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // TODO Make trailing commas optional
    // Source process with attributes
    ($processes:ident, source $id:tt {
        $(
            trigger_mode: $trigger_mode:expr,
        )?
        $(
            action: $action:expr,
        )?
        $(,)?
    } $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Source::builder();
            let builder = builder.id($id);
            $(
                let builder = builder.trigger_mode($trigger_mode);
            )*
            $(
                let builder = builder.action($action);
            )*
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // Pool process with no attributes
    ($processes:ident, pool $id:tt {} $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Pool::builder();
            builder.id($id);
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // Pool process with attributes
    ($processes:ident, pool $id:tt {
        $(
            trigger_mode: $trigger_mode:expr,
        )?
        $(
            action: $action:expr,
        )?
        $(
            capacity: $capacity:expr,
        )?
        $(
            resources: $resources:expr,
        )?
        $(,)?
    } $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Pool::builder();
            let builder = builder.id($id);
            $(
                let builder = builder.trigger_mode($trigger_mode);
            )*
            $(
                let builder = builder.action($action);
            )*
            $(
                let builder = builder.capacity($capacity);
            )*
            $(
                let builder = builder.state($crate::model::process_state::PoolState{resources: $resources});
            )*
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // Drain process with no attributes
    ($processes:ident, drain $id:tt {} $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Drain::builder();
            builder.id($id);
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // Drain process with attributes
    ($processes:ident, drain $id:tt {
        $(
            trigger_mode: $trigger_mode:expr,
        )?
        $(
            action: $action:expr,
        )?
        $(,)?
    } $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Drain::builder();
            let builder = builder.id($id);
            $(
                let builder = builder.trigger_mode($trigger_mode);
            )*
            $(
                let builder = builder.action($action);
            )*
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // Stepper process with no attributes
    ($processes:ident, stepper $id:tt {} $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Stepper::builder();
            builder.id($id);
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };

    // Stepper process with attributes
    ($processes:ident, stepper $id:tt {
        $($attr_name:ident: $attr_value:expr),* $(,)?
    } $($rest:tt)*) => {
        {
            let mut builder = $crate::model::nodes::Stepper::builder();
            builder.id($id);
            $processes.push($crate::model::process::Process::new(Box::new(builder.build().unwrap())));
        }
        processes_internal!($processes, $($rest)*);
    };
}

/// Internal macro for defining connections
#[macro_export]
#[doc(hidden)]
macro_rules! connections_internal {
    // Base case: no more connections
    () => {
        vec![]
    };

    // Connection with all attributes
    ($source:tt -> $target:tt {
        id: $id:literal
        $(, flow_rate: $flow_rate:expr)?
        $(,)?
    } $($rest:tt)*) => {{
        let mut connections = connections_internal!($($rest)*);

        let (source_id, source_port) = $crate::dsl::parse_endpoint($source);
        let (target_id, target_port) = $crate::dsl::parse_endpoint($target);

        connections.push($crate::model::connection::Connection {
            id: $id.to_string(),
            source_id: source_id.to_string(),
            source_port: source_port.map(|s| s.to_string()),
            target_id: target_id.to_string(),
            target_port: target_port.map(|s| s.to_string()),
            flow_rate: None $(.or(Some($flow_rate)))?,
        });

        connections
    }};

    // Connection without id
    ($source:tt -> $target:tt {
        $(flow_rate: $flow_rate:expr)?
        $(,)?
    } $($rest:tt)*) => {{
        let mut connections = connections_internal!($($rest)*);

        let (source_id, source_port) = $crate::dsl::parse_endpoint($source);
        let (target_id, target_port) = $crate::dsl::parse_endpoint($target);

        connections.push($crate::model::connection::Connection {
            id: format!("conn_{}_{}", source_id, target_id),
            source_id: source_id.to_string(),
            source_port: source_port.map(|s| s.to_string()),
            target_id: target_id.to_string(),
            target_port: target_port.map(|s| s.to_string()),
            flow_rate: None $(.or(Some($flow_rate)))?,
        });

        connections
    }};
}

/// Helper function to create a default stepper process
pub fn create_stepper() -> Process {
    let mut builder = Stepper::builder();
    builder.id("stepper".to_string());
    Process::new(Box::new(builder.build().unwrap()))
}

/// Helper function to parse an endpoint string (e.g., "source1.out")
pub fn parse_endpoint(endpoint: &str) -> (&str, Option<&str>) {
    if let Some(idx) = endpoint.find('.') {
        let (id, port) = endpoint.split_at(idx);
        (id, Some(&port[1..]))
    } else {
        (endpoint, None)
    }
}

/// Macro for creating a simulation and running it for a specified number of steps
#[macro_export]
macro_rules! run_simulation {
    (
        steps: $steps:expr,
        $($sim_def:tt)*
    ) => {{
        let mut sim = simulation!($($sim_def)*)?;
        sim.step_n($steps)
    }};

    (
        until: $time:expr,
        $($sim_def:tt)*
    ) => {{
        let mut sim = simulation!($($sim_def)*)?;
        sim.step_until($time)
    }};
}

// Re-export macros for easier use
pub use crate::{connections_internal, processes_internal, run_simulation, simulation};
