<script lang="ts">
	import '@xyflow/svelte/dist/style.css';
	import {
		SvelteFlow,
		Background,
		Controls,
		Panel,
		MiniMap,
		type Node,
		type Edge,
		type Connection as FlowConnection,
		MarkerType,
		ConnectionMode,
		useSvelteFlow,
		type NodeTypes
	} from '@xyflow/svelte';
	import { writable } from 'svelte/store';
	import { activeModelId, openModels } from '$lib/stores/simulation';
	import { storageManager } from '$lib/storage/StorageManager';
	import { theme } from '$lib/stores/theme';
	import { v4 as uuidv4 } from 'uuid';
	import ProcessNode from './nodes/ProcessNode.svelte';
	import { useDnD } from '$lib/utils/dnd';
	import SimulationControls from './SimulationControls.svelte';
	import SimulationChart from './SimulationChart.svelte';
	import NodeTypesPanel from './NodeTypesPanel.svelte';
	import EmptyState from './EmptyState.svelte';
	import {
		addSimulationProcess,
		removeSimulationProcess,
		addSimulationConnection,
		removeSimulationConnection
	} from '$lib/stores/simulationManager';
	import { ProcessType, type Connection } from '$lib/simcraft';

	const nodes = writable<Node[]>([]);
	const edges = writable<Edge[]>([]);
	const { screenToFlowPosition } = useSvelteFlow();
	const nodeType = useDnD();

	$: if ($activeModelId) {
		const model = $openModels.get($activeModelId);
		if (model) {
			nodes.set(model.nodes);
			edges.set(model.edges);
		}
	}

	const nodeTypes: NodeTypes | undefined = {
		[ProcessType.Source]: ProcessNode,
		[ProcessType.Pool]: ProcessNode,
		[ProcessType.Drain]: ProcessNode,
		[ProcessType.Delay]: ProcessNode
	};

	async function onConnect(connection: FlowConnection) {
		// NOTE At this point, the Connection is already created,
		// whereas with ReactFlow, onconnect is called before the Connection is created.
		// We don't have access to the Edge id here, and so can't revert the Connection if it fails.
		const { source, target, sourceHandle, targetHandle } = connection;

		const newConnection: Connection = {
			// TODO Try make connection management more robust
			id: `xy-edge__${source}${sourceHandle}-${target}${targetHandle}`,
			sourceID: source,
			sourcePort: 'out',
			targetID: target,
			targetPort: 'in',
			flowRate: 1.0
		};

		await addSimulationConnection(newConnection)
			.then(() => {
				saveFlowState();
			})
			.catch((error) => {
				console.error('Failed to add connection to simulation:', error);
			});
	}

	async function onDelete(params: { nodes: Node[]; edges: Edge[] }) {
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
		const newNode = {
			id: `${processType}-${uuidv4()}`,
			type: processType,
			position,
			data: { label: `${processType}` },
			draggable: true,
			selectable: true,
			deletable: true,
			selected: false,
			dragging: false
		};

		await addSimulationProcess(processType, newNode.id)
			.then(() => {
				nodes.update((ns) => [...ns, newNode]);
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
</script>

<div class="flow-container">
	{#if $activeModelId}
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
			connectionMode={ConnectionMode.Loose}
			defaultEdgeOptions={{
				type: 'default',
				markerEnd: {
					type: MarkerType.Arrow
				}
			}}
		>
			<Background />
			<Controls position="top-left" />
			<MiniMap />
			<Panel position="top-center" class="node-types-panel">
				<NodeTypesPanel {onNodeClick} />
			</Panel>
			<Panel position="top-right" class="chart-panel">
				<SimulationChart />
			</Panel>
			<Panel position="bottom-left" class="controls-panel">
				<SimulationControls />
			</Panel>
		</SvelteFlow>
	{:else}
		<EmptyState />
	{/if}
</div>

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

	:global(.chart-panel) {
		background-color: rgba(255, 255, 255, 0.9) !important;
		backdrop-filter: blur(4px);
		border: 1px solid #e0e0e0 !important;
		margin: 1rem !important;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}
</style>
