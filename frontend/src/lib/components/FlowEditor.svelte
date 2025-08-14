<script lang="ts">
	import '@xyflow/svelte/dist/style.css';
	import {
		SvelteFlow,
		Background,
		Controls,
		Panel,
		MiniMap,
		type Node as SvelteFlowNode,
		type Edge as SvelteFlowEdge,
		type Connection as FlowConnection,
		MarkerType,
		ConnectionMode,
		useSvelteFlow,
		type NodeTypes as SvelteFlowilegesNodeTypes
	} from '@xyflow/svelte';
	import { writable } from 'svelte/store';
	import { activeModelId, openModels } from '$lib/stores/simulation';
	import { storageManager } from '$lib/storage/StorageManager';
	import { theme } from '$lib/stores/theme';
	import { v4 as uuidv4 } from 'uuid';
	import ProcessNodeComponent from './nodes/ProcessNode.svelte';
	import { useDnD } from '$lib/utils/dnd';
	import SimulationControls from './SimulationControls.svelte';
	import NodeTypesPanel from './NodeTypesPanel.svelte';
	import EmptyState from './EmptyState.svelte';
	import {
		addSimulationProcess,
		removeSimulationProcess,
		addSimulationConnection,
		removeSimulationConnection
	} from '$lib/stores/simulationManager';
	import {
		ProcessType,
		type ProcessNode as SimProcessNode,
		type ConnectionEdge as SimConnectionEdge,
		type ConnectionSettings
	} from '$lib/simcraft/base';
	import ConfigurationMenu from './ConfigurationMenu.svelte';
	import { selectedElement, configPanelVisible } from '$lib/stores/viewStates';

	const nodes = writable<SimProcessNode[]>([]);
	const edges = writable<SimConnectionEdge[]>([]);
	const { screenToFlowPosition } = useSvelteFlow();
	const nodeType = useDnD();

	// Update nodes and edges when activeModelId or openModels change
	$: {
		if ($activeModelId) {
			const model = $openModels.get($activeModelId);
			if (model) {
				nodes.set(model.nodes || []);
				edges.set(model.edges || []);
			} else {
				nodes.set([]);
				edges.set([]);
			}
		} else {
			nodes.set([]);
			edges.set([]);
		}
	}

	const nodeTypes: SvelteFlowilegesNodeTypes | undefined = {
		[ProcessType.Source]: ProcessNodeComponent,
		[ProcessType.Pool]: ProcessNodeComponent,
		[ProcessType.Drain]: ProcessNodeComponent,
		[ProcessType.Delay]: ProcessNodeComponent
	};

	async function onConnect(connection: FlowConnection) {
		// NOTE At this point, the Connection is already created,
		// whereas with ReactFlow, onconnect is called before the Connection is created.
		// We don't have access to the Edge id here, and so can't revert the Connection if it fails.
		const { source, target, sourceHandle, targetHandle } = connection;

		if (!source || !target || !sourceHandle || !targetHandle) {
			console.error('Missing source or target or handles in connection');
			return;
		}

		await addSimulationConnection(
			// TODO Try make connection management more robust
			`xy-edge__${source}${sourceHandle}-${target}${targetHandle}`,
			source,
			target,
			sourceHandle,
			targetHandle,
			{
				sourcePort: 'out',
				targetPort: 'in',
				flowRate: 1.0
			}
		)
			.then(() => {
				saveFlowState();
			})
			.catch((error) => {
				console.error('Failed to add connection to simulation:', error);
			});
	}

	async function onDelete(params: { nodes: any[]; edges: any[] }) {
		const nodesToDelete = params.nodes;
		const edgesToDelete = params.edges;

		// Remove edges first
		const deleteEdgePromises = edgesToDelete.map(async (edge) => {
			try {
				await removeSimulationConnection(edge.id);
				edges.update((es) => es.filter((e) => e.id !== edge.id));
			} catch (err) {
				console.error(`Failed to remove edge ${edge.id}:`, err);
			}
		});

		await Promise.all(deleteEdgePromises);

		// Then remove nodes
		const deleteNodePromises = nodesToDelete.map(async (node) => {
			try {
				await removeSimulationProcess(node.id);
				nodes.update((ns) => ns.filter((n) => n.id !== node.id));
			} catch (err) {
				console.error(`Failed to remove node ${node.id}:`, err);
			}
		});

		await Promise.all(deleteNodePromises);

		saveFlowState();
	}

	async function saveFlowState() {
		if (!$activeModelId) return;

		const model = $openModels.get($activeModelId);
		if (!model) return;

		const updatedModel = {
			...model,
			nodes: $nodes,
			edges: $edges,
			lastModified: Date.now()
		};

		await storageManager.saveModel(updatedModel);
		openModels.update((models) => {
			models.set($activeModelId!, updatedModel);
			return models;
		});
	}

	const onDragOver = (event: DragEvent) => {
		event.preventDefault();
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'move';
		}
	};

	export const addNode = async (processType: ProcessType, position: { x: number; y: number }) => {
		const nodeId = `${processType}-${uuidv4()}`;

		await addSimulationProcess(processType, nodeId, position)
			.then(() => {
				saveFlowState();
			})
			.catch((error) => {
				console.error('Failed to add process to simulation:', error);
			});
	};

	function onNodeClick(processType: ProcessType) {
		const position = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
		addNode(processType, position);
	}

	const onDrop = (event: DragEvent) => {
		event.preventDefault();

		if (!$nodeType) {
			return;
		}

		const position = screenToFlowPosition({
			x: event.clientX,
			y: event.clientY
		});

		addNode($nodeType, position);
	};

	// Event handlers for selection
	const handleNodeSelect = (event: CustomEvent<{ node: SvelteFlowNode }>) => {
		const clickedSvelteFlowNode = event.detail.node;
		const fullNode = $nodes.find((n) => n.id === clickedSvelteFlowNode.id);
		if (fullNode) {
			selectedElement.set(fullNode);
		} else {
			console.warn('Clicked node not found in local store:', clickedSvelteFlowNode.id);
			selectedElement.set(null); // Should not happen if stores are in sync
		}
	};

	const handleNodeDoubleClick = (event: CustomEvent<{ node: SvelteFlowNode }>) => {
		const clickedSvelteFlowNode = event.detail.node;
		const fullNode = $nodes.find((n) => n.id === clickedSvelteFlowNode.id);
		if (fullNode) {
			selectedElement.set(fullNode);
			configPanelVisible.set(true);
		}
	};

	const handleEdgeSelect = (event: CustomEvent<{ edge: SvelteFlowEdge }>) => {
		const clickedSvelteFlowEdge = event.detail.edge;
		const fullEdge = $edges.find((e) => e.id === clickedSvelteFlowEdge.id);
		if (fullEdge) {
			selectedElement.set(fullEdge);
		} else {
			console.warn('Clicked edge not found in local store:', clickedSvelteFlowEdge.id);
			selectedElement.set(null); // Should not happen
		}
	};

	const handlePaneClick = () => {
		selectedElement.set(null);
		// configPanelVisible.set(false); // User requested this to be a toggle, so pane click shouldn't hide it
	};
