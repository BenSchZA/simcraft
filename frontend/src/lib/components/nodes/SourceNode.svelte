<script lang="ts">
	import { Handle, Position, type NodeProps } from '@xyflow/svelte';
	import type { Writable } from 'svelte/store';

	type $$Props = NodeProps;

	export let data: { label: Writable<string> };
	export let selected = false;
	export let dragging = false;
	export let isConnectable = true;

	const { label } = data;
</script>

<div class="source-node" class:selected class:dragging>
	<Handle id="top-handle" type="source" position={Position.Top} {isConnectable} />
	<Handle id="bottom-handle" type="source" position={Position.Bottom} {isConnectable} />
	<Handle id="right-handle" type="source" position={Position.Right} {isConnectable} />
	<Handle id="left-handle" type="source" position={Position.Left} {isConnectable} />
	<div class="label">{label}</div>
</div>

<style>
	.source-node {
		background-color: #2ea44f;
		color: white;
		border: 1px solid #2c974b;
		border-radius: 4px;
		padding: 10px;
		min-width: 100px;
		min-height: 50px;
		text-align: center;
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
	}

	.selected {
		box-shadow: 0 0 0 2px #fff;
	}

	.dragging {
		opacity: 0.8;
	}

	.label {
		font-size: 12px;
	}

	:global(.source-node .react-flow__handle) {
		width: 8px;
		height: 8px;
		background-color: #555;
		border-radius: 50%;
		border: 1px solid #404040;
	}

	:global(.source-node .react-flow__handle:hover) {
		background-color: #777;
	}
</style>
