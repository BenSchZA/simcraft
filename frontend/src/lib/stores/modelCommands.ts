import { get } from 'svelte/store';
import { activeModelId, openModels, getOrCreateSimulationInstance } from './simulation';
import { storageManager } from '$lib/storage/StorageManager';
import type {
	ProcessType,
	ProcessSettingsType,
	ConnectionSettings
} from '$lib/simcraft/base';
import {
	createProcessNode,
	createConnectionEdge,
	nodeToProcess,
	edgeToConnection
} from '$lib/simcraft/base';

export class ModelCommandError extends Error {
	constructor(message: string, cause?: unknown) {
		super(message);
		this.name = 'ModelCommandError';
		this.cause = cause;
	}
}

export interface AddProcessCommand {
	type: 'ADD_PROCESS';
	payload: {
		processType: ProcessType;
		nodeId: string;
		position: { x: number; y: number };
		settings?: Partial<ProcessSettingsType>;
	};
}

export interface RemoveProcessCommand {
	type: 'REMOVE_PROCESS';
	payload: {
		nodeId: string;
	};
}

export interface AddConnectionCommand {
	type: 'ADD_CONNECTION';
	payload: {
		id: string;
		sourceId: string;
		targetId: string;
		sourceHandle: string | null;
		targetHandle: string | null;
		settings?: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>;
	};
}

export interface RemoveConnectionCommand {
	type: 'REMOVE_CONNECTION';
	payload: {
		connectionId: string;
	};
}

export interface UpdateProcessCommand {
	type: 'UPDATE_PROCESS';
	payload: {
		nodeId: string;
		settings: Partial<ProcessSettingsType>;
	};
}

export interface UpdateConnectionCommand {
	type: 'UPDATE_CONNECTION';
	payload: {
		connectionId: string;
		settings: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>;
	};
}

export type ModelCommand = 
	| AddProcessCommand 
	| RemoveProcessCommand 
	| AddConnectionCommand 
	| RemoveConnectionCommand
	| UpdateProcessCommand
	| UpdateConnectionCommand;

export async function executeCommand(command: ModelCommand): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) {
		throw new ModelCommandError('No active model');
	}

	const models = get(openModels);
	const model = models.get(currentModelId);
	if (!model) {
		throw new ModelCommandError('Model not found');
	}

	const instance = await getOrCreateSimulationInstance(currentModelId);
	if (!instance?.adapter) {
		throw new ModelCommandError('Failed to create simulation instance');
	}

	try {
		switch (command.type) {
			case 'ADD_PROCESS':
				await executeAddProcess(currentModelId, model, instance, command.payload);
				break;
			case 'REMOVE_PROCESS':
				await executeRemoveProcess(currentModelId, model, instance, command.payload);
				break;
			case 'ADD_CONNECTION':
				await executeAddConnection(currentModelId, model, instance, command.payload);
				break;
			case 'REMOVE_CONNECTION':
				await executeRemoveConnection(currentModelId, model, instance, command.payload);
				break;
			case 'UPDATE_PROCESS':
				await executeUpdateProcess(currentModelId, model, instance, command.payload);
				break;
			case 'UPDATE_CONNECTION':
				await executeUpdateConnection(currentModelId, model, instance, command.payload);
				break;
			default:
				throw new ModelCommandError(`Unknown command type: ${(command as any).type}`);
		}
	} catch (error) {
		throw new ModelCommandError(`Command execution failed: ${command.type}`, error);
	}
}

async function executeAddProcess(
	modelId: string, 
	model: any, 
	instance: any, 
	payload: AddProcessCommand['payload']
): Promise<void> {
	const { processType, nodeId, position, settings } = payload;
	
	const node = createProcessNode(nodeId, processType, position, settings);
	
	await instance.adapter.addProcess(nodeToProcess(node));
	
	await updateModelState(modelId, {
		...model,
		nodes: [...model.nodes, node],
		lastModified: Date.now()
	});
}

async function executeRemoveProcess(
	modelId: string,
	model: any,
	instance: any,
	payload: RemoveProcessCommand['payload']
): Promise<void> {
	const { nodeId } = payload;
	
	await instance.adapter.removeProcess(nodeId);
	
	await updateModelState(modelId, {
		...model,
		nodes: model.nodes.filter((n: any) => n.id !== nodeId),
		lastModified: Date.now()
	});
}

async function executeAddConnection(
	modelId: string,
	model: any,
	instance: any,
	payload: AddConnectionCommand['payload']
): Promise<void> {
	const { id, sourceId, targetId, sourceHandle, targetHandle, settings } = payload;
	
	const edge = createConnectionEdge(id, sourceId, targetId, sourceHandle, targetHandle, settings);
	
	await instance.adapter.addConnection(edgeToConnection(edge));
	
	await updateModelState(modelId, {
		...model,
		edges: [...model.edges, edge],
		lastModified: Date.now()
	});
}

async function executeRemoveConnection(
	modelId: string,
	model: any,
	instance: any,
	payload: RemoveConnectionCommand['payload']
): Promise<void> {
	const { connectionId } = payload;
	
	await instance.adapter.removeConnection(connectionId);
	
	await updateModelState(modelId, {
		...model,
		edges: model.edges.filter((e: any) => e.id !== connectionId),
		lastModified: Date.now()
	});
}

async function executeUpdateProcess(
	modelId: string,
	model: any,
	instance: any,
	payload: UpdateProcessCommand['payload']
): Promise<void> {
	const { nodeId, settings } = payload;
	
	const nodeIndex = model.nodes.findIndex((n: any) => n.id === nodeId);
	if (nodeIndex < 0) {
		throw new ModelCommandError('Node not found');
	}

	const updatedNode = {
		...model.nodes[nodeIndex],
		data: {
			...model.nodes[nodeIndex].data,
			settings: {
				...model.nodes[nodeIndex].data.settings,
				...settings
			}
		}
	};

	await instance.adapter.updateProcess(nodeId, nodeToProcess(updatedNode));
	
	const updatedNodes = [...model.nodes];
	updatedNodes[nodeIndex] = updatedNode;
	
	await updateModelState(modelId, {
		...model,
		nodes: updatedNodes,
		lastModified: Date.now()
	});
}

async function executeUpdateConnection(
	modelId: string,
	model: any,
	instance: any,
	payload: UpdateConnectionCommand['payload']
): Promise<void> {
	const { connectionId, settings } = payload;
	
	const edgeIndex = model.edges.findIndex((e: any) => e.id === connectionId);
	if (edgeIndex < 0) {
		throw new ModelCommandError('Connection not found');
	}

	const updatedEdge = {
		...model.edges[edgeIndex],
		data: {
			...model.edges[edgeIndex].data,
			settings: {
				...model.edges[edgeIndex].data.settings,
				...settings
			}
		}
	};

	await instance.adapter.updateConnection(connectionId, edgeToConnection(updatedEdge));
	
	const updatedEdges = [...model.edges];
	updatedEdges[edgeIndex] = updatedEdge;
	
	await updateModelState(modelId, {
		...model,
		edges: updatedEdges,
		lastModified: Date.now()
	});
}

async function updateModelState(modelId: string, updatedModel: any): Promise<void> {
	openModels.update((models) => {
		models.set(modelId, updatedModel);
		return models;
	});
	
	await storageManager.saveModel(updatedModel);
}