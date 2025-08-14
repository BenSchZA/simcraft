import type {
	SimcraftAdapter,
	Process,
	Connection,
	SimulationState,
	Event,
	SimulationResult
} from './base';

type StateUpdateCallback = (states: SimulationState[]) => void;
type MessageHandler = (data: any) => void;

export class BrowserAdapter implements SimcraftAdapter {
	private worker: Worker | null = null;
	private messageHandlers: Map<string, MessageHandler[]> = new Map();
	private pendingPromises: Map<
		string,
		{ resolve: (value: any) => void; reject: (reason: any) => void }
	> = new Map();
	public stateUpdateCallback: StateUpdateCallback | null = null;
	private _isRunning = false;
	private workerReady = false;
	private workerReadyPromise: Promise<void>;
	private workerReadyResolve!: () => void;

	constructor() {
		this.workerReadyPromise = new Promise((resolve) => {
			this.workerReadyResolve = resolve;
		});

		this.worker = new Worker(new URL('../workers/simulation.worker.ts', import.meta.url), {
			type: 'module'
		});
		this.setupWorkerHandlers();
	}

	private setupWorkerHandlers() {
		if (!this.worker) return;

		this.worker.onmessage = (e) => {
			const { type, error, ...data } = e.data;

			if (type === 'ready') {
				console.log('Worker ready');
				this.workerReady = true;
				this.workerReadyResolve();
			}

			// Handle state updates
			if (type === 'stateUpdate' && data.states) {
				this.stateUpdateCallback?.(data.states);
			}

			// Handle errors
			if (type === 'error') {
				const pendingPromise = this.pendingPromises.get(type);
				if (pendingPromise) {
					pendingPromise.reject(new Error(error));
					this.pendingPromises.delete(type);
				}
				return;
			}

			// Handle paused state
			if (type === 'paused') {
				this._isRunning = false;
			}

			// Resolve pending promises
			const pendingPromise = this.pendingPromises.get(type);
			if (pendingPromise) {
				pendingPromise.resolve(data);
				this.pendingPromises.delete(type);
			}

			// Call registered handlers
			const handlers = this.messageHandlers.get(type) || [];
			handlers.forEach((handler) => handler(e.data));
		};

		this.worker.onerror = (error) => {
			console.error('Worker error:', error);
			this._isRunning = false;
		};
	}

	private async sendMessage(type: string, data: any = {}): Promise<any> {
		if (!this.worker) {
			throw new Error('Worker not initialised');
		}

		// Wait for worker to be ready before sending any messages
		await this.workerReadyPromise;

		return new Promise((resolve, reject) => {
			this.pendingPromises.set(type, { resolve, reject });
			this.worker!.postMessage({ type, ...data });
		});
	}

	async initialise(processes: Process[], connections: Connection[]): Promise<SimulationState> {
		const { state } = await this.sendMessage('initialise', { processes, connections });
		return state;
	}

	isInitialized(): boolean {
		return this.worker !== null && this.workerReady;
	}

	isRunning(): boolean {
		return this._isRunning;
	}

	async step(): Promise<SimulationResult> {
		return await this.sendMessage('step');
	}

	async stepUntil(until: number): Promise<SimulationResult> {
		return await this.sendMessage('stepUntil', { until });
	}

	async play(delayMs: number): Promise<boolean> {
		const { success } = await this.sendMessage('play', { stepDelay: delayMs });
		this._isRunning = success;
		return success;
	}

	async pause(): Promise<boolean> {
		const { success } = await this.sendMessage('pause');
		this._isRunning = !success;
		return success;
	}

	onStateUpdate(callback: StateUpdateCallback): void {
		this.stateUpdateCallback = callback;
	}

	async reset(): Promise<void> {
		await this.sendMessage('reset');
	}

	async destroy(): Promise<void> {
		await this.pause();
		this.stateUpdateCallback = null;
		if (this.worker) {
			this.worker.terminate();
			this.worker = null;
		}
		this.messageHandlers.clear();
		this.pendingPromises.clear();
	}

	async getState(): Promise<SimulationState> {
		const { states } = await this.sendMessage('getState');
		return states[0];
	}

	async addProcess(process: Process): Promise<void> {
		await this.sendMessage('addProcess', { process });
	}

	async removeProcess(processId: string): Promise<void> {
		await this.sendMessage('removeProcess', { processId });
	}

	async getProcesses(): Promise<Process[]> {
		const { processes } = await this.sendMessage('getProcesses');
		return processes;
	}

	async updateProcess(processId: string, process: Process): Promise<void> {
		const response = await this.sendMessage('updateProcess', { processId, process });
		if (!response.success) {
			throw new Error(response.error || 'Failed to update process');
		}
	}

	async addConnection(connection: Connection): Promise<void> {
		await this.sendMessage('addConnection', { connection });
	}

	async removeConnection(connectionId: string): Promise<void> {
		await this.sendMessage('removeConnection', { connectionId });
	}

	async updateConnection(connectionId: string, connection: Connection): Promise<void> {
		await this.sendMessage('updateConnection', { connectionId, connection });
	}

	async getCurrentStep(): Promise<number> {
		const { step } = await this.sendMessage('getCurrentStep');
		return step;
	}

	async getCurrentTime(): Promise<number> {
		const { time } = await this.sendMessage('getCurrentTime');
		return time;
	}
}
