use criterion::*;
use std::hint::black_box;
use std::time::Duration;

use simcraft::dsl::*;
use simcraft::model::nodes::{Action, TriggerMode};
use simcraft::simulator::Simulate;
use simcraft::utils::errors::SimulationError;

fn simulation_benchmark(steps: u64) -> Result<(), SimulationError> {
    let mut sim = simulation! {
        processes {
            source "source1" {
                trigger_mode: TriggerMode::Automatic,
                action: Action::PushAny,
            }
            source "source2" {
                trigger_mode: TriggerMode::Automatic,
                action: Action::PushAny,
            }
            pool "pool1" {
                trigger_mode: TriggerMode::Automatic,
                action: Action::PushAny,
                capacity: 10.0,
            }
            pool "pool2" {
                trigger_mode: TriggerMode::Automatic,
                action: Action::PushAny,
                capacity: 15.0,
            }
            pool "pool3" {
                trigger_mode: TriggerMode::Automatic,
                action: Action::PushAny,
            }
        }
        connections {
            "source1.out" -> "pool1.in" {
                id: "conn1",
                flow_rate: 2.0
            }
            "source2.out" -> "pool1.in" {
                id: "conn2",
                flow_rate: 1.5
            }
            "pool1.out" -> "pool2.in" {
                id: "conn3",
                flow_rate: 1.0
            }
            "pool1.out" -> "pool3.in" {
                id: "conn4",
                flow_rate: 0.5
            }
            "pool2.out" -> "pool3.in" {
                id: "conn5",
                flow_rate: 1.0
            }
        }
    }?;

    sim.step_n(steps as usize)?;
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("default");
    group.sample_size(10);

    for steps in [1_000, 5_000, 10_000, 50_000, 100_000, 500_000].iter() {
        group.throughput(Throughput::Elements(*steps as u64));
        group.bench_with_input(format!("Simulate {} steps", steps), steps, |b, s| {
            b.iter(|| simulation_benchmark(black_box(*s)));
        });
    }

    group.finish();
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(10));
  targets = criterion_benchmark
}
criterion_main!(benches);
