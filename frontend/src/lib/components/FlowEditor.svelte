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
		type Connection,
		MarkerType,
		ConnectionMode,
		useSvelteFlow,
		type NodeTypes
	} from '@xyflow/svelte';
	import { writable } from 'svelte/store';
	import { activeModelId, openModels } from '$lib/stores/simulation';
	import { storageManager } from '$lib/storage/StorageManager';
	import { v4 as uuidv4 } from 'uuid';
	import SourceNode from './nodes/SourceNode.svelte';
	import PoolNode from './nodes/PoolNode.svelte';
	import { useDnD } from '$lib/utils/dnd';
	import SimulationControls from './SimulationControls.svelte';
	import SimulationChart from './SimulationChart.svelte';

	const nodes = writable<Node[]>([]);
	const edges = writable<Edge[]>([]);
	const { screenToFlowPosition } = useSvelteFlow();
	const type = useDnD();

	$: if ($activeModelId) {
		const model = $openModels.get($activeModelId);
		if (model) {
			nodes.set(model.nodes);
			edges.set(model.edges);
		}
	}

	const nodeTypes: NodeTypes | undefined = {
		source: SourceNode,
		pool: PoolNode
	};

	function onConnect(connection: Connection) {
		const { source, target, sourceHandle, targetHandle } = connection;

		const newEdge: Edge = {
			id: `e${uuidv4()}`,
			source,
			target,
			sourceHandle,
			targetHandle,
			type: 'default',
			markerEnd: {
				type: MarkerType.Arrow,
				width: 20,
				height: 20
			},
			data: { flowRate: 1.0 }
		};

		edges.update((es) => [...es, newEdge]);
		saveFlowState();
	}

	function onDelete(params: { nodes: Node[]; edges: Edge[] }) {
		const nodesToDelete = params.nodes;
		nodes.update((ns) => ns.filter((n) => !nodesToDelete.some((nd: Node) => nd.id === n.id)));

		const edgesToDelete = params.edges;
		edges.update((es) => es.filter((e) => !edgesToDelete.some((ed: Edge) => ed.id === e.id)));

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

	const onDrop = (event: DragEvent) => {
		event.preventDefault();

		if (!$type) {
			return;
		}

		const position = screenToFlowPosition({
			x: event.clientX,
			y: event.clientY
		});

		const newNode = {
			id: `${$type}-${uuidv4()}`,
			type: $type,
			position,
			data: { label: `${$type}` },
			draggable: true,
			selectable: true,
			deletable: true,
			selected: false,
			dragging: false,
			zIndex: 0
		};

		nodes.update((ns) => [...ns, newNode]);
		saveFlowState();
	};
</script>

<div class="flow-container">
	{#if $activeModelId}
		<SvelteFlow
			{nodes}
			{edges}
			{nodeTypes}
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
			<Panel position="top-right" class="chart-panel">
				<SimulationChart />
			</Panel>
			<Panel position="bottom-left" class="controls-panel">
				<SimulationControls />
			</Panel>
		</SvelteFlow>
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
		border-radius: 8px !important;
		margin: 1rem !important;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	:global(.chart-panel) {
		background-color: rgba(255, 255, 255, 0.9) !important;
		backdrop-filter: blur(4px);
		border: 1px solid #e0e0e0 !important;
		border-radius: 8px !important;
		margin: 1rem !important;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		width: 400px;
	}
</style>
