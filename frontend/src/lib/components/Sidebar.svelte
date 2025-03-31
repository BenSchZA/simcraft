<script lang="ts">
	import { onMount } from 'svelte';
	import { storageManager } from '$lib/storage/StorageManager';
	import type { ModelMetadata } from '$lib/stores/simulation';
	import { activeModelId, sidebarVisible, openModels, runningStates } from '$lib/stores/simulation';
	import { v4 as uuidv4 } from 'uuid';
	import { useDnD } from '$lib/utils/dnd';

	let models: ModelMetadata[] = [];
	let searchTerm = '';
	const type = useDnD();

	onMount(async () => {
		await loadModels();
	});

	async function loadModels() {
		models = await storageManager.listModels();
		models.sort((a, b) => b.lastModified - a.lastModified);
	}

	const onDragStart = (event: DragEvent, nodeType: string) => {
		if (!event.dataTransfer) {
			return;
		}
		type.set(nodeType);
		event.dataTransfer.effectAllowed = 'move';
	};

	async function createNewModel() {
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
		await loadModels();
		openModels.update((models) => {
			models.set(newModel.id, newModel);
			return models;
		});
		$activeModelId = newModel.id;
	}

	async function deleteModel(id: string) {
		await storageManager.deleteModel(id);
		await loadModels();
		openModels.update((models) => {
			models.delete(id);
			return models;
		});
		if ($activeModelId === id) {
			$activeModelId = null;
		}
	}

	async function openModel(model: ModelMetadata) {
		const loadedModel = await storageManager.loadModel(model.id);
		if (loadedModel) {
			openModels.update((models) => {
				models.set(model.id, loadedModel);
				return models;
			});
			$activeModelId = model.id;
		}
	}

	$: filteredModels = searchTerm
		? models.filter((m) => m.name.toLowerCase().includes(searchTerm.toLowerCase()))
		: models;
</script>

<div class="sidebar" class:hidden={!$sidebarVisible}>
	<div class="sidebar-header">
		<h2>Models</h2>
		<button on:click={() => ($sidebarVisible = false)} class="close-button">×</button>
	</div>

	<div class="node-palette">
		<h3>Node Types</h3>
		<div class="node-types">
			<div
				class="node source-node"
				draggable={true}
				on:dragstart={(event) => onDragStart(event, 'source')}
				role="button"
				tabindex="0"
			>
				Source
			</div>
			<div
				class="node pool-node"
				draggable={true}
				on:dragstart={(event) => onDragStart(event, 'pool')}
				role="button"
				tabindex="0"
			>
				Pool
			</div>
		</div>
	</div>

	<div class="search-bar">
		<input type="text" bind:value={searchTerm} placeholder="Search models..." />
	</div>

	<button class="new-model-button" on:click={createNewModel}> New Model </button>

	<div class="models-list">
		{#each filteredModels as model}
			<div
				class="model-item"
				class:active={$activeModelId === model.id}
				class:running={$runningStates[model.id]}
				on:click={() => {
					openModel(model).catch((error) => {
						console.error('Failed to open model:', error);
					});
				}}
				on:keydown={(e) => e.key === 'Enter' && openModel(model)}
				tabindex="0"
				role="button"
			>
				<span class="model-name">{model.name}</span>
				<span class="model-date">
					{new Date(model.lastModified).toLocaleDateString()}
				</span>
				<button class="delete-button" on:click|stopPropagation={() => deleteModel(model.id)}>
					×
				</button>
			</div>
		{/each}
	</div>
</div>

<style>
	.sidebar {
		width: 250px;
		height: 100%;
		background-color: #1e1e1e;
		color: #ffffff;
		border-right: 1px solid #333;
		display: flex;
		flex-direction: column;
		transition: transform 0.3s ease;
	}

	.hidden {
		transform: translateX(-100%);
	}

	.sidebar-header {
		padding: 1rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
		border-bottom: 1px solid #333;
	}

	.node-palette {
		padding: 1rem;
		border-bottom: 1px solid #333;
	}

	.node-palette h3 {
		margin: 0 0 1rem 0;
		font-size: 0.9rem;
		color: #888;
	}

	.node-types {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.node {
		padding: 0.5rem;
		border-radius: 4px;
		cursor: grab;
		font-size: 0.9rem;
		min-width: 80px;
		text-align: center;
	}

	.source-node {
		background-color: #2ea44f;
		border: 1px solid #2c974b;
	}

	.pool-node {
		background-color: #1e1e1e;
		border: 1px solid #404040;
	}

	.close-button {
		background: none;
		border: none;
		color: #fff;
		font-size: 1.5rem;
		cursor: pointer;
	}

	.search-bar {
		padding: 1rem;
	}

	.search-bar input {
		width: 100%;
		padding: 0.5rem;
		background-color: #2d2d2d;
		border: 1px solid #404040;
		color: #fff;
		border-radius: 4px;
	}

	.new-model-button {
		margin: 0 1rem 1rem;
		padding: 0.5rem;
		background-color: #2ea44f;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}

	.new-model-button:hover {
		background-color: #2c974b;
	}

	.models-list {
		flex: 1;
		overflow-y: auto;
	}

	.model-item {
		padding: 0.75rem 1rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
		cursor: pointer;
		border-left: 3px solid transparent;
		position: relative;
	}

	.model-item:hover {
		background-color: #2d2d2d;
	}

	.model-item.active {
		background-color: #2d2d2d;
	}

	.model-item.running::before {
		content: '';
		position: absolute;
		left: -3px;
		top: 0;
		width: 3px;
		height: 100%;
		background: #2ea44f;
	}

	.model-name {
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.model-date {
		font-size: 0.8rem;
		color: #888;
		margin: 0 0.5rem;
	}

	.delete-button {
		background: none;
		border: none;
		color: #666;
		cursor: pointer;
		padding: 0 0.5rem;
	}

	.delete-button:hover {
		color: #ff4444;
	}
</style>
