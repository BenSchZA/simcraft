<script lang="ts">
	import { useDnD } from '$lib/utils/dnd';
	import SourceNodeIcon from './nodes/node-icons/SourceNodeIcon.svelte';
	import PoolNodeIcon from './nodes/node-icons/PoolNodeIcon.svelte';
	import DrainNodeIcon from './nodes/node-icons/DrainNodeIcon.svelte';
	import DelayNodeIcon from './nodes/node-icons/DelayNodeIcon.svelte';
	import { ProcessType } from '$lib/simcraft';

	const nodeType = useDnD();

	const nodeTypes = {
		[ProcessType.Source]: SourceNodeIcon,
		[ProcessType.Pool]: PoolNodeIcon,
		[ProcessType.Drain]: DrainNodeIcon,
		[ProcessType.Delay]: DelayNodeIcon
	};

	const onDragStart = (event: DragEvent, processType: ProcessType) => {
		if (!event.dataTransfer) return;
		nodeType.set(processType);
		event.dataTransfer.effectAllowed = 'move';
	};

	const onKeyDown = (event: KeyboardEvent, processType: ProcessType) => {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			onNodeClick(processType);
		}
	};

	let { onNodeClick } = $props();
</script>

<div class="node-types-panel">
	<div class="node-types">
		{#each Object.entries(nodeTypes) as [processType, Icon]}
			<div
				class="node"
				draggable={true}
				ondragstart={(event) => onDragStart(event, processType as ProcessType)}
				onclick={() => onNodeClick(processType as ProcessType)}
				onkeydown={(event) => onKeyDown(event, processType as ProcessType)}
				role="button"
				tabindex="0"
				title={processType}
			>
				{#key Icon}
					<Icon />
				{/key}
			</div>
		{/each}
	</div>
</div>

<style>
	.node-types-panel {
		position: absolute;
		border-radius: 12px;
		z-index: 100;
		background: linear-gradient(135deg, rgba(250, 250, 250, 0.95) 0%, rgba(245, 245, 245, 0.95) 100%);
		border: 1px solid rgba(0, 0, 0, 0.1);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
		backdrop-filter: blur(8px);
		padding: 0.5rem;
	}

	.node-types {
		display: flex;
		gap: 0.75rem;
		padding: 0;
		align-items: center;
	}

	.node {
		padding: 0.625rem;
		min-width: 44px;
		height: 44px;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 8px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: grab;
	}

	.node:hover {
		background: #3b82f6;
		border-color: #3b82f6;
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
		transform: translateY(-2px);
	}

	.node:active {
		cursor: grabbing;
		transform: translateY(0);
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
	}

	.node:hover :global(svg) {
		color: white;
	}
</style>
