<script lang="ts">
	export let label: string;
	export let position = { x: 0, y: 40 };
	export let onPositionChange: (position: { x: number; y: number }) => void;

	let dragging = false;
	let startPos = { x: 0, y: 0 };

	function handleMouseDown(event: MouseEvent) {
		dragging = true;
		startPos = {
			x: event.clientX - position.x,
			y: event.clientY - position.y
		};
		event.stopPropagation();
	}

	function handleMouseMove(event: MouseEvent) {
		if (!dragging) return;
		const newPosition = {
			x: event.clientX - startPos.x,
			y: event.clientY - startPos.y
		};
		position = newPosition;
		onPositionChange(newPosition);
		event.preventDefault();
	}

	function handleMouseUp() {
		if (dragging) {
			onPositionChange(position);
		}
		dragging = false;
	}
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<div
	class="node-label"
	style="transform: translate({position.x}px, {position.y}px)"
	on:mousedown={handleMouseDown}
	role="none"
>
	{label}
</div>

<style>
	.node-label {
		position: absolute;
		font-size: 11px;
		padding: 1px 2px;
		cursor: move;
		user-select: none;
		white-space: nowrap;
		color: var(--text-secondary);
	}
</style>
