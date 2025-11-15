use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;
use uuid::Uuid;

use simcraft::{
    model::{connection::Connection, process::Process},
    simulator::{Simulate, Simulation, Event, SimulationState, StatefulSimulation},
};

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
) -> Result<Vec<Event>, String> {
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
) -> Result<Vec<Event>, String> {
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

#[tauri::command]
async fn simulation_state(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<SimulationState, String> {
    let simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    Ok(simulation.get_simulation_state())
}

#[tauri::command]
async fn step_until(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    until: f64,
) -> Result<Vec<Event>, String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .step_until(until)
        .map_err(|e| format!("Failed to step simulation until {}: {}", until, e))
}

#[tauri::command]
async fn reset_simulation(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .reset()
        .map_err(|e| format!("Failed to reset simulation: {}", e))
}

#[tauri::command]
async fn get_state(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<SimulationState, String> {
    let simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    Ok(simulation.get_simulation_state())
}

#[tauri::command]
async fn add_process(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    process: Process,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .add_process(process)
        .map_err(|e| format!("Failed to add process: {}", e))
}

#[tauri::command]
async fn remove_process(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    process_id: String,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .remove_process(&process_id)
        .map_err(|e| format!("Failed to remove process: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn update_process(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    process_id: String,
    process: Process,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .update_process(&process_id, process)
        .map_err(|e| format!("Failed to update process: {}", e))
}

#[tauri::command]
async fn get_processes(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<Vec<Process>, String> {
    let simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    Ok(simulation.processes().values().cloned().collect())
}

#[tauri::command]
async fn add_connection(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    connection: Connection,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .add_connection(connection)
        .map_err(|e| format!("Failed to add connection: {}", e))
}

#[tauri::command]
async fn remove_connection(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    connection_id: String,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .remove_connection(&connection_id)
        .map_err(|e| format!("Failed to remove connection: {}", e))
}

#[tauri::command]
async fn update_connection(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
    connection_id: String,
    connection: Connection,
) -> Result<(), String> {
    let mut simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get_mut(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    simulation
        .update_connection(&connection_id, connection)
        .map_err(|e| format!("Failed to update connection: {}", e))
}

#[tauri::command]
async fn get_current_step(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<u64, String> {
    let simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    Ok(simulation.current_step())
}

#[tauri::command]
async fn get_current_time(
    manager: State<'_, Arc<SimulationManager>>,
    simulation_id: String,
) -> Result<f64, String> {
    let simulations = manager.simulations.lock().unwrap();

    let simulation = simulations
        .get(&simulation_id)
        .ok_or_else(|| "Simulation not found".to_string())?;

    Ok(simulation.current_time())
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
            destroy_simulation,
            simulation_state,
            step_until,
            reset_simulation,
            get_state,
            add_process,
            remove_process,
            update_process,
            get_processes,
            add_connection,
            remove_connection,
            update_connection,
            get_current_step,
            get_current_time
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
