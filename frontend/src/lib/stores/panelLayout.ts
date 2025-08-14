import { writable, derived } from 'svelte/store';
import { v4 as uuidv4 } from 'uuid';

export enum PanelType {
	Model = 'model',
	Chart = 'chart'
}

export interface PanelTab {
	id: string;
	title: string;
	type: PanelType;
	modelId?: string; // For model-specific content
	closable: boolean;
}

export interface Panel {
	id: string;
	tabs: PanelTab[];
	activeTabId: string | null;
	size: number; // Percentage of parent container
}

export interface PanelGroup {
	id: string;
	direction: 'horizontal' | 'vertical';
	panels: PanelLayout[];
	size: number; // Percentage of parent container
}

export type PanelLayout = Panel | PanelGroup;

// Default layout - single panel with model tabs
const defaultLayout: PanelLayout = {
	id: 'root',
	tabs: [],
	activeTabId: null,
	size: 100
};

// Store for the panel layout
export const panelLayout = writable<PanelLayout>(defaultLayout);

// Store for tracking which panels have which content
export const panelContents = writable<Map<string, PanelTab[]>>(new Map());

// Helper functions
export function isPanelGroup(layout: PanelLayout): layout is PanelGroup {
	return 'panels' in layout;
}

export function isPanel(layout: PanelLayout): layout is Panel {
	return 'tabs' in layout;
}

// Add a new tab to a panel
export function addTabToPanel(panelId: string, tab: PanelTab) {
	panelLayout.update((layout) => {
		const panel = findPanel(layout, panelId);
		if (panel && isPanel(panel)) {
			panel.tabs.push(tab);
			panel.activeTabId = tab.id;
		}
		return layout;
	});
}

// Remove a tab from a panel
export function removeTabFromPanel(panelId: string, tabId: string) {
	panelLayout.update((layout) => {
		const panel = findPanel(layout, panelId);
		if (panel && isPanel(panel)) {
			panel.tabs = panel.tabs.filter((t) => t.id !== tabId);
			if (panel.activeTabId === tabId) {
				panel.activeTabId = panel.tabs.length > 0 ? panel.tabs[0].id : null;
			}
		}
		return layout;
	});
}

// Find a panel by ID
export function findPanel(layout: PanelLayout, panelId: string): PanelLayout | null {
	if (layout.id === panelId) {
		return layout;
	}

	if (isPanelGroup(layout)) {
		for (const panel of layout.panels) {
			const found = findPanel(panel, panelId);
			if (found) return found;
		}
	}

	return null;
}

// Get all panels (flattened)
export function getAllPanels(layout: PanelLayout): Panel[] {
	if (isPanel(layout)) {
		return [layout];
	}

	const panels: Panel[] = [];
	for (const panel of layout.panels) {
		panels.push(...getAllPanels(panel));
	}
	return panels;
}

// Derived store for active tabs across all panels
export const activeTabs = derived(panelLayout, ($layout) => {
	const panels = getAllPanels($layout);
	const active: { panelId: string; tab: PanelTab }[] = [];

	for (const panel of panels) {
		const activeTab = panel.tabs.find((t) => t.id === panel.activeTabId);
		if (activeTab) {
			active.push({ panelId: panel.id, tab: activeTab });
		}
	}

	return active;
});

// Open or focus a chart for a specific model
export function openChartForModel(modelId: string, modelName: string = 'Chart') {
	panelLayout.update((layout) => {
		// If layout is a single panel, convert to left/right split
		if (isPanel(layout)) {
			const chartsPanel: Panel = {
				id: 'charts-panel',
				tabs: [],
				activeTabId: null,
				size: 40
			};

			// Convert single panel to left panel
			layout.size = 60;
			layout.id = 'models-panel';

			// Create new chart tab
			const chartTab: PanelTab = {
				id: uuidv4(),
				title: `${modelName} Chart`,
				type: PanelType.Chart,
				modelId: modelId,
				closable: true
			};

			chartsPanel.tabs.push(chartTab);
			chartsPanel.activeTabId = chartTab.id;

			// Return new split layout
			return {
				id: 'root',
				direction: 'horizontal',
				panels: [layout, chartsPanel],
				size: 100
			} as PanelGroup;
		}

		// Layout is already split, work with existing panels
		const panels = getAllPanels(layout);

		// Check if a chart for this model already exists
		for (const panel of panels) {
			const existingChart = panel.tabs.find(
				(t) => t.type === PanelType.Chart && t.modelId === modelId
			);

			if (existingChart) {
				// Focus the existing chart
				panel.activeTabId = existingChart.id;
				return layout;
			}
		}

		// Create a new chart tab
		const chartTab: PanelTab = {
			id: uuidv4(),
			title: `${modelName} Chart`,
			type: PanelType.Chart,
			modelId: modelId,
			closable: true
		};

		// Find the charts panel (right panel) or create one if needed
		let chartsPanel = panels.find((p) => p.id === 'charts-panel');

		if (!chartsPanel) {
			// No charts panel exists, find the rightmost panel
			chartsPanel = panels[panels.length - 1];
		}

		if (chartsPanel) {
			chartsPanel.tabs.push(chartTab);
			chartsPanel.activeTabId = chartTab.id;
		}

		return layout;
	});
}

// Resize panels in a group
export function resizePanels(groupId: string, sizes: number[]) {
	panelLayout.update((layout) => {
		const group = findPanel(layout, groupId);
		if (group && isPanelGroup(group) && sizes.length === group.panels.length) {
			group.panels.forEach((panel, index) => {
				panel.size = sizes[index];
			});
		}
		return layout;
	});
}

// Close a panel (remove it from the layout)
export function closePanel(panelId: string) {
	panelLayout.update((layout) => {
		return closePanelRecursive(layout, panelId) || layout;
	});
}

function closePanelRecursive(layout: PanelLayout, targetId: string): PanelLayout | null {
	if (isPanelGroup(layout)) {
		const filteredPanels = layout.panels.filter((p) => p.id !== targetId);

		if (filteredPanels.length !== layout.panels.length) {
			// Panel was found and removed
			if (filteredPanels.length === 1) {
				// If only one panel left, return it directly
				const remainingPanel = filteredPanels[0];
				remainingPanel.size = 100; // Reset size to full
				return remainingPanel;
			}

			// Redistribute sizes
			const totalSize = 100;
			const newSize = totalSize / filteredPanels.length;
			filteredPanels.forEach((panel) => {
				panel.size = newSize;
			});

			layout.panels = filteredPanels;
			return layout;
		}

		// Try to find and close in nested groups
		for (let i = 0; i < layout.panels.length; i++) {
			const result = closePanelRecursive(layout.panels[i], targetId);
			if (result) {
				layout.panels[i] = result;
				return layout;
			}
		}
	}

	return null;
}
