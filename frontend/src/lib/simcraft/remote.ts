import type {
	SimcraftAdapter,
	Process,
	Connection,
	SimulationState,
	SimulationResult,
	Event
} from './base';
import { io, Socket } from 'socket.io-client';

type StateUpdateCallback = (state: SimulationState[]) => void;

export class RemoteAdapter implements SimcraftAdapter {
	private socket: Socket;
	private stateUpdateCallbacks: StateUpdateCallback[] = [];
	private _isRunning = false;

	constructor(private url: string = 'ws://localhost:3030') {
		this.socket = io(url, {
			transports: ['websocket'],
			autoConnect: true,
			reconnection: true,
			reconnectionAttempts: 5,
			reconnectionDelay: 1000
		});

		this.socket.on('connect', () => {
			console.log('Connected to simulation server');
		});

		this.socket.on('connect_error', (error) => {
			console.error('Connection error:', error);
		});

		this.socket.on('disconnect', () => {
			console.log('Disconnected from simulation server');
		});

		this.socket.on('state_update', (state: SimulationState[]) => {
			const states = Array.isArray(state) ? state : [state];
			this.stateUpdateCallbacks.forEach((callback) => callback(states));
		});
	}

	onStateUpdate(callback: StateUpdateCallback) {
		this.stateUpdateCallbacks.push(callback);
		return () => {
			this.stateUpdateCallbacks = this.stateUpdateCallbacks.filter((cb) => cb !== callback);
		};
	}

	async initialise(processes: Process[], connections: Connection[]): Promise<SimulationState> {
		const data = await this.emitWithResponse<SimulationState>('init', {
			processes,
			connections
		});

		if (!data) {
			throw new Error('Expected a non-empty array from simulation server');
		}
		return data;
	}

	isInitialized(): boolean {
		return this.socket.connected;
	}

	isRunning(): boolean {
		return this._isRunning;
	}

	async step(): Promise<SimulationResult> {
		try {
			const events: Event[] = await this.emitWithResponse<Event[]>('step', {});
			const state: SimulationState = await this.emitWithResponse<SimulationState>('state', {});
			return { events, state };
		} catch (error) {
			throw new Error(`Failed to step simulation: ${error}`);
		}
	}

	async play(delayMs: number): Promise<boolean> {
		this._isRunning = true;
		return await this.emitWithResponse<boolean>('play', { delay_ms: delayMs });
	}

	async pause(): Promise<boolean> {
		this._isRunning = false;
		return await this.emitWithResponse<boolean>('pause', {});
	}

	private async emitWithResponse<T>(event: string, data: any): Promise<T> {
		if (!this.socket.connected) {
			throw new Error('Socket is not connected');
		}

		return new Promise((resolve, reject) => {
			this.socket.timeout(30000).emit(event, data, (err: Error | null, response: T) => {
				if (err) {
					reject(err);
				} else {
					resolve(response);
				}
			});
		});
	}

	async reset(): Promise<void> {
		this._isRunning = false;
		return await this.emitWithResponse<void>('reset', {});
	}

	async destroy(): Promise<void> {
		this._isRunning = false;
		this.stateUpdateCallbacks = [];
		this.socket.disconnect();
	}
}
