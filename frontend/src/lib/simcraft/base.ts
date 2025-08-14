import type { Node, Edge } from '@xyflow/svelte';

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

// Enums for process settings - matching Rust backend exactly
export enum TriggerMode {
	Passive = 'Passive',
	Interactive = 'Interactive',
	Automatic = 'Automatic',
	Enabling = 'Enabling'
}

// Unified Action enum that matches Rust backend
export enum Action {
	PullAny = 'PullAny',
	PullAll = 'PullAll',
	PushAny = 'PushAny',
	PushAll = 'PushAll'
}

// Node-specific action constraints based on backend implementation
export const SourceActions = [Action.PushAny] as const; // PushAll is unimplemented
export const DrainActions = [Action.PullAny, Action.PullAll] as const;
export const PoolActions = [
	Action.PushAny,
	Action.PushAll,
	Action.PullAny,
	Action.PullAll
] as const;

// Type aliases for backward compatibility
export const SourceAction = Action;
export const PoolAction = Action;
export const DrainAction = Action;

export enum DelayAction {
	Delay = 'Delay',
	Queue = 'Queue'
}

export enum Overflow {
	Block = 'Block',
	Drain = 'Drain'
}

// Type alias for backward compatibility
export const PoolOverflow = Overflow;

// Base process settings interface
export interface ProcessSettings {
	id: string;
	type: ProcessType;
}

// Process-specific settings
export interface SourceSettings extends ProcessSettings {
	type: ProcessType.Source;
	triggerMode: TriggerMode;
	action: Action;
}

export interface PoolSettings extends ProcessSettings {
	type: ProcessType.Pool;
	triggerMode: TriggerMode;
	action: Action;
	overflow: Overflow;
	capacity: number;
}

export interface DrainSettings extends ProcessSettings {
	type: ProcessType.Drain;
	triggerMode: TriggerMode;
	action: Action;
}

export interface DelaySettings extends ProcessSettings {
	type: ProcessType.Delay;
	triggerMode: TriggerMode;
	action: DelayAction;
	releaseAmount: number;
	nextReleaseTime: number;
}

export interface StepperSettings extends ProcessSettings {
	type: ProcessType.Stepper;
	triggerMode: TriggerMode; // Always Automatic, but include for consistency
	dt: number; // Time step interval
}

// Union type for all process settings
export type ProcessSettingsType =
	| SourceSettings
	| PoolSettings
	| DrainSettings
	| DelaySettings
	| StepperSettings;

// Extended Node type for SvelteFlow with process settings
export interface ProcessNode extends Node {
	type: ProcessType;
	data: {
		settings: ProcessSettingsType;
		label?: string;
		[key: string]: any;
	};
}

// Connection settings
export interface ConnectionSettings {
	id: string;
	sourceId: string;
	targetId: string;
	sourcePort: string | null;
	targetPort: string | null;
	flowRate: number;
}

// Extended Edge type for SvelteFlow with connection settings
export interface ConnectionEdge extends Edge {
	data: {
		settings: ConnectionSettings;
		[key: string]: any;
	};
}

// Core simulation types
export interface Process {
	id: string;
	type: ProcessType;
	settings: ProcessSettingsType;
}

export interface Connection {
	id: string;
	sourceID: string;
	targetID: string;
	sourcePort: string | null;
	targetPort: string | null;
	flowRate: number;
}

// Type guard functions
export function isSourceSettings(settings: ProcessSettingsType): settings is SourceSettings {
	return settings.type === ProcessType.Source;
}

export function isPoolSettings(settings: ProcessSettingsType): settings is PoolSettings {
	return settings.type === ProcessType.Pool;
}

export function isDrainSettings(settings: ProcessSettingsType): settings is DrainSettings {
	return settings.type === ProcessType.Drain;
}

export function isDelaySettings(settings: ProcessSettingsType): settings is DelaySettings {
	return settings.type === ProcessType.Delay;
}

export function isStepperSettings(settings: ProcessSettingsType): settings is StepperSettings {
	return settings.type === ProcessType.Stepper;
}

// Helper functions to get valid actions for each process type
export function getValidActionsForProcessType(processType: ProcessType): readonly Action[] {
	switch (processType) {
		case ProcessType.Source:
			return SourceActions;
		case ProcessType.Drain:
			return DrainActions;
		case ProcessType.Pool:
			return PoolActions;
		default:
			return [];
	}
}

export function getValidDelayActions(): readonly DelayAction[] {
	return [DelayAction.Delay, DelayAction.Queue];
}

// Helper to check if an action is valid for a process type
export function isActionValidForProcessType(action: Action, processType: ProcessType): boolean {
	return getValidActionsForProcessType(processType).includes(action);
}

// Factory functions for creating process settings with defaults
export function createSourceSettings(
	id: string,
	partial: Partial<Omit<SourceSettings, 'id' | 'type'>> = {}
): SourceSettings {
	return {
		id,
		type: ProcessType.Source,
		triggerMode: TriggerMode.Automatic,
		action: SourceAction.PushAny,
		...partial
	};
}

