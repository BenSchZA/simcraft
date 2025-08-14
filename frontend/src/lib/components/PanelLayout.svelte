<script lang="ts">
	import {
		panelLayout,
		isPanelGroup,
		isPanel,
		resizePanels,
		type PanelLayout as PanelLayoutType
	} from '$lib/stores/panelLayout';
	import TabPanel from './TabPanel.svelte';
	import { onMount } from 'svelte';

	export let layout: PanelLayoutType;

	let containerEl: HTMLElement;
	let isResizing = false;
	let resizeStartX = 0;
	let resizeStartY = 0;
	let resizeIndex = -1;
	let initialSizes: number[] = [];

	function handleMouseDown(event: MouseEvent, index: number) {
		if (!isPanelGroup(layout)) return;

		isResizing = true;
		resizeIndex = index;
		resizeStartX = event.clientX;
		resizeStartY = event.clientY;

		// Store initial sizes
		initialSizes = layout.panels.map((p) => p.size);

		// Prevent text selection during resize
		event.preventDefault();
		document.body.style.cursor = layout.direction === 'horizontal' ? 'col-resize' : 'row-resize';
		document.body.style.userSelect = 'none';
	}

	function handleMouseMove(event: MouseEvent) {
		if (!isResizing || !isPanelGroup(layout) || !containerEl) return;

		const containerRect = containerEl.getBoundingClientRect();
		const totalSize =
			layout.direction === 'horizontal' ? containerRect.width : containerRect.height;
		const delta =
			layout.direction === 'horizontal'
				? event.clientX - resizeStartX
				: event.clientY - resizeStartY;

		const deltaPercent = (delta / totalSize) * 100;

		// Calculate new sizes
		const newSizes = [...initialSizes];
		newSizes[resizeIndex] += deltaPercent;
		newSizes[resizeIndex + 1] -= deltaPercent;

		// Ensure minimum sizes (10%)
		if (newSizes[resizeIndex] >= 10 && newSizes[resizeIndex + 1] >= 10) {
			resizePanels(layout.id, newSizes);
		}
	}

	function handleMouseUp() {
		if (!isResizing) return;

		isResizing = false;
		resizeIndex = -1;
		document.body.style.cursor = '';
		document.body.style.userSelect = '';
	}

	// Global mouse event handlers
	onMount(() => {
		const handleGlobalMouseMove = (e: MouseEvent) => handleMouseMove(e);
		const handleGlobalMouseUp = () => handleMouseUp();

		document.addEventListener('mousemove', handleGlobalMouseMove);
		document.addEventListener('mouseup', handleGlobalMouseUp);

		return () => {
			document.removeEventListener('mousemove', handleGlobalMouseMove);
			document.removeEventListener('mouseup', handleGlobalMouseUp);
		};
	});
</script>

<div
	class="panel-layout {isPanelGroup(layout) ? `panel-group-${layout.direction}` : ''}"
	bind:this={containerEl}
>
	{#if isPanel(layout)}
		<TabPanel panel={layout} />
	{:else if isPanelGroup(layout)}
		{#each layout.panels as panel, index}
			<div class="panel-wrapper" style="flex: {panel.size} 1 0%;">
				<svelte:self layout={panel} />
			</div>
			{#if index < layout.panels.length - 1}
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<div
					class="resize-handle {layout.direction}"
					on:mousedown={(e) => handleMouseDown(e, index)}
					role="separator"
					aria-orientation={layout.direction === 'horizontal' ? 'vertical' : 'horizontal'}
				>
					<div class="resize-handle-inner"></div>
				</div>
			{/if}
		{/each}
	{/if}
</div>

<style>
	.panel-layout {
		width: 100%;
		height: 100%;
		display: flex;
		position: relative;
	}

	.panel-group-horizontal {
		flex-direction: row;
	}

	.panel-group-vertical {
		flex-direction: column;
	}

	.panel-wrapper {
		min-width: 0;
		min-height: 0;
		overflow: hidden;
		display: flex;
	}

	.resize-handle {
		position: relative;
		background: transparent;
		transition: background-color 0.2s;
		flex-shrink: 0;
		z-index: 10;
	}

	.resize-handle.horizontal {
		width: 4px;
		cursor: col-resize;
	}

	.resize-handle.vertical {
		height: 4px;
		cursor: row-resize;
	}

	.resize-handle:hover {
		background-color: var(--accent);
	}

	.resize-handle-inner {
		position: absolute;
		background: #e5e7eb;
		transition: background-color 0.2s;
	}

	.resize-handle.horizontal .resize-handle-inner {
		left: 1px;
		right: 1px;
		top: 0;
		bottom: 0;
	}

	.resize-handle.vertical .resize-handle-inner {
		top: 1px;
		bottom: 1px;
		left: 0;
		right: 0;
	}

	.resize-handle:hover .resize-handle-inner {
		background-color: var(--accent);
	}

	/* Active resizing state */
	:global(body.resizing) .resize-handle {
		background-color: var(--accent);
	}

	:global(body.resizing) .resize-handle-inner {
		background-color: var(--accent);
	}
</style>
