import { executeCommand } from './modelCommands';
import type {
	ProcessType,
	ProcessSettingsType,
	ConnectionSettings,
} from '$lib/simcraft/base';

export async function addProcessCommand(
	type: ProcessType,
	nodeId: string,
	position: { x: number; y: number },
	settings?: Partial<ProcessSettingsType>
): Promise<void> {
	return executeCommand({
		type: 'ADD_PROCESS',
		payload: { processType: type, nodeId, position, settings }
	});
}

export async function removeProcessCommand(nodeId: string): Promise<void> {
	return executeCommand({
		type: 'REMOVE_PROCESS',
		payload: { nodeId }
	});
}

export async function addConnectionCommand(
	id: string,
	sourceId: string,
	targetId: string,
	sourceHandle: string | null,
	targetHandle: string | null,
	settings?: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>
): Promise<void> {
	return executeCommand({
		type: 'ADD_CONNECTION',
		payload: { id, sourceId, targetId, sourceHandle, targetHandle, settings }
	});
}

export async function removeConnectionCommand(connectionId: string): Promise<void> {
	return executeCommand({
		type: 'REMOVE_CONNECTION',
		payload: { connectionId }
	});
}

export async function updateProcessCommand(
	nodeId: string,
	settings: Partial<ProcessSettingsType>
): Promise<void> {
	return executeCommand({
		type: 'UPDATE_PROCESS',
		payload: { nodeId, settings }
	});
}

export async function updateConnectionCommand(
	connectionId: string,
	settings: Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>
): Promise<void> {
	return executeCommand({
		type: 'UPDATE_CONNECTION',
		payload: { connectionId, settings }
	});
}
