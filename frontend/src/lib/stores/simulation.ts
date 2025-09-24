import { writable, derived, get } from 'svelte/store';
import type { ProcessNode, ConnectionEdge, Connection } from '$lib/simcraft/base';
import type { SimulationState } from '$lib/simcraft/base';
import { BrowserAdapter } from '$lib/simcraft/browser';
import type { SimcraftAdapter } from '$lib/simcraft/base';
import { nodeToProcess, edgeToConnection, ProcessType } from '$lib/simcraft/base';

export interface SimulationInstance {
	adapter: SimcraftAdapter;
	currentStep: number;
	maxSteps: number;
	isRunning: boolean;
	stateUpdateCallback: ((states: SimulationState[]) => void) | null;
	unsubscribe?: () => void;
}

export interface SimulationModel {
	id: string;
	name: string;
	nodes: ProcessNode[];
	edges: ConnectionEdge[];
	settings: {
		stepDelay: number;
	};
	lastModified: number;
}

export interface ModelMetadata {
	id: string;
	name: string;
	lastModified: number;
}

export const activeModelId = writable<string | null>(null);
export const models = writable<ModelMetadata[]>([]);
export const openModels = writable<Map<string, SimulationModel>>(new Map());
export const simulationInstances = writable<Map<string, SimulationInstance>>(new Map());

export const activeNodeId = writable<string | null>(null);
export const sidebarVisible = writable(true);
export const shouldResetChart = writable<boolean>(false);

export class SimulationError extends Error {
	constructor(message: string, cause?: unknown) {
		super(message);
		this.name = 'SimulationError';
		this.cause = cause;
	}
}

export async function getOrCreateSimulationInstance(modelId: string): Promise<SimulationInstance> {
	if (!modelId) {
		throw new Error('Model ID is required');
	}

	const instances = get(simulationInstances);
	if (!instances.has(modelId)) {
		console.log('Creating new simulation instance for model', modelId);

		const model = get(openModels).get(modelId);
		if (!model) {
			throw new SimulationError('Model not found');
		}
		console.log('Model', model);

		const adapter = new BrowserAdapter();

		const instance: SimulationInstance = {
			adapter,
			currentStep: 0,
			maxSteps: Infinity,
			isRunning: false,
			stateUpdateCallback: null,
			unsubscribe: undefined
		};

		// Convert nodes and edges to processes and connections
		const processes: any[] = model.nodes.map(nodeToProcess);
		// Add stepper process with correct format
		processes.push({ type: ProcessType.Stepper, id: 'stepper' });
		const connections: Connection[] = model.edges.map(edgeToConnection);

		console.log('Initialising simulation');
		const state = await instance.adapter.initialise(processes, connections);
		console.log('Initialised simulation', state);

		instances.set(modelId, instance);
		simulationInstances.set(instances);
	}
	return instances.get(modelId)!;
}

openModels.subscribe((models) => {
	const instances = get(simulationInstances);

	for (const [modelId, instance] of instances) {
		if (!models.has(modelId)) {
			if (instance.unsubscribe) {
				instance.unsubscribe();
			}
			instance.adapter.destroy().catch(console.error);
			instance.stateUpdateCallback = null;
			instances.delete(modelId);
		}
	}

	simulationInstances.set(instances);
});

export const activeSimulation = derived(
	[activeModelId, simulationInstances],
	([$activeModelId, $instances]) => {
		if (!$activeModelId) return null;
		return $instances.get($activeModelId) || null;
	}
);

export function setSimulationStateUpdateCallback(
	modelId: string,
	callback: (states: SimulationState[]) => void
) {
	simulationInstances.update((instances) => {
		const instance = instances.get(modelId);
		if (instance) {
			instance.stateUpdateCallback = callback;
			instance.adapter.onStateUpdate(callback);
		}
		return instances;
	});
}

export function setSimulationRunning(modelId: string, isRunning: boolean) {
	simulationInstances.update((instances) => {
		const instance = instances.get(modelId);
		if (instance) {
			instance.isRunning = isRunning;
			instances.set(modelId, instance);
		}
		return instances;
	});
}

export const isActiveSimulationRunning = derived(
	activeSimulation,
	($activeSimulation) => $activeSimulation?.isRunning || false
);

export const runningStates = derived([simulationInstances], ([$instances]) => {
	return Array.from($instances.entries()).reduce(
		(acc, [id, instance]) => {
			acc[id] = instance.isRunning;
			return acc;
		},
		{} as Record<string, boolean>
	);
});
