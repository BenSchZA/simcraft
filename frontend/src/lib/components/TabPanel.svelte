<script lang="ts">
	import {
		panelLayout,
		removeTabFromPanel,
		closePanel,
		type Panel,
		PanelType
	} from '$lib/stores/panelLayout';
	import { activeModelId, runningStates, openModels, sidebarVisible } from '$lib/stores/simulation';
	import FlowEditor from './FlowEditor.svelte';
	import SimulationChart from './SimulationChart.svelte';
	import EmptyState from './EmptyState.svelte';

	export let panel: Panel;

	// For model tabs, sync with activeModelId; for other tabs use panel's activeTabId
	$: activeTab = (() => {
		// Check if this panel has the active model
		const modelTab = panel.tabs.find(
			(t) => t.type === PanelType.Model && t.modelId === $activeModelId
		);
		if (modelTab) {
			// This panel has the active model, so highlight that tab
			return modelTab.id;
		}
		// Otherwise use the panel's own activeTabId
		return panel.activeTabId;
	})();

	function activateTab(tabId: string) {
		// Find the tab
		const tab = panel.tabs.find((t) => t.id === tabId);

		// If it's a model tab, update the global active model
		if (tab && tab.type === PanelType.Model && tab.modelId) {
			// Setting activeModelId will trigger the reactive statement to update activeTab
			$activeModelId = tab.modelId;
		} else {
			// For non-model tabs, update the panel's activeTabId
			panelLayout.update((layout) => {
				panel.activeTabId = tabId;
				return layout;
			});
		}
	}

	function closeTab(tabId: string) {
		// Find the tab to get its modelId
		const tab = panel.tabs.find((t) => t.id === tabId);

		// If it's a model tab, remove the model from openModels (which will trigger tab removal)
		if (tab && tab.type === PanelType.Model && tab.modelId) {
			openModels.update((models) => {
				models.delete(tab.modelId!);
				return models;
			});

			// If this was the active model, clear the active model
			if ($activeModelId === tab.modelId) {
				// Set to another open model if available
				const remainingModels = Array.from($openModels.keys());
				if (remainingModels.length > 0) {
					activeModelId.set(remainingModels[0]);
				} else {
					activeModelId.set(null);
				}
			}
		} else {
			// For non-model tabs (like charts), just remove the tab
			removeTabFromPanel(panel.id, tabId);
		}

		// If no tabs left, close the panel (but only for charts panel)
		if (panel.tabs.length <= 1 && panel.id === 'charts-panel') {
			closePanel(panel.id);
		}
	}

	// Context menu state
	let showContextMenu = false;
	let contextMenuX = 0;
	let contextMenuY = 0;
	let contextMenuTabId: string | null = null;

	function handleRightClick(event: MouseEvent, tabId: string) {
		event.preventDefault();
		contextMenuTabId = tabId;
		contextMenuX = event.clientX;
		contextMenuY = event.clientY;
		showContextMenu = true;
	}

	function closeContextMenu() {
		showContextMenu = false;
		contextMenuTabId = null;
	}

	// Close context menu when clicking outside
	function handleWindowClick() {
		if (showContextMenu) {
			closeContextMenu();
		}
	}
</script>

<svelte:window on:click={handleWindowClick} />

