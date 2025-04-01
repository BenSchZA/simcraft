import { writable, derived, get } from 'svelte/store';
import type { Node, Edge } from '@xyflow/svelte';
import type { SimcraftAdapter } from '$lib/simcraft';
import { createAdapter } from '$lib/simcraft';

export interface SimulationInstance {
	adapter: SimcraftAdapter;
	stepDelay: number;
	isRunning: boolean;
	unsubscribe?: () => void;
}

export interface SimulationModel {
	id: string;
	name: string;
	nodes: Node[];
	edges: Edge[];
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
export const sidebarVisible = writable(true);
export const shouldResetChart = writable<boolean>(false);
export const simulationInstances = writable<Map<string, SimulationInstance>>(new Map());

export class SimulationError extends Error {
	constructor(message: string, cause?: unknown) {
		super(message);
		this.name = 'SimulationError';
		this.cause = cause;
	}
}

function getOrCreateSimulationInstance(modelId: string): SimulationInstance {
	if (!modelId) {
		throw new Error('Model ID is required');
	}

	const instances = get(simulationInstances);
	if (!instances.has(modelId)) {
		const adapter = createAdapter();
		const instance: SimulationInstance = {
			adapter,
			stepDelay: 100,
			isRunning: false,
			unsubscribe: undefined
		};

		instances.set(modelId, instance);
		simulationInstances.set(instances);
	}
	return instances.get(modelId)!;
}

export async function getInitialisedSimulation(modelId: string): Promise<SimulationInstance> {
	if (!modelId) {
		throw new SimulationError('Model ID is required');
	}

	const model = get(openModels).get(modelId);
	if (!model) {
		throw new SimulationError('Model not found');
	}

	const simulation = getOrCreateSimulationInstance(modelId);
	if (simulation.adapter.isInitialized()) {
		return simulation;
	}

	try {
		const processes = [
			{ type: 'Stepper', id: 'stepper' },
			...model.nodes.map((node) => ({
				type: node.type!,
				id: node.id
			}))
		];

		const connections = model.edges.map((edge) => ({
			id: edge.id,
			sourceID: edge.source,
			sourcePort: 'out',
			targetID: edge.target,
			targetPort: 'in',
			// TODO Debug why flowRate isn't set sometimes
			flowRate: (edge.data?.flowRate as number) ?? 1.0
		}));

		const result = await simulation.adapter.initialise(processes, connections);
		if (result === null) {
			throw new SimulationError('Failed to initialise simulation');
		}
		return simulation;
	} catch (error) {
		throw new SimulationError('Failed to initialise simulation', error);
	}
}

// Automatically create simulation instances for open models
openModels.subscribe((models) => {
	const instances = get(simulationInstances);

	// Create instances for new models
	for (const [modelId] of models) {
		if (!instances.has(modelId)) {
			getOrCreateSimulationInstance(modelId);
		}
	}

	// Cleanup instances for closed models
	for (const [modelId, instance] of instances) {
		if (!models.has(modelId)) {
			if (instance.unsubscribe) {
				instance.unsubscribe();
			}
			instance.adapter.destroy();
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
