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

<div class="node-types-panel bg-secondary/80 shadow-lg backdrop-blur-sm">
	<div class="node-types flex gap-3 p-2">
		{#each Object.entries(nodeTypes) as [processType, Icon]}
			<div
				class="node text-primary hover:text-accent flex cursor-grab items-center justify-center transition-colors"
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
		top: 1rem;
		left: 50%;
		transform: translateX(-50%);
		border-radius: 2rem;
		z-index: 100;
	}

	.node {
		padding: 0.5rem;
		min-width: 2.5rem;
		height: 2.5rem;
	}

	.node:active {
		cursor: grabbing;
	}
</style>
