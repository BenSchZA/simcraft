<script lang="ts">
	import { Handle, Position, type NodeProps } from '@xyflow/svelte';
	import NodeLabel from './NodeLabel.svelte';
	import PoolNodeIcon from './node-icons/PoolNodeIcon.svelte';
	import { ProcessType } from '$lib';
	import SourceNodeIcon from './node-icons/SourceNodeIcon.svelte';
	import DelayNodeIcon from './node-icons/DelayNodeIcon.svelte';
	import DrainNodeIcon from './node-icons/DrainNodeIcon.svelte';

	type $$Props = NodeProps;

	export let type: ProcessType;
	export let data: Record<string, any>;
	export let selected = false;
	export let dragging = false;
	export let isConnectable = true;

	let labelPosition = data.labelPosition || { x: 0, y: 40 };

	function onLabelPositionChange(newPosition: { x: number; y: number }) {
		labelPosition = newPosition;
		data.labelPosition = newPosition;
	}
</script>

<div class="node" class:selected class:dragging>
	<Handle id="top-handle" type="source" position={Position.Top} {isConnectable} />
	<Handle id="bottom-handle" type="source" position={Position.Bottom} {isConnectable} />
	<Handle id="right-handle" type="source" position={Position.Right} {isConnectable} />
	<Handle id="left-handle" type="source" position={Position.Left} {isConnectable} />
	<div class="node-shape">
		{#if type === ProcessType.Pool}
			<PoolNodeIcon />
		{:else if type === ProcessType.Source}
			<SourceNodeIcon />
		{:else if type === ProcessType.Drain}
			<DrainNodeIcon />
		{:else if type === ProcessType.Delay}
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
		color: var(--cyan);
	}

	.dragging {
		opacity: 0.8;
	}
</style>
