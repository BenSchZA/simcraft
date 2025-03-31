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
	private stateUpdateCallback: StateUpdateCallback | null = null;
	private _isRunning = false;
	private runInterval: number | null = null;

	async initialise(processes: Process[], connections: Connection[]): Promise<SimulationState> {
		try {
			this.simulation = WasmSimulation.new(JSON.stringify(processes), JSON.stringify(connections));
			return await this.getState();
		} catch (error) {
			throw new Error(`Failed to create WASM simulation: ${error}`);
		}
	}

	isInitialized(): boolean {
		return this.simulation !== null;
	}

	isRunning(): boolean {
		return this._isRunning;
	}

	async step(): Promise<SimulationResult> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}

		try {
			const events: Event[] = this.simulation.step();
			const state: SimulationState = await this.getState();
			this.stateUpdateCallback?.([state]);
			return { events, state };
		} catch (error) {
			throw new Error(`Failed to step simulation: ${error}`);
		}
	}

	async play(delayMs: number): Promise<boolean> {
		if (!this.simulation) {
			return false;
		}

		this._isRunning = true;
		this.runInterval = window.setInterval(async () => {
			if (!this._isRunning) {
				this.stopInterval();
				return;
			}

			try {
				await this.step();
			} catch (error) {
				console.error('Error in continuous simulation:', error);
				this.stopInterval();
			}
		}, delayMs);

		return true;
	}

	async pause(): Promise<boolean> {
		this._isRunning = false;
		this.stopInterval();
		return true;
	}

	onStateUpdate(callback: StateUpdateCallback): void {
		this.stateUpdateCallback = callback;
	}

	private stopInterval() {
		if (this.runInterval !== null) {
			window.clearInterval(this.runInterval);
			this.runInterval = null;
		}
	}

	async reset(): Promise<void> {
		if (!this.simulation) {
			throw new Error('Simulation not initialised');
		}
		await this.simulation.reset();
	}

	async destroy(): Promise<void> {
		await this.pause();
		this.stateUpdateCallback = null;
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