</script>

<div class="flow-container">
	{#if $activeModelId}
		{#key $activeModelId}
			<SvelteFlow
				{nodes}
				{edges}
				{nodeTypes}
				colorMode={$theme}
				fitView
				onconnect={onConnect}
				ondelete={onDelete}
				on:dragover={onDragOver}
				on:drop={onDrop}
				on:nodeclick={handleNodeSelect}
				on:nodedoubleclick={handleNodeDoubleClick}
				on:edgeclick={handleEdgeSelect}
				on:paneclick={handlePaneClick}
				connectionMode={ConnectionMode.Loose}
				defaultEdgeOptions={{
					type: 'default',
					markerEnd: {
						type: MarkerType.ArrowClosed
					}
				}}
			>
				<Background />
				<Controls position="bottom-right" />
				<!-- <MiniMap /> -->
				<Panel position="top-left" class="node-types-panel">
					<NodeTypesPanel {onNodeClick} />
				</Panel>
				<Panel position="bottom-center" class="controls-panel">
					<SimulationControls />
				</Panel>
			</SvelteFlow>
		{/key}

		<!-- Configuration Drawer -->
		<div class="config-drawer {$configPanelVisible ? 'open' : ''}">
			<div class="drawer-header">
				<h3>Configuration</h3>
				<button
					class="close-button"
					on:click={() => configPanelVisible.set(false)}
					aria-label="Close configuration panel"
				>
					<svg width="20" height="20" viewBox="0 0 20 20" fill="none">
						<path
							d="M15 5L5 15M5 5L15 15"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
						/>
					</svg>
				</button>
			</div>
			<div class="drawer-content">
				{#if $selectedElement}
					<ConfigurationMenu selectedElement={$selectedElement} onSettingsChange={saveFlowState} />
				{:else}
					<div class="no-selection">
						<p>Select a node or connection to configure its settings</p>
					</div>
				{/if}
			</div>
		</div>
	{:else}
		<EmptyState />
	{/if}
</div>
<!-- Configuration Toggle -->
<button
	class="config-toggle-chevron {$selectedElement ? 'has-selection' : ''}"
	on:click={() => configPanelVisible.update((v: boolean) => !v)}
	title={$selectedElement ? 'Configure Selected Element' : 'Select an element to configure'}
	aria-label={$selectedElement ? 'Configure Selected Element' : 'Select an element to configure'}
	disabled={!$selectedElement}
>
	<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
		<path
			d="M10 4L6 8L10 12"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		/>
	</svg>
</button>

<style>
	.flow-container {
		width: 100%;
		height: 100%;
		background-color: #ffffff;
		position: relative;
	}

	:global(.controls-panel) {
		background-color: rgba(255, 255, 255, 0.9) !important;
		backdrop-filter: blur(4px);
		border: 1px solid #e0e0e0 !important;
		margin: 1rem !important;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	/* Configuration Drawer Styles */

	.config-drawer {
		position: fixed;
		top: 0;
		right: 0;
		width: 400px;
		height: 100vh;
		background: #ffffff;
		border-left: 1px solid #e5e7eb;
		box-shadow: -4px 0 15px rgba(0, 0, 0, 0.1);
		transform: translateX(100%);
		transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		z-index: 1000;
		display: flex;
		flex-direction: column;
	}

	.config-drawer.open {
		transform: translateX(0);
	}

	.drawer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.5rem 1.5rem 1rem 1.5rem;
		border-bottom: 1px solid #f3f4f6;
		background: #fafafa;
	}

	.drawer-header h3 {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
	}

	.close-button {
		background: none;
		border: none;
		padding: 0.5rem;
		cursor: pointer;
		color: #6b7280;
		border-radius: 0.375rem;
		transition: all 0.2s;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.close-button:hover {
		background-color: #f3f4f6;
		color: #374151;
	}

	.drawer-content {
		flex: 1;
		overflow-y: auto;
		padding: 1.5rem;
	}

	.no-selection {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 200px;
		color: #6b7280;
		text-align: center;
	}

	/* Configuration Toggle */
	.config-toggle-chevron {
		position: fixed;
		top: 50%;
		right: 0;
		transform: translateY(-50%);
		background: #ffffff;
		border: 1px solid #e5e7eb;
		border-right: none;
		border-radius: 0.5rem 0 0 0.5rem;
		width: 32px;
		height: 48px;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		box-shadow: -2px 0 4px rgba(0, 0, 0, 0.1);
		z-index: 900;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
		color: #6b7280;
	}

	.config-toggle-chevron:hover:not(:disabled) {
		background: #f9fafb;
		color: #374151;
		transform: translateY(-50%) translateX(-2px);
		box-shadow: -4px 0 8px rgba(0, 0, 0, 0.15);
	}

	.config-toggle-chevron:disabled {
		opacity: 0.4;
		cursor: not-allowed;
		color: #d1d5db;
	}

	/* Highlight when element is selected */
	.config-toggle-chevron.has-selection {
		background: #f0f9ff;
		border-color: #3b82f6;
		color: #3b82f6;
	}
</style>
