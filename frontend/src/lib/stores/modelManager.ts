import { v4 as uuidv4 } from 'uuid';
import { storageManager } from '$lib/storage/StorageManager';
import { models, openModels, activeModelId } from './simulation';
import type { ModelMetadata } from './simulation';
import { get } from 'svelte/store';

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
