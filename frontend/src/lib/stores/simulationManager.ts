import { get } from 'svelte/store';
import { activeModelId, simulationInstances, openModels } from './simulation';
import type {
	ProcessType,
	ProcessNode,
	ConnectionEdge,
	ProcessSettingsType,
	ConnectionSettings,
	SourceSettings,
	PoolSettings,
	DrainSettings,
	DelaySettings,
	StepperSettings
} from '$lib/simcraft/base';
import {
	createProcessNode,
	createConnectionEdge,
	nodeToProcess,
	edgeToConnection,
	isSourceSettings,
	isPoolSettings,
	isDrainSettings,
	isDelaySettings,
	isStepperSettings
} from '$lib/simcraft/base';

export class SimulationError extends Error {
	constructor(message: string, cause?: unknown) {
		super(message);
		this.name = 'SimulationError';
		this.cause = cause;
	}
}

export async function addSimulationProcess(
	type: ProcessType,
	nodeId: string,
	position: { x: number; y: number },
	settings?: Partial<ProcessSettingsType>
): Promise<ProcessNode> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) throw new SimulationError('No active model');

	try {
		const instances = get(simulationInstances);
		const instance = instances.get(currentModelId);
		if (!instance?.adapter) {
			throw new SimulationError('Simulation not initialised');
		}

		// Create the process node with settings
		const node = createProcessNode(nodeId, type, position, settings);

		// Add to simulation
		await instance.adapter.addProcess(nodeToProcess(node));

		// Add to model
		openModels.update((models) => {
			const model = models.get(currentModelId);
			if (model) {
				const updatedModel = {
					...model,
					nodes: [...model.nodes, node],
					lastModified: Date.now()
				};
				models.set(currentModelId, updatedModel);
			}
			return models;
		});

		return node;
	} catch (error) {
		throw new SimulationError('Failed to add process to simulation', error);
	}
}

export async function updateSimulationProcess<T extends ProcessSettingsType>(
	nodeId: string,
	settings: Partial<Omit<T, 'type' | 'id'>>
): Promise<void> {
	console.log('updateSimulationProcess', nodeId, settings);
	const currentModelId = get(activeModelId);
	if (!currentModelId) throw new SimulationError('No active model');

	try {
		const instances = get(simulationInstances);
		const instance = instances.get(currentModelId);
		if (!instance?.adapter) {
			throw new SimulationError('Simulation not initialised');
		}

		// First, get the current node to update
		const models = get(openModels);
		const model = models.get(currentModelId);
		if (!model) {
			throw new SimulationError('Model not found');
		}

		const nodeIndex = model.nodes.findIndex((n) => n.id === nodeId);
		if (nodeIndex < 0) {
			throw new SimulationError('Node not found');
		}

		const node = model.nodes[nodeIndex];
		const currentSettings = node.data.settings;
		let updatedNode = { ...node };

		// Type guard checks to ensure type safety and create updated node
		if (isSourceSettings(currentSettings)) {
			const updatedSettings = {
				...currentSettings,
				...(settings as Partial<Omit<SourceSettings, 'type' | 'id'>>)
			};
			updatedNode.data.settings = updatedSettings;
		} else if (isPoolSettings(currentSettings)) {
			const updatedSettings = {
				...currentSettings,
				...(settings as Partial<Omit<PoolSettings, 'type' | 'id'>>)
			};
			updatedNode.data.settings = updatedSettings;
		} else if (isDrainSettings(currentSettings)) {
			const updatedSettings = {
				...currentSettings,
				...(settings as Partial<Omit<DrainSettings, 'type' | 'id'>>)
			};
			updatedNode.data.settings = updatedSettings;
		} else if (isDelaySettings(currentSettings)) {
			const updatedSettings = {
				...currentSettings,
				...(settings as Partial<Omit<DelaySettings, 'type' | 'id'>>)
			};
			updatedNode.data.settings = updatedSettings;
		} else if (isStepperSettings(currentSettings)) {
			// Stepper has no additional settings to update
			return;
		}

		// Try to update the backend first (atomic approach)
		const backendProcess = nodeToProcess(updatedNode);
		await instance.adapter.updateProcess(nodeId, backendProcess);

		// Only update the UI if backend update succeeds
		openModels.update((models) => {
			const model = models.get(currentModelId);
			if (model) {
				const updatedNodes = [...model.nodes];
				updatedNodes[nodeIndex] = updatedNode;
				const updatedModel = {
					...model,
					nodes: updatedNodes,
					lastModified: Date.now()
				};
				models.set(currentModelId, updatedModel);
			}
			return models;
		});
	} catch (error) {
		throw new SimulationError('Failed to update process in simulation', error);
	}
}

