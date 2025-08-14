<script lang="ts">
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import PanelLayout from '$lib/components/PanelLayout.svelte';
	import { sidebarVisible, openModels, activeModelId } from '$lib/stores/simulation';
	import {
		panelLayout,
		addTabToPanel,
		removeTabFromPanel,
		PanelType,
		type PanelTab,
		getAllPanels,
		isPanelGroup
	} from '$lib/stores/panelLayout';
	import { storageManager } from '$lib/storage/StorageManager';
	import { SvelteFlowProvider } from '@xyflow/svelte';
	import DnDProvider from '$lib/components/DnDProvider.svelte';
	import { v4 as uuidv4 } from 'uuid';

	onMount(async () => {
		await storageManager.init();

		// Initialize panel layout with open models
		openModels.subscribe((models) => {
			const currentLayout = $panelLayout;
			const panels = getAllPanels(currentLayout);
			const openModelIds = new Set(Array.from(models.keys()));

			// First, remove tabs for models that are no longer open
			for (const panel of panels) {
				const modelTabs = panel.tabs.filter((t) => t.type === PanelType.Model);
				for (const tab of modelTabs) {
					if (tab.modelId && !openModelIds.has(tab.modelId)) {
						removeTabFromPanel(panel.id, tab.id);
					}
				}
			}

			// Then, add tabs for newly opened models
			for (const model of models.values()) {
				// Check if tab already exists
				let tabExists = false;

				for (const panel of panels) {
					if (panel.tabs.some((t) => t.modelId === model.id)) {
						tabExists = true;
						break;
					}
				}

				if (!tabExists) {
					const tab: PanelTab = {
						id: uuidv4(),
						title: model.name,
						type: PanelType.Model,
						modelId: model.id,
						closable: true
					};

					// Add to models panel (left panel) if it exists, otherwise to root
					if (isPanelGroup(currentLayout)) {
						// Layout is split, add to left panel (models-panel)
						const modelsPanel = panels.find((p) => p.id === 'models-panel') || panels[0];
						if (modelsPanel) {
							addTabToPanel(modelsPanel.id, tab);
							// If this model is the active one, make its tab active
							if ($activeModelId === model.id) {
								panelLayout.update((layout) => {
									if (modelsPanel) {
										modelsPanel.activeTabId = tab.id;
									}
									return layout;
								});
							}
						}
					} else {
						// Single panel, add to it
						addTabToPanel(currentLayout.id, tab);
						// If this model is the active one, make its tab active
						if ($activeModelId === model.id) {
							panelLayout.update((layout) => {
								if ('tabs' in layout) {
									layout.activeTabId = tab.id;
								}
								return layout;
							});
						}
					}
				}
			}
		});
	});
</script>

<div class="layout bg-primary">
	<SvelteFlowProvider>
		<DnDProvider>
			<Sidebar />
			<div class="main-content" class:sidebar-hidden={!$sidebarVisible}>
				<div class="content-wrapper">
					<PanelLayout layout={$panelLayout} />
				</div>
				{#if !$sidebarVisible}
					<button
						class="show-sidebar-button bg-secondary text-primary hover:bg-accent/20 border-accent/30 border"
						on:click={() => ($sidebarVisible = true)}
					>
						☰
					</button>
				{/if}
			</div>
		</DnDProvider>
	</SvelteFlowProvider>
</div>

<style>
	.layout {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.main-content {
		flex: 1;
		display: flex;
		position: relative;
		margin-left: 0;
		transition: margin-left 0.3s ease;
	}

	.main-content.sidebar-hidden {
		margin-left: -250px;
	}

	.content-wrapper {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.show-sidebar-button {
		position: absolute;
		top: 10px;
		left: 10px;
		z-index: 1000;
		border-radius: 4px;
		padding: 8px;
		cursor: pointer;
		transition: all 0.3s ease;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		font-family:
			-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
			'Helvetica Neue', sans-serif;
	}
</style>