export function createPoolSettings(
	id: string,
	partial: Partial<Omit<PoolSettings, 'id' | 'type'>> = {}
): PoolSettings {
	return {
		id,
		type: ProcessType.Pool,
		triggerMode: TriggerMode.Automatic,
		action: PoolAction.PushAny,
		overflow: PoolOverflow.Block,
		capacity: 10,
		...partial
	};
}

export function createDrainSettings(
	id: string,
	partial: Partial<Omit<DrainSettings, 'id' | 'type'>> = {}
): DrainSettings {
	return {
		id,
		type: ProcessType.Drain,
		triggerMode: TriggerMode.Automatic,
		action: DrainAction.PullAny,
		...partial
	};
}

export function createDelaySettings(
	id: string,
	partial: Partial<Omit<DelaySettings, 'id' | 'type'>> = {}
): DelaySettings {
	return {
		id,
		type: ProcessType.Delay,
		triggerMode: TriggerMode.Automatic,
		action: DelayAction.Delay,
		releaseAmount: 1,
		nextReleaseTime: 0,
		...partial
	};
}

export function createStepperSettings(
	id: string,
	partial: Partial<Omit<StepperSettings, 'id' | 'type'>> = {}
): StepperSettings {
	return {
		id,
		type: ProcessType.Stepper,
		triggerMode: TriggerMode.Automatic,
		dt: 1.0,
		...partial
	};
}

// Helper functions to convert between UI and simulation types
export function nodeToProcess(node: ProcessNode): any {
	// Convert UI node to backend format with flattened settings
	const settings = node.data.settings;
	const baseProcess = {
		id: settings.id,
		type: node.type
	};

	// Flatten the settings into the process object, excluding id and type
	const { id, type, ...processFields } = settings;
	return {
		...baseProcess,
		...processFields
	};
}

export function edgeToConnection(edge: ConnectionEdge): any {
	// Convert UI edge to backend Connection format
	return {
		id: edge.id as string,
		sourceID: edge.source,
		targetID: edge.target,
		sourcePort: edge.data.settings.sourcePort,
		targetPort: edge.data.settings.targetPort,
		flowRate: edge.data.settings.flowRate
	};
}

// Helper function to create a new connection edge
export function createConnectionEdge(
	id: string,
	source: string,
	target: string,
	sourceHandle: string | null,
	targetHandle: string | null,
	settings?: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>
): ConnectionEdge {
	return {
		id,
		source,
		target,
		sourceHandle,
		targetHandle,
		data: {
			settings: {
				id,
				sourceId: source,
				targetId: target,
				sourcePort: settings?.sourcePort ?? null,
				targetPort: settings?.targetPort ?? null,
				flowRate: settings?.flowRate ?? 1.0
			}
		}
	};
}

// Helper function to create a new process node
export function createProcessNode(
	id: string,
	type: ProcessType,
	position: { x: number; y: number },
	settings?: Partial<ProcessSettingsType>
): ProcessNode {
	let processSettings: ProcessSettingsType;

	switch (type) {
		case ProcessType.Source:
			processSettings = createSourceSettings(id, settings as Partial<SourceSettings>);
			break;
		case ProcessType.Pool:
			processSettings = createPoolSettings(id, settings as Partial<PoolSettings>);
			break;
		case ProcessType.Drain:
			processSettings = createDrainSettings(id, settings as Partial<DrainSettings>);
			break;
		case ProcessType.Delay:
			processSettings = createDelaySettings(id, settings as Partial<DelaySettings>);
			break;
		case ProcessType.Stepper:
			processSettings = createStepperSettings(id);
			break;
		default:
			throw new Error(`Unknown process type: ${type}`);
	}

	return {
		id,
		type,
		position,
		data: {
			settings: processSettings,
			label: type
		}
	};
}

type StateUpdateCallback = (state: SimulationState[]) => void;

export interface SimcraftAdapter {
	stateUpdateCallback: StateUpdateCallback | null;
	initialise(processes: Process[], connections: Connection[]): Promise<SimulationState>;
	isInitialized(): boolean;
	isRunning(): boolean;
	step(): Promise<SimulationResult>;
	stepUntil(until: number): Promise<SimulationResult>;
	play(delayMs: number): Promise<boolean>;
	pause(): Promise<boolean>;
	onStateUpdate(callback: StateUpdateCallback): void;
	reset(): Promise<void>;
	destroy(): Promise<void>;
	addProcess(process: Process): Promise<void>;
	removeProcess(processId: string): Promise<void>;
	updateProcess(processId: string, process: Process): Promise<void>;
	addConnection(connection: Connection): Promise<void>;
	removeConnection(connectionId: string): Promise<void>;
	updateConnection(connectionId: string, connection: Connection): Promise<void>;
	getProcesses(): Promise<Process[]>;
	getState(): Promise<SimulationState>;
	getCurrentStep(): Promise<number>;
	getCurrentTime(): Promise<number>;
}
