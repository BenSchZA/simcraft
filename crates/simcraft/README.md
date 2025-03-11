# Simcraft

An event-driven simulation framework for modelling and simulating resource flows.

## Domain-Specific Language (DSL)

Simcraft provides a domain-specific language (DSL) for easily defining simulation models. The DSL allows you to define processes and connections between them in a declarative way.

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