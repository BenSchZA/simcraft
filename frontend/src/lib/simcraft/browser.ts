import { Simulation as WasmSimulation } from 'simcraft_web';
import type { SimcraftAdapter, Process, Connection, SimulationState } from './base';

export class BrowserAdapter implements SimcraftAdapter {
	private simulation: WasmSimulation | null = null;

	async initialise(processes: Process[], connections: Connection[]): Promise<void> {
		try {
			this.simulation = WasmSimulation.new(JSON.stringify(processes), JSON.stringify(connections));
		} catch (error) {
			throw new Error(`Failed to create WASM simulation: ${error}`);
		}
	}

	async step(): Promise<SimulationState[]> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}

		try {
			const results = this.simulation.step();
			return Array.from(results as Array<SimulationState>);
		} catch (error) {
			throw new Error(`Failed to step simulation: ${error}`);
		}
	}

	async step_until(until: number): Promise<SimulationState[]> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}

		try {
			const results = this.simulation.step_until(until);
			return Array.from(results as Array<SimulationState>);
		} catch (error) {
			throw new Error(`Failed to step simulation until ${until}: ${error}`);
		}
	}

	async step_n(n: number): Promise<SimulationState[]> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}

		try {
			const results = this.simulation.step_n(n);
			return Array.from(results as Array<SimulationState>);
		} catch (error) {
			throw new Error(`Failed to step simulation ${n} times: ${error}`);
		}
	}

	async destroy(): Promise<void> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}
	}
}