<div class="tab-panel">
	<div class="tab-header">
		{#if !$sidebarVisible}
			<button class="hamburger-button" on:click={() => ($sidebarVisible = true)} title="Show sidebar">
				<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor">
					<path d="M3 5h14M3 10h14M3 15h14" stroke-width="2" stroke-linecap="round" />
				</svg>
			</button>
		{/if}
		<div class="tabs">
			{#each panel.tabs as tab}
				<div
					class="tab"
					class:active={tab.id === activeTab}
					class:running={tab.type === PanelType.Model && tab.modelId && $runningStates[tab.modelId]}
					on:click={() => activateTab(tab.id)}
					on:contextmenu={(e) => handleRightClick(e, tab.id)}
					on:keydown={(e) => e.key === 'Enter' && activateTab(tab.id)}
					tabindex="0"
					role="tab"
				>
					<span class="tab-icon">
						{#if tab.type === PanelType.Model}
							<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
								<path
									d="M7 1L1 4v6l6 3 6-3V4L7 1zM2 4.5L7 2l5 2.5L7 7 2 4.5zM2 5.5L7 8l5-2.5V9L7 11.5 2 9V5.5z"
								/>
							</svg>
						{:else if tab.type === PanelType.Chart}
							<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
								<path d="M1 12h12v1H1v-1zm0-11h1v10H1V1zm3 7h2v3H4V8zm3-4h2v7H7V4zm3 2h2v5h-2V6z" />
							</svg>
						{/if}
					</span>
					<span class="tab-name">{tab.title}</span>
					{#if tab.closable}
						<button
							class="close-tab"
							on:click|stopPropagation={() => closeTab(tab.id)}
							aria-label="Close tab"
						>
							×
						</button>
					{/if}
				</div>
			{/each}
		</div>
	</div>

	<div class="tab-content">
		{#if panel.tabs.length === 0}
			<EmptyState />
		{:else if activeTab}
			{@const activeTabData = panel.tabs.find((t) => t.id === activeTab)}
			{#if activeTabData}
				{#if activeTabData.type === PanelType.Model}
					<div class="model-content">
						<FlowEditor />
					</div>
				{:else if activeTabData.type === PanelType.Chart}
					<div class="chart-content">
						<SimulationChart standalone={true} />
					</div>
				{/if}
			{/if}
		{/if}
	</div>
</div>

{#if showContextMenu && contextMenuTabId && panel.tabs.find((t) => t.id === contextMenuTabId)?.closable}
	<div
		class="context-menu"
		style="left: {contextMenuX}px; top: {contextMenuY}px;"
		on:click|stopPropagation
		on:keydown|stopPropagation
		role="menu"
		tabindex="-1"
	>
		<button
			class="context-menu-item"
			on:click={() => {
				if (contextMenuTabId) closeTab(contextMenuTabId);
				closeContextMenu();
			}}
		>
			Close Tab
		</button>
	</div>
{/if}

<style>
	.tab-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		width: 100%;
		background: #ffffff;
		overflow: hidden;
	}

	.tab-header {
		display: flex;
		align-items: center;
		background: linear-gradient(180deg, rgba(250, 250, 250, 0.95) 0%, rgba(245, 245, 245, 0.95) 100%);
		border-bottom: 1px solid rgba(0, 0, 0, 0.08);
		min-height: 40px;
		flex-shrink: 0;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
	}

	.hamburger-button {
		display: flex;
		align-items: center;
		justify-content: center;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 6px;
		padding: 0.5rem;
		margin: 0 0.5rem;
		cursor: pointer;
		color: #6b7280;
		transition: all 0.2s ease;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
		flex-shrink: 0;
	}

	.hamburger-button:hover {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
		transform: translateY(-1px);
	}

	.hamburger-button:active {
		transform: translateY(0);
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
	}

	.tabs {
		display: flex;
		flex: 1;
		overflow-x: auto;
		scrollbar-width: thin;
		gap: 2px;
		padding: 0 0.5rem 0 0;
	}

	.tabs::-webkit-scrollbar {
		height: 4px;
	}

	.tabs::-webkit-scrollbar-track {
		background: transparent;
	}

	.tabs::-webkit-scrollbar-thumb {
		background: rgba(0, 0, 0, 0.15);
		border-radius: 2px;
	}

	.tabs::-webkit-scrollbar-thumb:hover {
		background: rgba(0, 0, 0, 0.25);
	}

	.tab {
		display: flex;
		align-items: center;
		padding: 0 14px;
		height: 36px;
		cursor: pointer;
		user-select: none;
		white-space: nowrap;
		color: #6b7280;
		background: transparent;
		transition: all 0.2s ease;
		position: relative;
		border-radius: 6px 6px 0 0;
		margin-top: 4px;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.tab:hover {
		background: rgba(59, 130, 246, 0.08);
		color: #374151;
	}

	.tab.active {
		background: white;
		color: #111827;
		box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.05);
		border-bottom: 2px solid #3b82f6;
	}

	.tab.running {
		border-bottom-color: #10b981;
	}

	.tab.running::before {
		content: '';
		position: absolute;
		left: 8px;
		top: 50%;
		transform: translateY(-50%);
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #10b981;
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	.tab.running .tab-icon {
		margin-left: 10px;
	}

	.tab-icon {
		display: flex;
		align-items: center;
		margin-right: 6px;
		opacity: 0.6;
	}

	.tab.active .tab-icon {
		opacity: 1;
	}

	.tab-name {
		font-size: 0.875rem;
	}

	.close-tab {
		background: none;
		border: none;
		margin-left: 8px;
		padding: 4px;
		cursor: pointer;
		color: inherit;
		opacity: 0;
		transition: all 0.2s ease;
		font-size: 16px;
		line-height: 1;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
	}

	.tab:hover .close-tab,
	.tab.active .close-tab {
		opacity: 0.6;
	}

	.close-tab:hover {
		opacity: 1 !important;
		background: rgba(0, 0, 0, 0.1);
	}

	.tab-content {
		flex: 1;
		overflow: hidden;
		position: relative;
		background: white;
	}

	.model-content,
	.chart-content {
		width: 100%;
		height: 100%;
		position: absolute;
		top: 0;
		left: 0;
	}

	/* Context Menu */
	.context-menu {
		position: fixed;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
		padding: 0.25rem;
		z-index: 1000;
		min-width: 160px;
		backdrop-filter: blur(8px);
	}

	.context-menu-item {
		display: block;
		width: 100%;
		padding: 0.625rem 0.875rem;
		text-align: left;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		border-radius: 6px;
		transition: all 0.2s ease;
	}

	.context-menu-item:hover {
		background: rgba(59, 130, 246, 0.1);
		color: #3b82f6;
	}

	/* Dark theme support */
	:global(.dark) .tab-panel {
		background: #1a1a1a;
	}

	:global(.dark) .tab-header {
		background: linear-gradient(180deg, rgba(42, 42, 42, 0.95) 0%, rgba(32, 32, 32, 0.95) 100%);
		border-bottom-color: rgba(255, 255, 255, 0.1);
	}

	:global(.dark) .hamburger-button {
		background: #2a2a2a;
		border-color: rgba(255, 255, 255, 0.1);
		color: #9ca3af;
	}

	:global(.dark) .hamburger-button:hover {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	:global(.dark) .tab {
		color: #9ca3af;
	}

	:global(.dark) .tab:hover {
		background: rgba(59, 130, 246, 0.15);
		color: #e5e7eb;
	}

	:global(.dark) .tab.active {
		background: #1a1a1a;
		color: #ffffff;
	}

	:global(.dark) .tab-content {
		background: #1a1a1a;
	}

	:global(.dark) .context-menu {
		background: #2a2a2a;
		border-color: rgba(255, 255, 255, 0.1);
	}

	:global(.dark) .context-menu-item {
		color: #e5e7eb;
	}

	:global(.dark) .context-menu-item:hover {
		background: rgba(59, 130, 246, 0.2);
		color: #60a5fa;
	}
</style>
