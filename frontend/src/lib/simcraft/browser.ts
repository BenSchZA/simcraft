import { Simulation as WasmSimulation } from 'simcraft_web';
import type {
	SimcraftAdapter,
	Process,
	Connection,
	SimulationState,
	Event,
	SimulationResult,
	ProcessState
} from './base';

type StateUpdateCallback = (state: SimulationState[]) => void;

export class BrowserAdapter implements SimcraftAdapter {
	private simulation: WasmSimulation | null = null;
	private stateUpdateCallbacks: StateUpdateCallback[] = [];
	private isRunning = false;
	private runInterval: number | null = null;

	async initialise(processes: Process[], connections: Connection[]): Promise<SimulationState> {
		try {
			this.simulation = WasmSimulation.new(JSON.stringify(processes), JSON.stringify(connections));
			const state: SimulationState = this.simulation.get_simulation_state();
			return state;
		} catch (error) {
			throw new Error(`Failed to create WASM simulation: ${error}`);
		}
	}

	async step(): Promise<SimulationResult> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}

		try {
			const events: Event[] = this.simulation.step();
			const state: SimulationState = await this.getState();
			return { events, state };
		} catch (error) {
			throw new Error(`Failed to step simulation: ${error}`);
		}
	}

	async play(delayMs: number): Promise<boolean> {
		if (!this.simulation) {
			return false;
		}

		this.isRunning = true;
		this.runInterval = window.setInterval(async () => {
			if (!this.isRunning) {
				this.stopInterval();
				return;
			}

			try {
				const result = await this.step();
				const state = result.state;
				this.stateUpdateCallbacks.forEach((callback) => callback([state]));
			} catch (error) {
				console.error('Error in continuous simulation:', error);
				this.stopInterval();
			}
		}, delayMs);

		return true;
	}

	async pause(): Promise<boolean> {
		this.isRunning = false;
		this.stopInterval();
		return true;
	}

	onStateUpdate(callback: StateUpdateCallback): () => void {
		this.stateUpdateCallbacks.push(callback);
		return () => {
			this.stateUpdateCallbacks = this.stateUpdateCallbacks.filter((cb) => cb !== callback);
		};
	}

	private stopInterval() {
		if (this.runInterval !== null) {
			window.clearInterval(this.runInterval);
			this.runInterval = null;
		}
	}

	async destroy(): Promise<void> {
		await this.pause();
		this.stateUpdateCallbacks = [];
		this.simulation = null;
	}

	async getState(): Promise<SimulationState> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}

		const raw_state = this.simulation.get_simulation_state();
		const state: SimulationState = {
			...raw_state,
			process_states: Object.fromEntries(raw_state.process_states) as Record<string, ProcessState>
		};

		return state;
	}
}
