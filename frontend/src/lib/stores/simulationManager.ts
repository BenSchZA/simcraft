import { get } from 'svelte/store';
import { activeModelId, getInitialisedSimulation } from './simulation';
import { ProcessType, type Connection } from '$lib/simcraft';

export class SimulationError extends Error {
	constructor(message: string, cause?: unknown) {
		super(message);
		this.name = 'SimulationError';
		this.cause = cause;
	}
}

export async function addSimulationProcess(
	processType: ProcessType,
	nodeId: string
): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) return;

	try {
		const simulation = await getInitialisedSimulation(currentModelId);
		console.log('Adding process to simulation', processType, nodeId);

		await simulation.adapter.addProcess({
			type: processType,
			id: nodeId
		});
	} catch (error) {
		throw new SimulationError('Failed to add process to simulation', error);
	}
}

export async function removeSimulationProcess(nodeId: string): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) return;

	try {
		const simulation = await getInitialisedSimulation(currentModelId);
		await simulation.adapter.removeProcess(nodeId);
	} catch (error) {
		throw new SimulationError('Failed to remove process from simulation', error);
	}
}

export async function addSimulationConnection(connection: Connection): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) return;

	try {
		const simulation = await getInitialisedSimulation(currentModelId);
		await simulation.adapter.addConnection(connection);
	} catch (error) {
		throw new SimulationError('Failed to add connection to simulation', error);
	}
}

export async function removeSimulationConnection(connectionId: string): Promise<void> {
	const currentModelId = get(activeModelId);
	if (!currentModelId) return;

	try {
		const simulation = await getInitialisedSimulation(currentModelId);
		await simulation.adapter.removeConnection(connectionId);
	} catch (error) {
		throw new SimulationError('Failed to remove connection from simulation', error);
	}
}
