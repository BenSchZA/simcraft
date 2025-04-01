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

<div class="layout bg-primary">
	<SvelteFlowProvider>
		<DnDProvider>
			<Sidebar />
			<div class="main-content" class:sidebar-hidden={!$sidebarVisible}>
				<div class="content-wrapper">
					<TabSystem />
					<div class="editor-container bg-white">
						<FlowEditor />
					</div>
				</div>
				{#if !$sidebarVisible}
					<button
						class="show-sidebar-button bg-secondary text-primary hover:bg-accent/20 border-accent/30 border"
						on:click={() => ($sidebarVisible = true)}
					>
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
		height: 100%;
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
		border-radius: 4px;
		padding: 8px;
		cursor: pointer;
		transition: all 0.3s ease;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		font-family:
			-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
			'Helvetica Neue', sans-serif;
	}
</style>
