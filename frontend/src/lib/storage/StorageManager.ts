import { openDB, type IDBPDatabase } from 'idb';
import * as Automerge from '@automerge/automerge';
import type { SimulationModel, ModelMetadata } from '$lib/stores/simulation';

const DB_NAME = 'simcraft-flow';
const DB_VERSION = 1;
const MODELS_STORE = 'models';
const METADATA_STORE = 'metadata';

export class StorageManager {
	private db: IDBPDatabase | null = null;

	async init() {
		this.db = await openDB(DB_NAME, DB_VERSION, {
			upgrade(db) {
				// Store for Automerge documents
				db.createObjectStore(MODELS_STORE);
				// Store for quick access to model metadata
				db.createObjectStore(METADATA_STORE, { keyPath: 'id' });
			}
		});
	}

	async saveModel(model: SimulationModel): Promise<void> {
		if (!this.db) await this.init();

		const doc = Automerge.from(model);
		const binary = Automerge.save(doc);

		await this.db!.put(MODELS_STORE, binary, model.id);
		await this.db!.put(METADATA_STORE, {
			id: model.id,
			name: model.name,
			lastModified: model.lastModified
		});
	}

	async loadModel(id: string): Promise<SimulationModel | null> {
		if (!this.db) await this.init();

		const binary = await this.db!.get(MODELS_STORE, id);
		if (!binary) return null;

		const doc = Automerge.load<SimulationModel>(binary);
		return Automerge.toJS(doc);
	}

	async listModels(): Promise<ModelMetadata[]> {
		if (!this.db) await this.init();
		return this.db!.getAll(METADATA_STORE);
	}

	async deleteModel(id: string): Promise<void> {
		if (!this.db) await this.init();
		await this.db!.delete(MODELS_STORE, id);
		await this.db!.delete(METADATA_STORE, id);
	}
}

// Export a singleton instance
export const storageManager = new StorageManager();
