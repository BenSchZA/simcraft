<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Chart } from 'chart.js/auto';

	import { createAdapter, type SimulationState, type SimcraftAdapter } from '$lib/simcraft';

	let sources: { id: string; name: string }[] = [{ id: 'source-1', name: 'Test Source' }];
	let pools: { id: string; name: string }[] = [{ id: 'pool-1', name: 'Test Pool' }];
	let connections: { id: string; sourceId: string; targetId: string }[] = [
		{ id: 'connection-1', sourceId: 'source-1', targetId: 'pool-1' }
	];
	let isSimulating = false;
	let chart: Chart | null = null;

	let newSourceName = '';
	let newPoolName = '';
	let selectedSource = '';
	let selectedPool = '';

	let stepDelay = 100; // Default delay in ms
	let simulation: SimcraftAdapter | null = null;

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

	function updateChart(newStates: SimulationState[]) {
		if (!chart || newStates.length === 0) return;

		for (const state of newStates) {
			const timestamp = state.time;

			for (const [id, processState] of Object.entries(state.process_states)) {
				if ('Pool' in processState) {
					let dataset = chart.data.datasets.find((d) => d.label === id);

					if (!dataset) {
						dataset = {
							label: id,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0,
							borderColor: getColorForId(id),
							backgroundColor: getColorForId(id)
						};
						chart.data.datasets.push(dataset);
					}

					dataset.data.push({
						x: timestamp,
						y: processState.Pool?.resources || 0
					});

					chart.data.labels?.push(timestamp);
				}
			}
		}

		const numberOfDataPoints = chart.data.labels?.length || 0;
		if (numberOfDataPoints > 1000) {
			if (numberOfDataPoints % 100 === 0) {
				chart.update();
			}
		} else if (numberOfDataPoints > 100) {
			if (numberOfDataPoints % 10 === 0) {
				chart.update();
			}
		} else {
			chart.update();
		}
	}

	function getColorForId(id: string): string {
		const hash = Array.from(id).reduce((acc, char) => acc + char.charCodeAt(0), 0);
		const hue = hash % 360;
		return `hsl(${hue}, 70%, 50%)`;
	}

	function resetChart() {
		if (!chart) return;

		chart.data = {
			labels: [],
			datasets: []
		};
		chart.update();
	}

	async function initSimulation() {
		if (!simulation) {
			const processes = [
				{ type: 'Stepper', id: 'stepper' },
				...sources.map((s) => ({ type: 'Source', id: s.id })),
				...pools.map((p) => ({ type: 'Pool', id: p.id }))
			] as any[];

			const simulationConnections = connections.map((c) => ({
				id: c.id,
				sourceID: c.sourceId,
				sourcePort: 'out',
				targetID: c.targetId,
				targetPort: 'in',
				flowRate: 1.0
			}));

			const adapter = createAdapter();
			const initialState = await adapter.initialise(processes, simulationConnections);
			updateChart([initialState]);

			// Set up state update listener
			const unsubscribe = adapter.onStateUpdate((states) => {
				updateChart(states);
			});

			// Clean up listener on component destroy
			// return () => unsubscribe();
			return Promise.resolve(adapter);
		} else {
			throw new Error('Simulation already initialised');
		}
	}

	async function play() {
		if (isSimulating) return;

		if (!simulation) {
			simulation = await initSimulation();
		}

		try {
			const success = await simulation.play(stepDelay);
			if (success) {
				isSimulating = true;
			} else {
				throw new Error('Failed to start simulation');
			}
		} catch (error) {
			console.error('Simulation error:', error);
			isSimulating = false;
		}
	}

	async function pause() {
		if (!isSimulating || !simulation) return;

		try {
			const success = await simulation.pause();
			if (success) {
				isSimulating = false;
			}
		} catch (error) {
			console.error('Stop simulation error:', error);
		}
	}

	async function step() {
		if (isSimulating) return;

		if (!simulation) {
			simulation = await initSimulation();
		}

		try {
			const result = await simulation.step();
			const state = result.state;
			updateChart([state]);
		} catch (error) {
			console.error('Step error:', error);
		}
	}

	async function reset() {
		if (!simulation) return;

		await pause();
		await simulation.destroy();
		simulation = null;
		isSimulating = false;
		resetChart();
	}

	onMount(async () => {
		const ctx = document.getElementById('simulationChart') as HTMLCanvasElement;

		chart = new Chart(ctx, {
			type: 'line',
			data: {
				labels: [],
				datasets: []
			},
			options: {
				interaction: {
					intersect: false,
					mode: 'index'
				},
				responsive: true,
				animation: false,
				scales: {
					x: {
						type: 'linear',
						title: {
							display: true,
							text: 'Time'
						},
						ticks: {
							autoSkip: true
						}
					},
					y: {
						beginAtZero: true
					}
				},
				plugins: {
					decimation: {
						enabled: true
					}
					// tooltip: {
					// 	callbacks: {
					// 		footer: function (tooltipItems) {
					// 			return 'Step ' + tooltipItems[0].dataIndex;
					// 		}
					// 	}
					// }
				}
			}
		});
	});

	onDestroy(async () => {
		if (chart) {
			chart.destroy();
		}
		if (simulation) {
			await simulation.destroy();
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
		<div class="control-group">
			<div class="form-group">
				<label class="form-group-label" for="stepDelay">Step Delay (ms):</label>
				<input
					type="number"
					id="stepDelay"
					bind:value={stepDelay}
					min="10"
					max="5000"
					disabled={isSimulating}
				/>
			</div>
		</div>
		<div class="button-group">
			<button on:click={play} disabled={isSimulating}>Play</button>
			<button on:click={pause} disabled={!isSimulating || !simulation}>Pause</button>
			<button on:click={step} disabled={isSimulating}>Step</button>
			<button on:click={reset} disabled={!simulation}>Reset</button>
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
	}

	ul {
		list-style-type: none;
		padding: 0;
	}

	li {
		margin-bottom: 5px;
	}

	.control-group {
		display: flex;
		gap: 20px;
		margin-bottom: 15px;
	}

	.form-group {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.form-group input[type='number'] {
		width: 100px;
		padding: 5px;
		border: 1px solid #ccc;
		border-radius: 3px;
	}

	.form-group-label {
		font-weight: bold;
		color: black;
	}

	button:not(:disabled):hover {
		background-color: #45a049;
	}
</style>
