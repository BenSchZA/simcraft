<script lang="ts">
	import { onMount } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TabSystem from '$lib/components/TabSystem.svelte';
	import FlowEditor from '$lib/components/FlowEditor.svelte';
	import { sidebarVisible } from '$lib/stores/simulation';
	import { storageManager } from '$lib/storage/StorageManager';
	import { SvelteFlowProvider } from '@xyflow/svelte';
	import DnDProvider from '$lib/components/DnDProvider.svelte';

	onMount(async () => {
		await storageManager.init();
	});
</script>

<div class="layout">
	<SvelteFlowProvider>
		<DnDProvider>
			<Sidebar />
			<div class="main-content" class:sidebar-hidden={!$sidebarVisible}>
				<div class="content-wrapper">
					<TabSystem />
					<div class="editor-container">
						<FlowEditor />
					</div>
				</div>
				{#if !$sidebarVisible}
					<button class="show-sidebar-button" on:click={() => ($sidebarVisible = true)}>
						☰
					</button>
				{/if}
			</div>
		</DnDProvider>
	</SvelteFlowProvider>
</div>

<style>
	.layout {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}

	.main-content {
		flex: 1;
		display: flex;
		position: relative;
		margin-left: 0;
		transition: margin-left 0.3s ease;
	}

	.main-content.sidebar-hidden {
		margin-left: -250px;
	}

	.content-wrapper {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.editor-container {
		flex: 1;
		position: relative;
		overflow: hidden;
	}

	.show-sidebar-button {
		position: absolute;
		top: 10px;
		left: 10px;
		z-index: 1000;
		background-color: #2d2d2d;
		color: white;
		border: 1px solid #404040;
		border-radius: 4px;
		padding: 8px;
		cursor: pointer;
	}

	.show-sidebar-button:hover {
		background-color: #333;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		font-family:
			-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
			'Helvetica Neue', sans-serif;
	}
</style>
