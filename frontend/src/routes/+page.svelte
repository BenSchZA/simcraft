<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Chart } from 'chart.js/auto';
	import { adapter } from '$lib/simcraft';
	import type { SimulationState, PoolState, SourceState } from '$lib/simcraft';

	import { debug } from '@tauri-apps/plugin-log';

	let sources: { id: string; name: string }[] = [{ id: 'source-1', name: 'Test Source' }];
	let pools: { id: string; name: string }[] = [{ id: 'pool-1', name: 'Test Pool' }];
	let connections: { id: string; sourceId: string; targetId: string }[] = [
		{ id: 'connection-1', sourceId: 'source-1', targetId: 'pool-1' }
	];
	let simulationResults: SimulationState[] = [];
	let isSimulating = false;
	let chart: Chart | null = null;

	let newSourceName = '';
	let newPoolName = '';
	let selectedSource = '';
	let selectedPool = '';

	function addSource() {
		if (newSourceName) {
			sources = [...sources, { id: `source-${sources.length + 1}`, name: newSourceName }];
			newSourceName = '';
		}
	}

	function addPool() {
		if (newPoolName) {
			pools = [...pools, { id: `pool-${pools.length + 1}`, name: newPoolName }];
			newPoolName = '';
		}
	}

	function addConnection() {
		if (selectedSource && selectedPool) {
			connections = [
				...connections,
				{
					id: `connection-${connections.length + 1}`,
					sourceId: selectedSource,
					targetId: selectedPool
				}
			];
			selectedSource = '';
			selectedPool = '';
		}
	}

	function updateChart() {
		if (!chart || simulationResults.length === 0) return;

		const datasets = [];
		const labels = simulationResults.map((state) => `Step ${state.step}`);

		// Add datasets for each pool's resources
		for (const pool of pools) {
			const data = simulationResults.map((state) => {
				const poolState = state.process_states[pool.id];
				if (poolState && 'Pool' in poolState) {
					return poolState.Pool.resources;
				}
				return 0;
			});

			datasets.push({
				label: `${pool.name} Resources`,
				data,
				borderColor: `hsl(${(360 * datasets.length) / pools.length}, 70%, 50%)`,
				tension: 0.1
			});
		}

		// Add datasets for each source's produced resources
		for (const source of sources) {
			const data = simulationResults.map((state) => {
				const sourceState = state.process_states[source.id];
				if (sourceState && 'Source' in sourceState) {
					return sourceState.Source.resources_produced;
				}
				return 0;
			});

			datasets.push({
				label: `${source.name} Produced`,
				data,
				borderColor: `hsl(${(360 * (datasets.length + 1)) / (sources.length + 1)}, 70%, 50%)`,
				tension: 0.1
			});
		}

		chart.data = { labels, datasets };
		chart.update();
	}

	async function runSimulation() {
		if (isSimulating) return;
		isSimulating = true;
		simulationResults = [];

		const processes = [
			{ type: 'Stepper', id: 'stepper' },
			...sources.map((s) => ({ type: 'Source', id: s.id })),
			...pools.map((p) => ({ type: 'Pool', id: p.id }))
		];

		const simulationConnections = connections.map((c) => ({
			id: c.id,
			sourceID: c.sourceId,
			sourcePort: 'out',
			targetID: c.targetId,
			targetPort: 'in'
		}));

		try {
			const simulation = adapter;
			await simulation.initialise(processes, simulationConnections);

			while (isSimulating) {
				const results = await simulation.step();
				simulationResults = [...simulationResults, ...results];
				updateChart();
				await new Promise((resolve) => setTimeout(resolve, 100));
			}

			await simulation.destroy();
		} catch (error) {
			console.error('Simulation error:', error);
			isSimulating = false;
		}
	}

	function stopSimulation() {
		isSimulating = false;
	}

	onMount(() => {
		const ctx = document.getElementById('simulationChart') as HTMLCanvasElement;
		chart = new Chart(ctx, {
			type: 'line',
			data: {
				labels: [],
				datasets: []
			},
			options: {
				responsive: true,
				animation: false,
				scales: {
					y: {
						beginAtZero: true
					}
				}
			}
		});
	});

	onDestroy(() => {
		if (chart) {
			chart.destroy();
		}
	});
</script>

<div class="container">
	<div class="section">
		<h2>Add Components</h2>

		<div class="form-group">
			<h3>Add Source</h3>
			<input type="text" bind:value={newSourceName} placeholder="Source name" />
			<button on:click={addSource}>Add Source</button>
		</div>

		<div class="form-group">
			<h3>Add Pool</h3>
			<input type="text" bind:value={newPoolName} placeholder="Pool name" />
			<button on:click={addPool}>Add Pool</button>
		</div>

		<div class="form-group">
			<h3>Add Connection</h3>
			<select bind:value={selectedSource}>
				<option value="">Select Source</option>
				{#each sources as source}
					<option value={source.id}>{source.name}</option>
				{/each}
			</select>
			<select bind:value={selectedPool}>
				<option value="">Select Pool</option>
				{#each pools as pool}
					<option value={pool.id}>{pool.name}</option>
				{/each}
			</select>
			<button on:click={addConnection}>Connect</button>
		</div>
	</div>

	<div class="section">
		<h2>Components</h2>

		<div class="components-list">
			<div>
				<h3>Sources</h3>
				<ul>
					{#each sources as source}
						<li>{source.name} (ID: {source.id})</li>
					{/each}
				</ul>
			</div>

			<div>
				<h3>Pools</h3>
				<ul>
					{#each pools as pool}
						<li>{pool.name} (ID: {pool.id})</li>
					{/each}
				</ul>
			</div>

			<div>
				<h3>Connections</h3>
				<ul>
					{#each connections as connection}
						<li>{connection.sourceId} → {connection.targetId}</li>
					{/each}
				</ul>
			</div>
		</div>
	</div>

	<div class="section">
		<h2>Simulation Controls</h2>
		<div class="button-group">
			<button on:click={runSimulation} disabled={isSimulating}>Run Simulation</button>
			<button on:click={stopSimulation} disabled={!isSimulating}>Stop Simulation</button>
		</div>
	</div>

	<div class="section">
		<h2>Simulation Results</h2>
		<div class="chart-container">
			<canvas id="simulationChart"></canvas>
		</div>
	</div>
</div>

<style>
	.container {
		max-width: 1200px;
		margin: 0 auto;
		padding: 20px;
	}

	.section {
		margin-bottom: 30px;
		padding: 20px;
		border: 1px solid #ccc;
		border-radius: 5px;
	}

	.form-group {
		margin-bottom: 20px;
	}

	.button-group {
		display: flex;
		gap: 10px;
	}

	.form-group input,
	.form-group select {
		margin-right: 10px;
		padding: 5px;
		border: 1px solid #ccc;
		border-radius: 3px;
	}

	button {
		padding: 5px 10px;
		background-color: #4caf50;
		color: white;
		border: none;
		border-radius: 3px;
		cursor: pointer;
	}

	button:disabled {
		background-color: #cccccc;
		cursor: not-allowed;
	}

	.components-list {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 20px;
	}

	.chart-container {
		width: 100%;
		height: 400px;
	}

	ul {
		list-style-type: none;
		padding: 0;
	}

	li {
		margin-bottom: 5px;
	}
</style>
