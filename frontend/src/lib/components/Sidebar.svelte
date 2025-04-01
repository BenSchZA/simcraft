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

<div class="sidebar bg-secondary border-accent/20 border-r" class:hidden={!$sidebarVisible}>
	<div class="sidebar-header">
		<h2 class="text-primary font-semibold">Models</h2>
		<button
			on:click={() => ($sidebarVisible = false)}
			class="close-button text-primary hover:text-accent">×</button
		>
	</div>

	<div class="node-palette">
		<input
			type="text"
			placeholder="Search nodes..."
			class="bg-primary border-accent/30 text-primary placeholder:text-secondary focus:border-accent mb-3 w-full rounded border px-2 py-1.5"
		/>
		<div class="node-types">
			{#each Object.entries(nodeTypes) as [processType, icon]}
				<div
					class="node bg-accent/20 border-accent/30 text-primary hover:bg-accent/30 flex items-center border"
					draggable={true}
					on:dragstart={(event) => onDragStart(event, processType as ProcessType)}
					role="button"
					tabindex="0"
				>
					<svelte:component this={icon} classes="w-5 h-5" />
					<span class="ml-2">{processType}</span>
				</div>
			{/each}
		</div>
	</div>

	<div class="search-bar">
		<input
			type="text"
			bind:value={searchTerm}
			placeholder="Search models..."
			class="bg-primary border-accent/30 text-primary placeholder:text-secondary focus:border-accent w-full rounded border px-2 py-1.5"
		/>
	</div>

	<button
		class="new-model-button bg-accent/20 text-primary hover:bg-accent/30 border-accent/30 border"
		on:click={createNewModel}
	>
		New Model
	</button>

	<div class="models-list">
		{#each filteredModels as model}
			<div
				class="model-item hover:bg-accent/10"
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
				<span class="model-name text-primary">{model.name}</span>
				<span class="model-date text-secondary text-sm">
					{new Date(model.lastModified).toLocaleDateString()}
				</span>
				<button
					class="delete-button text-secondary hover:text-accent"
					on:click|stopPropagation={() => deleteModel(model.id)}
				>
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
	}

	.hidden {
		transform: translateX(-100%);
	}

	.sidebar-header {
		padding: 1rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
		border-bottom: 1px solid;
	}

	.node-palette {
		padding: 1rem;
		border-bottom: 1px solid;
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

	.close-button {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		transition: color 0.3s ease;
	}

	.search-bar {
		padding: 1rem;
	}

	.search-bar input {
		width: 100%;
		padding: 0.5rem;
		border-radius: 4px;
		outline: none;
	}

	.new-model-button {
		margin: 0 1rem 1rem;
		padding: 0.5rem;
		border-radius: 4px;
		cursor: pointer;
		transition: all 0.3s ease;
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
		transition: all 0.3s ease;
	}

	.model-item.active {
		border-left-color: var(--accent);
		background-color: color-mix(in srgb, var(--accent) 10%, transparent);
	}

	.model-item.running {
		border-left-color: var(--green);
	}

	.delete-button {
		background: none;
		border: none;
		font-size: 1.2rem;
		cursor: pointer;
		padding: 0 0.5rem;
		transition: color 0.3s ease;
	}

	.model-name {
		flex: 1;
		margin-right: 1rem;
	}

	.model-date {
		margin-right: 1rem;
	}
</style>
