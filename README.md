# Simcraft
[![Build Status](https://github.com/BenSchZA/simcraft/actions/workflows/test-simcraft.yml/badge.svg)](https://github.com/BenSchZA/simcraft/actions/workflows/test-simcraft.yml)

A Discrete-Event Simulation (DES) framework for generalised simulation modelling.

<img width="1443" alt="image" src="https://github.com/user-attachments/assets/646347b9-7d69-4adf-8eec-444ffc320aff" />

## Event-Driven Simulation Architecture

An event-driven simulation architecture provides a flexible framework for implementing various simulation modelling paradigms:

- **System Dynamics:** Events represent the flow of resources or information between interconnected nodes, evolving over time through differential or difference equations. A periodic time-stepping event approximates continuous changes at fixed intervals.
- **Agent-Based Modelling (ABM):** Events represent agent decisions, message exchanges, and state transitions, allowing agents to interact asynchronously with each other and their environment.
- **Discrete-Event Simulation (DES):** Events represent state changes occurring at specific points in time, dynamically scheduling the next event without requiring fixed time steps.
- **Discrete-Time Simulation (DTS):** Events represent updates to the system state at uniform, fixed time steps, ensuring that all changes occur at regular intervals, regardless of necessity.

The framework is inspired by the [DEVS](https://www.cs.mcgill.ca/~hv/classes/MS/DEVS.pdf) (Discrete EVent System Specification) formalism and the [SimRS](https://simrs.com/) DEVS implementation.

## Resource Flow Model Domain-Specific Language (DSL)

As a first application of the framework, Simcraft provides a domain-specific language (DSL) for easily defining resource flow models as defined in the ["Engineering Emergence: Applied Theory for Game Design"](https://eprints.illc.uva.nl/id/eprint/2118/1/DS-2012-12.text.pdf) paper by Joris Dormans.

The DSL allows you to define processes (e.g., Source, Pool, Drain nodes) and connections (i.e. flows) between them in a declarative way.

### Basic Usage

```rust
use simcraft::dsl::*;
use simcraft::simulator::Simulate;
use simcraft::utils::errors::SimulationError;

fn main() -> Result<(), SimulationError> {
    // Create a simulation using the DSL
    let mut sim = simulation! {
        processes {
            source "source1" {}
            pool "pool1" {}
        }
        connections {
            "source1.out" -> "pool1.in" {
                id: "conn1",
                flow_rate: 1.0
            }
        }
    }?;

    // Run the simulation for 5 steps
    let results = sim.step_n(5)?;

    // Process the results
    println!("Final time: {}", results.last().unwrap().time);

    Ok(())
}
```

Or equivalent using the framework directly:

```rust
use simcraft::prelude::*;
use simcraft::model::nodes::{Source, Pool};
use simcraft::simulator::simulation_trait::StatefulSimulation;

fn main() -> Result<(), SimulationError> {
    // Create processes
    let source = Source::builder()
        .id("source1")
        .build()
        .unwrap();
    let pool = Pool::builder()
        .id("pool1")
        .build()
        .unwrap();

    // Create connection
    let connection = Connection::new(
        "conn1".to_string(),
        "source1".to_string(),
        Some("out".to_string()),
        "pool1".to_string(),
        Some("in".to_string()),
        Some(1.0),
    );

    // Create simulation and add processes
    let mut sim = Simulation::new(vec![], vec![])?;
    sim.add_process(source)?;
    sim.add_process(pool)?;
    sim.add_connection(connection)?;

    // Run the simulation for 5 steps
    let _ = sim.step_n(5)?;

    // Get final state
    let final_state = sim.get_simulation_state();
    println!("Final time: {}", final_state.time);

    Ok(())
}
```

Or equivalent using YAML format:

```yaml
name: "Basic Source to Pool"
description: "Simple example showing a source flowing to a pool"
processes:
  - id: "source1"
    type: "Source"
    triggerMode: "Automatic"
    action: "PushAny"
  - id: "pool1"
    type: "Pool"
    triggerMode: "Automatic"
    action: "PullAny"
connections:
  - id: "conn1"
    sourceID: "source1"
    targetID: "pool1"
    flowRate: 1.0
```

### Process Types

The DSL supports the following process types:

#### Source

A source process generates resources.

```rust
source "source1" {
    // Attributes can be added here
}
```

#### Pool

A pool process stores resources.

```rust
pool "pool1" {
    capacity: 10.0  // Optional capacity limit
}
```

### Connections

Connections define how resources flow between processes.

```rust
"source1.out" -> "pool1.in" {
    id: "conn1",
    flow_rate: 1.0  // Optional flow rate
}
```

The connection syntax uses the format `"process_id.port"` for both source and target endpoints. If the port is omitted, the default port for the process type is used.

### Running Simulations

You can use the `run_simulation!` macro to create and run a simulation in one step:

```rust
let results = run_simulation! {
    steps: 5,  // Run for 5 steps
    processes {
        source "source1" {}
        pool "pool1" {}
    }
    connections {
        "source1.out" -> "pool1.in" {
            id: "conn1",
            flow_rate: 1.0
        }
    }
}?;
```

Or run until a specific time:

```rust
let results = run_simulation! {
    until: 10.0,  // Run until time = 10.0
    processes {
        source "source1" {}
        pool "pool1" {}
    }
    connections {
        "source1.out" -> "pool1.in" {
            id: "conn1",
            flow_rate: 1.0
        }
    }
}?;
```

## Examples

### Multiple Sources to Pool

```rust
let mut sim = simulation! {
    processes {
        source "source1" {}
        source "source2" {}
        pool "pool1" {}
    }
    connections {
        "source1.out" -> "pool1.in" {
            id: "conn1",
            flow_rate: 1.0
        }
        "source2.out" -> "pool1.in" {
            id: "conn2",
            flow_rate: 2.0
        }
    }
}?;
```

### Pool with Capacity

```rust
let mut sim = simulation! {
    processes {
        source "source1" {}
        pool "pool1" {
            capacity: 3.0  // Pool will not accept more than 3.0 resources
        }
    }
    connections {
        "source1.out" -> "pool1.in" {
            id: "conn1",
            flow_rate: 1.0
        }
    }
}?;
```
