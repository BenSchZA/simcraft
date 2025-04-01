export interface ProcessState {
	Source?: {
		resources_produced: number;
	};
	Pool?: {
		resources: number;
	};
	Drain?: {
		resources_consumed: number;
	};
	Delay?: {
		resources_received: number;
		resources_released: number;
	};
	Stepper?: {
		current_step: number;
	};
}

export interface SourceState {
	resources_produced: number;
}

export interface PoolState {
	resources: number;
}

export interface DrainState {
	resources_consumed: number;
}

export interface DelayState {
	resources_received: number;
	resources_released: number;
}

export interface StepperState {
	current_step: number;
}

export interface SimulationState {
	time: number;
	step: number;
	process_states: Record<string, ProcessState>;
}

export enum ProcessType {
	Source = 'Source',
	Pool = 'Pool',
	Drain = 'Drain',
	Delay = 'Delay',
	Stepper = 'Stepper'
}

export interface EventPayload {
	[key: string]: any;
}

export interface Event {
	source_id: string;
	source_port: string | null;
	target_id: string;
	target_port: string | null;
	time: number;
	payload: EventPayload;
}

export interface SimulationResult {
	events: Event[];
	state: SimulationState;
}

export interface Process {
	type: string;
	id: string;
}

export interface Connection {
	id: string;
	sourceID: string;
	sourcePort: string;
	targetID: string;
	targetPort: string;
	flowRate: number;
}

type StateUpdateCallback = (state: SimulationState[]) => void;

export interface SimcraftAdapter {
	initialise(processes: Process[], connections: Connection[]): Promise<SimulationState>;
	isInitialized(): boolean;
	isRunning(): boolean;
	step(): Promise<SimulationResult>;
	play(delayMs: number): Promise<boolean>;
	pause(): Promise<boolean>;
	onStateUpdate(callback: StateUpdateCallback): void;
	reset(): Promise<void>;
	destroy(): Promise<void>;
	addProcess(process: Process): Promise<void>;
	removeProcess(processId: string): Promise<void>;
	addConnection(connection: Connection): Promise<void>;
	removeConnection(connectionId: string): Promise<void>;
	getProcesses(): Promise<Process[]>;
}
