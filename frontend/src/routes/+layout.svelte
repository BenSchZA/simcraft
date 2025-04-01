<script lang="ts">
	import '../app.css';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	let { children } = $props();
</script>

<div class="bg-primary min-h-screen">
	<header class="bg-secondary border-accent/20 h-12 border-b">
		<div class="flex h-full items-center justify-between px-4">
			<h1 class="text-primary text-base font-medium">Simcraft</h1>
			<div class="flex items-center">
				<ThemeToggle />
			</div>
		</div>
	</header>

	<main class="h-[calc(100vh-3rem)]">
		{@render children()}
	</main>
</div>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		font-family:
			-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans',
			'Helvetica Neue', sans-serif;
	}
</style>
