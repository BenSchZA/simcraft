import { invoke } from '@tauri-apps/api/core';
import type {
	SimcraftAdapter,
	Process,
	Connection,
	SimulationState,
	SimulationResult,
	Event
} from './base';

type StateUpdateCallback = (state: SimulationState[]) => void;

export class DesktopAdapter implements SimcraftAdapter {
	private simulationId: string | null = null;
	private stateUpdateCallbacks: StateUpdateCallback[] = [];
	private _isRunning = false;
	private runInterval: number | null = null;

	async initialise(processes: Process[], connections: Connection[]): Promise<SimulationState> {
		try {
			this.simulationId = await invoke<string>('create_simulation', { processes, connections });
			const state = await invoke<SimulationState>('simulation_state', {
				simulationId: this.simulationId
			});
			return state;
		} catch (error) {
			throw new Error(`Failed to create Tauri simulation: ${error}`);
		}
	}

	isInitialized(): boolean {
		return this.simulationId !== null;
	}

	isRunning(): boolean {
		return this._isRunning;
	}

	async step(): Promise<SimulationResult> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}

		try {
			const events = await invoke<Event[]>('simulation_step', {
				simulationId: this.simulationId
			});
			const state = await invoke<SimulationState>('simulation_state', {
				simulationId: this.simulationId
			});
			return { events, state };
		} catch (error) {
			throw new Error(`Failed to step simulation: ${error}`);
		}
	}

	async play(delayMs: number): Promise<boolean> {
		if (!this.simulationId) {
			return false;
		}

		this._isRunning = true;
		this.runInterval = window.setInterval(async () => {
			if (!this._isRunning) {
				this.stopInterval();
				return;
			}

			try {
				const result = await this.step();
				const state = result.state;
				if (state) {
					this.stateUpdateCallbacks.forEach((callback) => callback([state]));
				}
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

	async reset(): Promise<void> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}
		await invoke('reset_simulation', { simulationId: this.simulationId });
	}

	async destroy(): Promise<void> {
		await this.pause();
		if (this.simulationId) {
			await invoke('destroy_simulation', { simulationId: this.simulationId });
		}
		this.stateUpdateCallbacks = [];
		this.simulationId = null;
	}

	async addProcess(process: Process): Promise<void> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}
		await invoke('add_process', { simulationId: this.simulationId, process });
	}

	async removeProcess(processId: string): Promise<void> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}
		await invoke('remove_process', { simulationId: this.simulationId, processId });
	}

	async getProcesses(): Promise<Process[]> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}
		const processes = await invoke<Process[]>('get_processes', { simulationId: this.simulationId });
		return processes;
	}

	async addConnection(connection: Connection): Promise<void> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}
		await invoke('add_connection', { simulationId: this.simulationId, connection });
	}

	async removeConnection(connectionId: string): Promise<void> {
		if (!this.simulationId) {
			throw new Error('Simulation not initialised');
		}
		await invoke('remove_connection', { simulationId: this.simulationId, connectionId });
	}
}
