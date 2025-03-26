use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;
use uuid::Uuid;

use simcraft::{
    model::{connection::Connection, process::Process},
    simulator::{Simulate, Simulation, SimulationState},
};

#[derive(Debug, Serialize, Deserialize)]
struct SimulationRequest {
    processes: Vec<Process>,
    connections: Vec<Connection>,
}

struct SimulationManager {
    simulations: Mutex<HashMap<String, Simulation>>,
}

impl Default for SimulationManager {
    fn default() -> Self {
        Self {
            simulations: Mutex::new(HashMap::new()),
        }
    }
}

#[tauri::command]
async fn create_simulation(
    manager: State<'_, Arc<SimulationManager>>,
    processes: Vec<Process>,
    connections: Vec<Connection>,
) -> Result<String, String> {
    let simulation = Simulation::new(processes, connections)
        .map_err(|e| format!("Failed to create simulation: {}", e))?;

    let id = Uuid::new_v4().to_string();

    manager
        .simulations
        .lock()
        .unwrap()
        .insert(id.clone(), simulation);

    Ok(id)
}

#[tauri::command]
async fn simulation_step(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<Vec<SimulationState>, String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .step()
        .map_err(|e| format!("Failed to step simulation: {}", e))
}

#[tauri::command]
async fn simulation_step_n(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    n: usize,
) -> Result<Vec<SimulationState>, String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .step_n(n)
        .map_err(|e| format!("Failed to step simulation {} times: {}", n, e))
}

#[tauri::command]
async fn destroy_simulation(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    simulations
        .remove(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let simulation_manager = Arc::new(SimulationManager::default());

    tauri::Builder::default()
        .manage(simulation_manager)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_simulation,
            simulation_step,
            simulation_step_n,
            destroy_simulation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
