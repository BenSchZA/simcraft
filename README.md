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

#### Drain

A drain process consumes resources, effectively removing them from the simulation.

```rust
drain "drain1" {
    // Attributes can be added here
}
```

#### Delay

A delay process holds resources for a period before releasing them. It can be used to model processing time or transport delays. The delay process supports two modes:

1. **Delay Mode (default)**: Each resource is delayed independently for the specified time period before being released.
2. **Queue Mode**: Resources are accumulated and released in fixed amounts after the delay period, like a batch processor.

```rust
// Delay mode (default) - each resource is delayed independently
delay "delay1" {
    action: DelayAction::Delay  // Optional: this is the default
}

// Queue mode - resources are released in batches
delay "delay2" {
    action: DelayAction::Queue,
    release_amount: 2.0  // Optional: amount to release per cycle (default: 1.0)
}
```

The delay time is determined by the `flow_rate` of the outgoing connection, where a flow rate of 1.0 equals one time unit of delay.

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

### Resource Processing Chain

This example demonstrates both delay modes in a processing chain:

```rust
let mut sim = simulation! {
    processes {
        source "input" {}
        delay "individual_processor" {
            action: DelayAction::Delay  // Process each resource independently
        }
        delay "batch_processor" {
            action: DelayAction::Queue,
            release_amount: 3.0  // Process resources in batches of 3
        }
        drain "output" {}
    }
    connections {
        "input.out" -> "individual_processor.in" {
            id: "input_flow",
            flow_rate: 1.0
        }
        "individual_processor.out" -> "batch_processor.in" {
            id: "middle_flow",
            flow_rate: 2.0  // 2 time units delay
        }
        "batch_processor.out" -> "output.in" {
            id: "output_flow",
            flow_rate: 1.0
        }
    }
}?;
```

In this example:
1. Resources flow from the source to an individual processor that delays each resource by 1 time unit
2. Then they pass through a batch processor that accumulates resources and releases them in groups of 3 after a 2 time unit delay
3. Finally, the processed resources are consumed by the drain