export async function removeSimulationProcess(nodeId: string): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) return;

	try {
		const instances = get(simulationInstances);
		const instance = instances.get(currentModelId);
		if (!instance?.adapter) {
			throw new SimulationError('Simulation not initialised');
		}

		await instance.adapter.removeProcess(nodeId);

		// Remove from model
		openModels.update((models) => {
			const model = models.get(currentModelId);
			if (model) {
				const updatedModel = {
					...model,
					nodes: model.nodes.filter((n) => n.id !== nodeId),
					lastModified: Date.now()
				};
				models.set(currentModelId, updatedModel);
			}
			return models;
		});
	} catch (error) {
		throw new SimulationError('Failed to remove process from simulation', error);
	}
}

export async function addSimulationConnection(
	id: string,
	sourceId: string,
	targetId: string,
	sourceHandle: string | null,
	targetHandle: string | null,
	settings?: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>
): Promise<ConnectionEdge> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) throw new SimulationError('No active model');

	try {
		const instances = get(simulationInstances);
		const instance = instances.get(currentModelId);
		if (!instance?.adapter) {
			throw new SimulationError('Simulation not initialised');
		}

		// Create the connection edge
		const edge = createConnectionEdge(id, sourceId, targetId, sourceHandle, targetHandle, settings);

		// Add to simulation
		await instance.adapter.addConnection(edgeToConnection(edge));

		// Add to model
		openModels.update((models) => {
			const model = models.get(currentModelId);
			if (model) {
				const updatedModel = {
					...model,
					edges: [...model.edges, edge],
					lastModified: Date.now()
				};
				models.set(currentModelId, updatedModel);
			}
			return models;
		});

		return edge;
	} catch (error) {
		throw new SimulationError('Failed to add connection to simulation', error);
	}
}

export async function updateSimulationConnection(
	connectionId: string,
	settings: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>
): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) throw new SimulationError('No active model');

	try {
		const instances = get(simulationInstances);
		const instance = instances.get(currentModelId);
		if (!instance?.adapter) {
			throw new SimulationError('Simulation not initialised');
		}

		// First, get the current connection to update
		const models = get(openModels);
		const model = models.get(currentModelId);
		if (!model) {
			throw new SimulationError('Model not found');
		}

		const edgeIndex = model.edges.findIndex((e) => e.id === connectionId);
		if (edgeIndex < 0) {
			throw new SimulationError('Connection not found');
		}

		const edge = model.edges[edgeIndex];
		const updatedEdge: ConnectionEdge = {
			...edge,
			data: {
				...edge.data,
				settings: {
					...edge.data.settings,
					...settings
				}
			}
		};

		// Try to update the backend first (atomic approach)
		await instance.adapter.updateConnection(connectionId, edgeToConnection(updatedEdge));

		// Only update the UI if backend update succeeds
		openModels.update((models) => {
			const model = models.get(currentModelId);
			if (model) {
				const updatedEdges = [...model.edges];
				updatedEdges[edgeIndex] = updatedEdge;
				const updatedModel = {
					...model,
					edges: updatedEdges,
					lastModified: Date.now()
				};
				models.set(currentModelId, updatedModel);
			}
			return models;
		});
	} catch (error) {
		throw new SimulationError('Failed to update connection in simulation', error);
	}
}

export async function removeSimulationConnection(connectionId: string): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) return;

	try {
		const instances = get(simulationInstances);
		const instance = instances.get(currentModelId);
		if (!instance?.adapter) {
			throw new SimulationError('Simulation not initialised');
		}

		await instance.adapter.removeConnection(connectionId);

		// Remove from model
		openModels.update((models) => {
			const model = models.get(currentModelId);
			if (model) {
				const updatedModel = {
					...model,
					edges: model.edges.filter((e) => e.id !== connectionId),
					lastModified: Date.now()
				};
				models.set(currentModelId, updatedModel);
			}
			return models;
		});
	} catch (error) {
		throw new SimulationError('Failed to remove connection from simulation', error);
	}
}
