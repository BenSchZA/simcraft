<script lang="ts">
	import { onMount } from 'svelte';
	import { activeModelId, sidebarVisible, runningStates, models } from '$lib/stores/simulation';
	import { useDnD } from '$lib/utils/dnd';
	import SourceNodeIcon from './nodes/node-icons/SourceNodeIcon.svelte';
	import PoolNodeIcon from './nodes/node-icons/PoolNodeIcon.svelte';
	import { loadModels, createNewModel, openModel, deleteModel } from '$lib/stores/modelManager';
	import { ProcessType } from '$lib/simcraft';
	import DrainNodeIcon from './nodes/node-icons/DrainNodeIcon.svelte';
	import DelayNodeIcon from './nodes/node-icons/DelayNodeIcon.svelte';

	let searchTerm = '';
	const type = useDnD();

	const nodeTypes = {
		[ProcessType.Source]: SourceNodeIcon,
		[ProcessType.Pool]: PoolNodeIcon,
		[ProcessType.Drain]: DrainNodeIcon,
		[ProcessType.Delay]: DelayNodeIcon
	};

	onMount(async () => {
		await loadModels().then((loadedModels) => {
			models.set(loadedModels);
		});
	});

	const onDragStart = (event: DragEvent, nodeType: ProcessType) => {
		if (!event.dataTransfer) {
			return;
		}
		type.set(nodeType);
		event.dataTransfer.effectAllowed = 'move';
	};

	$: filteredModels = searchTerm
		? $models.filter((m) => m.name.toLowerCase().includes(searchTerm.toLowerCase()))
		: $models;
</script>

<div class="sidebar" class:hidden={!$sidebarVisible}>
	<div class="sidebar-header">
		<h2>Models</h2>
		<button on:click={() => ($sidebarVisible = false)} class="close-button">×</button>
	</div>

	<div class="node-palette">
		<input type="text" placeholder="Search nodes..." class="node-search" />
		<div class="node-types">
			{#each Object.entries(nodeTypes) as [processType, icon]}
				<div
					class="node"
					draggable={true}
					on:dragstart={(event) => onDragStart(event, processType as ProcessType)}
					role="button"
					tabindex="0"
				>
					<svelte:component this={icon} classes="w-5 h-5" />
					<span class="node-label">{processType}</span>
				</div>
			{/each}
		</div>
	</div>

	<div class="search-bar">
		<input type="text" bind:value={searchTerm} placeholder="Search models..." />
	</div>

	<button class="new-model-button" on:click={createNewModel}>New Model</button>

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
		display: flex;
		flex-direction: column;
		transition: transform 0.3s ease;
		background: linear-gradient(180deg, rgba(250, 250, 250, 0.98) 0%, rgba(245, 245, 245, 0.98) 100%);
		border-right: 1px solid rgba(0, 0, 0, 0.08);
		box-shadow: 2px 0 8px rgba(0, 0, 0, 0.04);
	}

	.hidden {
		transform: translateX(-100%);
	}

	.sidebar-header {
		padding: 1rem 1rem 0.75rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
		border-bottom: 1px solid rgba(0, 0, 0, 0.06);
	}

	.sidebar-header h2 {
		font-size: 0.875rem;
		font-weight: 600;
		color: #374151;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.node-palette {
		padding: 1rem;
		border-bottom: 1px solid rgba(0, 0, 0, 0.06);
	}

	.node-search {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		outline: none;
		border: 1px solid rgba(0, 0, 0, 0.1);
		background: white;
		font-size: 0.875rem;
		transition: all 0.2s ease;
		color: #111827;
		margin-bottom: 0.75rem;
	}

	.node-search:focus {
		outline: 2px solid #3b82f6;
		outline-offset: -1px;
		border-color: transparent;
	}

	.node-search::placeholder {
		color: #9ca3af;
	}

	.node-types {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.node {
		padding: 0.5rem 0.75rem;
		border-radius: 8px;
		cursor: grab;
		font-size: 0.875rem;
		font-weight: 500;
		min-width: 85px;
		text-align: center;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
		transition: all 0.2s ease;
		color: #374151;
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.node:hover {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
		transform: translateY(-1px);
	}

	.node:hover :global(svg) {
		color: white;
	}

	.node:active {
		cursor: grabbing;
		transform: translateY(0);
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
	}

	.node-label {
		font-size: 0.8125rem;
	}

	.close-button {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		transition: all 0.2s ease;
		color: #6b7280;
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px;
		padding: 0;
	}

	.close-button:hover {
		background: rgba(0, 0, 0, 0.05);
		color: #374151;
	}

	.search-bar {
		padding: 0.75rem 1rem;
	}

	.search-bar input {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		outline: none;
		border: 1px solid rgba(0, 0, 0, 0.1);
		background: white;
		font-size: 0.875rem;
		transition: all 0.2s ease;
		color: #111827;
	}

	.search-bar input:focus {
		outline: 2px solid #3b82f6;
		outline-offset: -1px;
		border-color: transparent;
	}

	.search-bar input::placeholder {
		color: #9ca3af;
	}

	.new-model-button {
		margin: 0 1rem 0.75rem;
		padding: 0.625rem 1rem;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
		color: #374151;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.new-model-button:hover {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
		transform: translateY(-1px);
	}

	.new-model-button:active {
		transform: translateY(0);
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
	}

	.models-list {
		flex: 1;
		overflow-y: auto;
		padding: 0.5rem 0;
	}

	.models-list::-webkit-scrollbar {
		width: 6px;
	}

	.models-list::-webkit-scrollbar-track {
		background: transparent;
	}

	.models-list::-webkit-scrollbar-thumb {
		background: rgba(0, 0, 0, 0.15);
		border-radius: 3px;
	}

	.models-list::-webkit-scrollbar-thumb:hover {
		background: rgba(0, 0, 0, 0.25);
	}

	.model-item {
		padding: 0.625rem 1rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
		cursor: pointer;
		border-left: 3px solid transparent;
		position: relative;
		transition: all 0.2s ease;
		margin: 0 0.5rem;
		border-radius: 6px;
		color: #374151;
	}

	.model-item:hover {
		background: rgba(59, 130, 246, 0.08);
	}

	.model-item.active {
		border-left-color: #3b82f6;
		background: rgba(59, 130, 246, 0.1);
		box-shadow: 0 2px 4px rgba(59, 130, 246, 0.1);
	}

	.model-item.running {
		border-left-color: #10b981;
	}

	.delete-button {
		background: none;
		border: none;
		font-size: 1.25rem;
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		transition: all 0.2s ease;
		color: #9ca3af;
		border-radius: 4px;
		line-height: 1;
		opacity: 0;
	}

	.model-item:hover .delete-button {
		opacity: 1;
	}

	.delete-button:hover {
		background: rgba(239, 68, 68, 0.1);
		color: #ef4444;
	}

	.model-name {
		flex: 1;
		margin-right: 0.75rem;
		font-weight: 500;
		font-size: 0.875rem;
	}

	.model-date {
		margin-right: 0.75rem;
		font-size: 0.75rem;
		color: #9ca3af;
	}
</style>
