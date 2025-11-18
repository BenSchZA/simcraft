<script lang="ts">
	import '../app.css';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	let { children } = $props();
</script>

<div class="app-container">
	<!-- <header class="app-header">
		<div class="header-content">
			<h1>Simcraft</h1>
			<div class="header-actions">
				<ThemeToggle />
			</div>
		</div>
	</header> -->

	<main class="app-main">
		{@render children()}
	</main>
</div>

<style>
	.app-container {
		background: #ffffff;
		min-height: 100vh;
	}

	.app-main {
		height: 100vh;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		font-family:
			-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
			'Helvetica Neue', sans-serif;
		background: #ffffff;
	}

	:global(html) {
		background: #ffffff;
	}
</style>
