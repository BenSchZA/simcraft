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
	nodes: Node[];
	edges: Edge[];
	settings: {
		stepDelay: number;
	};
	lastModified: number;
}

export const activeModelId = writable<string | null>(null);
export const openModels = writable<Map<string, ModelMetadata>>(new Map());
export const sidebarVisible = writable(true);
export const shouldResetChart = writable<boolean>(false);
export const simulationInstances = writable<Map<string, SimulationInstance>>(new Map());

export function getOrCreateSimulationInstance(modelId: string): SimulationInstance {
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
