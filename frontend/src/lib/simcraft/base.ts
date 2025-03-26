export type ProcessState =
	| { Source: SourceState }
	| { Pool: PoolState }
	| { Stepper: StepperState };

export interface SourceState {
	resources_produced: number;
}

export interface PoolState {
	resources: number;
}

export interface StepperState {
	current_step: number;
}

export interface SimulationState {
	time: number;
	step: number;
	process_states: Record<string, ProcessState>;
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
}

export interface SimcraftAdapter {
	initialise(processes: Process[], connections: Connection[]): Promise<void>;
	step(): Promise<SimulationState[]>;
	step_until(until: number): Promise<SimulationState[]>;
	step_n(n: number): Promise<SimulationState[]>;
	destroy(): Promise<void>;
}
