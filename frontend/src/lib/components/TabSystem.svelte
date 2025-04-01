<script lang="ts">
	import { activeModelId, openModels, runningStates } from '$lib/stores/simulation';
	import { onMount } from 'svelte';
	import { storageManager } from '$lib/storage/StorageManager';

	$: tabs = Array.from($openModels.values());
	$: activeTab = $activeModelId;

	let saveTimeout: number;

	// Save open tabs whenever they change
	$: {
		const tabIds = tabs.map((tab) => tab.id);
		// Clear any pending save
		if (saveTimeout) clearTimeout(saveTimeout);
		// Schedule a new save after a short delay
		saveTimeout = setTimeout(() => {
			storageManager.saveOpenTabs(tabIds, activeTab).catch((error) => {
				console.error('Failed to save open tabs:', error);
			});
		}, 200);
	}

	onMount(async () => {
		// Load saved tabs on mount
		const { tabs: savedTabs, activeTab: savedActiveTab } = await storageManager.loadOpenTabs();

		if (savedTabs.length > 0) {
			// Load each saved tab
			for (const tabId of savedTabs) {
				const model = await storageManager.loadModel(tabId);
				if (model) {
					openModels.update((models) => {
						models.set(tabId, model);
						return models;
					});
				}
			}

			// Restore active tab
			if (savedActiveTab && savedTabs.includes(savedActiveTab)) {
				$activeModelId = savedActiveTab;
			} else {
				$activeModelId = savedTabs[0];
			}
		}
	});

	function closeTab(id: string) {
		openModels.update((models) => {
			models.delete(id);
			return models;
		});

		if ($activeModelId === id) {
			// Set the active tab to the next available tab
			const remainingTabs = Array.from($openModels.values());
			if (remainingTabs.length > 0) {
				$activeModelId = remainingTabs[0].id;
			} else {
				$activeModelId = null;
			}
		}
	}

	function activateTab(id: string) {
		$activeModelId = id;
	}
</script>

<div class="tab-container bg-secondary border-accent/20 border-b">
	{#if tabs.length === 0}
		<div class="empty-state text-secondary">No open models</div>
	{:else}
		<div class="tabs">
			{#each tabs as tab}
				<div
					class="tab text-primary"
					class:active={tab.id === activeTab}
					class:running={$runningStates[tab.id]}
					on:click={() => activateTab(tab.id)}
					on:keydown={(e) => e.key === 'Enter' && activateTab(tab.id)}
					tabindex="0"
					role="tab"
				>
					<span class="tab-name">{tab.name}</span>
					<button
						class="close-tab text-secondary hover:text-accent"
						on:click|stopPropagation={() => closeTab(tab.id)}
					>
						×
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.tab-container {
		position: relative;
		z-index: 10;
	}

	.tabs {
		display: flex;
		overflow-x: auto;
		white-space: nowrap;
	}

	.tab {
		display: flex;
		align-items: center;
		padding: 0.5rem 1rem;
		cursor: pointer;
		user-select: none;
		min-width: 100px;
		max-width: 200px;
		transition: all 0.3s ease;
		border-bottom: 3px solid transparent;
	}

	.tab:hover {
		background-color: color-mix(in srgb, var(--accent) 5%, transparent);
	}

	.tab.active {
		background-color: color-mix(in srgb, var(--accent) 10%, transparent);
		border-bottom-color: var(--accent);
	}

	.tab.running {
		border-bottom-color: var(--green);
	}

	.tab-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.close-tab {
		background: none;
		border: none;
		margin-left: 0.5rem;
		padding: 0 0.25rem;
		cursor: pointer;
		font-size: 1.2rem;
		line-height: 1;
		transition: color 0.3s ease;
	}

	.empty-state {
		padding: 0.5rem 1rem;
		font-style: italic;
	}
</style>
