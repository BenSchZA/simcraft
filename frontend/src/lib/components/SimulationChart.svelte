<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Chart, type ChartDataset } from 'chart.js/auto';
	import {
		activeModelId,
		simulationInstances,
		shouldResetChart,
		getInitialisedSimulation,
		openModels
	} from '$lib/stores/simulation';
	import { type SimulationState } from '$lib/simcraft';
	import type { ModelMetadata } from '$lib/stores/simulation';
	import { get } from 'svelte/store';

	let chartCanvas: HTMLCanvasElement;
	let currentChart: Chart | null = null;
	const chartStates = new Map<
		string,
		{
			datasets: ChartDataset[];
			labels: number[];
		}
	>();

	function getColorForId(id: string): string {
		const hash = Array.from(id).reduce((acc, char) => acc + char.charCodeAt(0), 0);
		const hue = hash % 360;
		return `hsl(${hue}, 70%, 50%)`;
	}

	function getOrCreateChartState(modelId: string) {
		if (!chartStates.has(modelId)) {
			chartStates.set(modelId, {
				datasets: [],
				labels: []
			});
		}
		return chartStates.get(modelId)!;
	}

	function updateChart(modelId: string, newStates: SimulationState[]) {
		if (newStates.length === 0) return;

		const chartState = getOrCreateChartState(modelId);
		const isActive = modelId === $activeModelId;

		for (const state of newStates) {
			const timestamp = state.time;

			for (const [id, processState] of Object.entries(state.process_states)) {
				if (processState.Pool) {
					let dataset = chartState.datasets.find((d) => d.label === `${id} (resources)`);
					if (!dataset) {
						dataset = {
							label: `${id} (resources)`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0,
							borderColor: getColorForId(id),
							backgroundColor: getColorForId(id)
						};
						chartState.datasets.push(dataset);
					}
					dataset.data.push({
						x: timestamp,
						y: processState.Pool.resources
					});
				}

				if (processState.Source) {
					let dataset = chartState.datasets.find((d) => d.label === `${id} (produced)`);
					if (!dataset) {
						dataset = {
							label: `${id} (produced)`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0,
							borderColor: getColorForId(id),
							backgroundColor: getColorForId(id)
						};
						chartState.datasets.push(dataset);
					}
					dataset.data.push({
						x: timestamp,
						y: processState.Source.resources_produced
					});
				}

				if (processState.Drain) {
					let dataset = chartState.datasets.find((d) => d.label === `${id} (consumed)`);
					if (!dataset) {
						dataset = {
							label: `${id} (consumed)`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0,
							borderColor: getColorForId(id),
							backgroundColor: getColorForId(id)
						};
						chartState.datasets.push(dataset);
					}
					dataset.data.push({
						x: timestamp,
						y: processState.Drain.resources_consumed
					});
				}

				if (processState.Delay) {
					let receivedDataset = chartState.datasets.find((d) => d.label === `${id} (received)`);
					if (!receivedDataset) {
						receivedDataset = {
							label: `${id} (received)`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0,
							borderColor: getColorForId(`${id}-received`),
							backgroundColor: getColorForId(`${id}-received`)
						};
						chartState.datasets.push(receivedDataset);
					}
					receivedDataset.data.push({
						x: timestamp,
						y: processState.Delay.resources_received
					});

					let releasedDataset = chartState.datasets.find((d) => d.label === `${id} (released)`);
					if (!releasedDataset) {
						releasedDataset = {
							label: `${id} (released)`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0,
							borderColor: getColorForId(`${id}-released`),
							backgroundColor: getColorForId(`${id}-released`)
						};
						chartState.datasets.push(releasedDataset);
					}
					releasedDataset.data.push({
						x: timestamp,
						y: processState.Delay.resources_released
					});
				}
			}

			chartState.labels.push(timestamp);
		}

		// Only update visible chart if this is the active model
		if (isActive && currentChart) {
			const numberOfDataPoints = chartState.labels.length;
			if (numberOfDataPoints > 1000) {
				if (numberOfDataPoints % 100 === 0) {
					currentChart.update();
				}
			} else if (numberOfDataPoints > 100) {
				if (numberOfDataPoints % 10 === 0) {
					currentChart.update();
				}
			} else {
				currentChart.update();
			}
		}
	}

	function resetChart(modelId: string) {
		const chartState = getOrCreateChartState(modelId);
		chartState.datasets = [];
		chartState.labels = [];
		if (currentChart && modelId === $activeModelId) {
			currentChart.data.datasets = [];
			currentChart.data.labels = [];
			currentChart.update();
		}
	}

	function destroyCurrentChart() {
		if (currentChart) {
			currentChart.destroy();
			currentChart = null;
		}
	}

	function createChart(modelId: string) {
		if (!chartCanvas) return;

		destroyCurrentChart();

		const chartState = getOrCreateChartState(modelId);
		currentChart = new Chart(chartCanvas, {
			type: 'line',
			data: {
				labels: chartState.labels,
				datasets: chartState.datasets
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
				}
			}
		});
	}

	async function setupSimulationListener(modelId: string) {
		const simulation = await getInitialisedSimulation(modelId);

		// Each model gets its own state update listener
		simulation.adapter.onStateUpdate((states) => {
			updateChart(modelId, states);
		});

		// Store cleanup function in simulation instance
		simulation.unsubscribe = () => {
			resetChart(modelId);
		};
	}

	// Set up listeners for all open models
	openModels.subscribe(async (models: Map<string, ModelMetadata>) => {
		for (const [modelId] of models) {
			await setupSimulationListener(modelId);
		}
	});

	// Switch chart when active model changes
	$: if ($activeModelId && chartCanvas) {
		createChart($activeModelId);
	}

	// Reset chart when requested
	$: if ($shouldResetChart && $activeModelId) {
		resetChart($activeModelId);
		shouldResetChart.set(false);
	}

	onDestroy(() => {
		destroyCurrentChart();
		// Clean up all model data
		for (const [modelId] of chartStates) {
			const simulation = get(simulationInstances).get(modelId);
			if (simulation?.unsubscribe) {
				simulation.unsubscribe();
			}
		}
		chartStates.clear();
	});
</script>

<div class="chart-container" class:hidden={!$activeModelId}>
	<canvas bind:this={chartCanvas}></canvas>
</div>

<style>
	.chart-container {
		width: 100%;
		padding: 0.5rem;
		box-sizing: border-box;
	}

	.hidden {
		display: none;
	}

	canvas {
		width: 100% !important;
	}
</style>
