export type ProcessState =
	| { Source: SourceState }
	| { Pool: PoolState };

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
}

type StateUpdateCallback = (state: SimulationState[]) => void;

export interface SimcraftAdapter {
	initialise(processes: Process[], connections: Connection[]): Promise<SimulationState>;
	step(): Promise<SimulationResult>;
	play(delayMs: number): Promise<boolean>;
	pause(): Promise<boolean>;
	onStateUpdate(callback: StateUpdateCallback): void;
	destroy(): Promise<void>;
}
