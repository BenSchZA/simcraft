import { invoke } from '@tauri-apps/api/core';
import type { SimcraftAdapter, Process, Connection, SimulationState } from './base';

export class DesktopAdapter implements SimcraftAdapter {
	private simulationId: string | null = null;

	async initialise(processes: Process[], connections: Connection[]): Promise<void> {
		try {
			this.simulationId = await invoke<string>('create_simulation', { processes, connections });
		} catch (error) {
			throw new Error(`Failed to create Tauri simulation: ${error}`);
		}
	}

	async step(): Promise<SimulationState[]> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}

		try {
			return await invoke<SimulationState[]>('simulation_step', {
				simulationId: this.simulationId
			});
		} catch (error) {
			throw new Error(`Failed to step simulation: ${error}`);
		}
	}

	async step_until(until: number): Promise<SimulationState[]> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}

		try {
			return await invoke<SimulationState[]>('simulation_step_until', {
				simulationId: this.simulationId,
				until
			});
		} catch (error) {
			throw new Error(`Failed to step simulation until ${until}: ${error}`);
		}
	}

	async step_n(n: number): Promise<SimulationState[]> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}

		try {
			return await invoke<SimulationState[]>('simulation_step_n', {
				simulationId: this.simulationId,
				n
			});
		} catch (error) {
			throw new Error(`Failed to step simulation ${n} times: ${error}`);
		}
	}

	async destroy(): Promise<void> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}

		await invoke('destroy_simulation', { simulationId: this.simulationId });
	}
}
