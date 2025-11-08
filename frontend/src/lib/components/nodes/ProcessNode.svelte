<script lang="ts">
	import { Handle, Position, type NodeProps } from '@xyflow/svelte';
	import NodeLabel from './NodeLabel.svelte';
	import PoolNodeIcon from './node-icons/PoolNodeIcon.svelte';
	import { ProcessType } from '$lib';
	import SourceNodeIcon from './node-icons/SourceNodeIcon.svelte';
	import DelayNodeIcon from './node-icons/DelayNodeIcon.svelte';
	import DrainNodeIcon from './node-icons/DrainNodeIcon.svelte';
	import { activeNodeId } from '$lib/stores/simulation';

	type $$Props = NodeProps;

	export let id: $$Props['id'];
	export let type: ProcessType;
	export let data: Record<string, any>;
	export let selected: $$Props['selected'] = false;
	export let dragging = false;
	export let isConnectable = true;

	let labelPosition = data.labelPosition || { x: 0, y: 40 };

	function onLabelPositionChange(newPosition: { x: number; y: number }) {
		labelPosition = newPosition;
		data.labelPosition = newPosition;
	}

	$: if (selected) {
		activeNodeId.set(id);
	} else {
		if ($activeNodeId === id) {
			activeNodeId.set(null);
		}
	}
</script>

<div class="node" class:selected class:dragging>
	<div class="node-shape">
		{#if type === ProcessType.Pool}
			<Handle id="in" type="source" position={Position.Left} {isConnectable} />
			<Handle id="out" type="source" position={Position.Right} {isConnectable} />
			<PoolNodeIcon />
		{:else if type === ProcessType.Source}
			<Handle id="out" type="source" position={Position.Right} {isConnectable} />
			<SourceNodeIcon />
		{:else if type === ProcessType.Drain}
			<Handle id="in" type="source" position={Position.Left} {isConnectable} />
			<DrainNodeIcon />
		{:else if type === ProcessType.Delay}
			<Handle id="in" type="source" position={Position.Left} {isConnectable} />
			<Handle id="out" type="source" position={Position.Right} {isConnectable} />
			<DelayNodeIcon />
		{:else}
			<div></div>
		{/if}
	</div>
	<NodeLabel
		label={data.label}
		bind:position={labelPosition}
		onPositionChange={onLabelPositionChange}
	/>
</div>

<style>
	.node {
		position: relative;
		width: 48px;
		height: 48px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.node-shape {
		width: 42px;
		height: 42px;
	}

	.selected .node-shape {
		color: #3b82f6;
	}

	.dragging {
		opacity: 0.8;
	}

	:global(.svelte-flow .svelte-flow__node .svelte-flow__handle) {
		width: 8px;
		height: 8px;
	}
	
	:global(.svelte-flow .svelte-flow__handle.connectingto) {
		background: #ff6060;
	}
	
	:global(.svelte-flow .svelte-flow__handle.valid) {
		background: #55dd99;
	}
</style>
