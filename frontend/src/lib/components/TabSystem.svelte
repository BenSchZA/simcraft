<script lang="ts">
	import { activeModelId, openModels, runningStates } from '$lib/stores/simulation';

	$: tabs = Array.from($openModels.values());
	$: activeTab = $activeModelId;

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

<div class="tab-container">
	{#if tabs.length === 0}
		<div class="empty-state">No open simulations</div>
	{:else}
		<div class="tabs">
			{#each tabs as tab}
				<div
					class="tab"
					class:active={tab.id === activeTab}
					class:running={$runningStates[tab.id]}
					on:click={() => activateTab(tab.id)}
					on:keydown={(e) => e.key === 'Enter' && activateTab(tab.id)}
					tabindex="0"
					role="tab"
				>
					<span class="tab-name">{tab.name}</span>
					<button class="close-tab" on:click|stopPropagation={() => closeTab(tab.id)}> × </button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.tab-container {
		background-color: #1e1e1e;
		border-bottom: 1px solid #333;
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
		background-color: #2d2d2d;
		border-right: 1px solid #333;
		color: #ccc;
		cursor: pointer;
		user-select: none;
		min-width: 100px;
		max-width: 200px;
	}

	.tab:hover {
		background-color: #333;
	}

	.tab.active {
		background-color: #1e1e1e;
		color: #fff;
	}

	.tab.running {
		border-bottom: 3px solid #2ea44f;
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
		color: #666;
		margin-left: 0.5rem;
		padding: 0 0.25rem;
		cursor: pointer;
	}

	.close-tab:hover {
		color: #ff4444;
	}

	.empty-state {
		padding: 0.5rem 1rem;
		color: #666;
		font-style: italic;
	}
</style>
