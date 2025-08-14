import { v4 as uuidv4 } from 'uuid';
import { storageManager } from '$lib/storage/StorageManager';
import { models, openModels, activeModelId } from './simulation';
import type { ModelMetadata } from './simulation';
import { get } from 'svelte/store';
import {
	ProcessType,
	type ProcessNode,
	type ConnectionEdge,
	createConnectionEdge
} from '$lib/simcraft/base';

// Example models
import simpleFlowExample from '$lib/examples/simple-flow.json';
import manufacturingLineExample from '$lib/examples/manufacturing-line.json';
import branchingFlowExample from '$lib/examples/branching-flow.json';

export async function loadModels(): Promise<ModelMetadata[]> {
	const models = await storageManager.listModels();
	models.sort((a, b) => b.lastModified - a.lastModified);
	return models;
}

export async function loadRecentModels(limit: number = 5): Promise<ModelMetadata[]> {
	const models = await loadModels();
	return models.slice(0, limit);
}

export async function createNewModel() {
	const newModel = {
		id: uuidv4(),
		name: 'Model',
		nodes: [],
		edges: [],
		settings: {
			stepDelay: 100
		},
		lastModified: Date.now()
	};

	await storageManager.saveModel(newModel);
	models.set(await loadModels());

	openModels.update((models) => {
		models.set(newModel.id, newModel);
		return models;
	});

	activeModelId.set(newModel.id);
}

export async function openModel(model: ModelMetadata) {
	const loadedModel = await storageManager.loadModel(model.id);
	if (loadedModel) {
		openModels.update((models) => {
			models.set(model.id, loadedModel);
			return models;
		});

		activeModelId.set(model.id);
	}
}

export async function deleteModel(id: string) {
	await storageManager.deleteModel(id);
	models.set(await loadModels());
	openModels.update((models) => {
		models.delete(id);
		return models;
	});

	if (get(activeModelId) === id) {
		activeModelId.set(null);
	}
}

// Example models interface
export interface ExampleModel {
	name: string;
	description: string;
	processes: any[];
	connections: any[];
}

// Convert JSON process to ProcessNode
function processToNode(process: any, position?: { x: number; y: number }): ProcessNode {
	const { id, type, ...settings } = process;

	return {
		id,
		type: type as ProcessType,
		position: position || { x: 0, y: 0 },
		data: {
			settings: {
				id,
				type: type as ProcessType,
				...settings
			},
			label: type
		}
	};
}

// Convert JSON connection to ConnectionEdge
function connectionToEdge(connection: any): ConnectionEdge {
	const { id, sourceID, targetID, sourcePort, targetPort, flowRate } = connection;

	return createConnectionEdge(id, sourceID, targetID, sourcePort, targetPort, { flowRate });
}

// Get available example models
export function getExampleModels(): ExampleModel[] {
	return [
		simpleFlowExample as ExampleModel,
		manufacturingLineExample as ExampleModel,
		branchingFlowExample as ExampleModel
	];
}

// Load an example model
export async function loadExampleModel(example: ExampleModel) {
	// Convert processes to ProcessNodes, using positions from display.position
	const nodes: ProcessNode[] = example.processes.map((process) => {
		const position = process.display?.position || { x: 100, y: 100 };
		return processToNode(process, position);
	});

	// Convert connections to ConnectionEdges
	const edges: ConnectionEdge[] = example.connections.map(connectionToEdge);

	const newModel = {
		id: uuidv4(),
		name: example.name,
		nodes,
		edges,
		settings: {
			stepDelay: 100
		},
		lastModified: Date.now()
	};

	await storageManager.saveModel(newModel);
	models.set(await loadModels());

	openModels.update((models) => {
		models.set(newModel.id, newModel);
		return models;
	});

	activeModelId.set(newModel.id);
}
