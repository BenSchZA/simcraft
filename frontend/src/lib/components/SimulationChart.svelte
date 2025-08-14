<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Chart, type ChartDataset } from 'chart.js/auto';
	import {
		activeModelId,
		activeNodeId,
		shouldResetChart,
		activeSimulation,
		setSimulationStateUpdateCallback,
		openModels
	} from '$lib/stores/simulation';
	import { selectedElement } from '$lib/stores/viewStates';
	import type { SimulationState } from '$lib/simcraft/base';

	// Props to support standalone mode
	export let standalone: boolean = false;
	export let modelId: string | null = null;

	let chartCanvas: HTMLCanvasElement;
	let currentChart: Chart | null = null;
	const chartStates = new Map<
		string,
		{
			datasets: ChartDataset[];
			labels: number[];
		}
	>();

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
					let dataset = chartState.datasets.find((d) => d.label === id);
					if (!dataset) {
						dataset = {
							label: id,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0
						};
						chartState.datasets.push(dataset);
					}
					dataset.data.push({
						x: timestamp,
						y: processState.Pool.resources
					});
				}

				if (processState.Source) {
					let dataset = chartState.datasets.find((d) => d.label === id);
					if (!dataset) {
						dataset = {
							label: id,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0
						};
						chartState.datasets.push(dataset);
					}
					dataset.data.push({
						x: timestamp,
						y: processState.Source.resources_produced
					});
				}

				if (processState.Drain) {
					let dataset = chartState.datasets.find((d) => d.label === id);
					if (!dataset) {
						dataset = {
							label: id,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0
						};
						chartState.datasets.push(dataset);
					}
					dataset.data.push({
						x: timestamp,
						y: processState.Drain.resources_consumed
					});
				}

				if (processState.Delay) {
					let receivedDataset = chartState.datasets.find((d) => d.label === `${id}-received`);
					if (!receivedDataset) {
						receivedDataset = {
							label: `${id}-received`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0
						};
						chartState.datasets.push(receivedDataset);
					}
					receivedDataset.data.push({
						x: timestamp,
						y: processState.Delay.resources_received
					});

					let releasedDataset = chartState.datasets.find((d) => d.label === `${id}-released`);
					if (!releasedDataset) {
						releasedDataset = {
							label: `${id}-released`,
							data: [],
							tension: 0.1,
							borderWidth: 1,
							radius: 0
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
		// if (isActive && currentChart) {
		// 	const numberOfDataPoints = chartState.labels.length;
		// 	if (numberOfDataPoints > 1000) {
		// 		if (numberOfDataPoints % 100 === 0) {
		// 			currentChart.update();
		// 		}
		// 	} else if (numberOfDataPoints > 100) {
		// 		if (numberOfDataPoints % 10 === 0) {
		// 			currentChart.update();
		// 		}
		// 	} else {
		// 		currentChart.update();
		// 	}
		// }
		// Only update chart if it's for the active/target model
		if (isActive || (standalone && modelId === $activeModelId)) {
			currentChart?.update();
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

		// Ensure datasets have proper structure
		chartState.datasets = chartState.datasets.map((ds) => ({
			...ds,
			data: ds.data || [],
			hidden: ds.hidden || false
		}));

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
				maintainAspectRatio: false,
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
						beginAtZero: true,
						title: {
							display: true,
							text: 'State'
						}
					}
				},
				plugins: {
					decimation: {
						enabled: true
					},
					legend: {
						display: true,
						position: 'bottom',
						labels: {
							filter: (item) => !item.hidden,
							boxWidth: 12,
							font: {
								size: 11
							}
						}
					}
				}
			}
		});
	}

	/// Display the results for a given node, hide all other nodes by setting hidden on chart dataset meta
	function displayNodeResults(nodeId: string | null) {
		if (!currentChart) return;

		const chartState = chartStates.get($activeModelId || '');
		if (!chartState) return;

		for (const dataset of chartState.datasets) {
			if (nodeId) {
				// Show datasets that match the nodeId or are related (e.g., delay-received/released)
				dataset.hidden = !dataset.label?.startsWith(nodeId);
			} else {
				// Show all datasets when no node is selected
				dataset.hidden = false;
			}
		}

		// Update chart data reference to trigger re-render
		currentChart.data.datasets = [...chartState.datasets];
		currentChart.update('none'); // Use 'none' mode for immediate update without animation
	}

	// Ensure callbacks are registered for all models
	openModels.subscribe((models) => {
		for (const model of models.values()) {
			setSimulationStateUpdateCallback(model.id, (states) => {
				updateChart(model.id, states);
			});
		}
	});

	// Also update callbacks when simulation instances change
	$: if ($activeSimulation) {
		setSimulationStateUpdateCallback($activeModelId!, (states) => {
			updateChart($activeModelId!, states);
		});
	}

	// Use provided modelId in standalone mode, otherwise use activeModelId
	$: targetModelId = standalone && modelId ? modelId : $activeModelId;

	// Switch chart when active model changes
	$: if (targetModelId && chartCanvas) {
		createChart(targetModelId);
		// Re-apply node filter after chart recreation
		const nodeId =
			!standalone && $selectedElement && 'type' in $selectedElement ? $selectedElement.id : null;
		displayNodeResults(nodeId);
	}

	// Reset chart when requested
	$: if ($shouldResetChart && targetModelId) {
		resetChart(targetModelId);
		shouldResetChart.set(false);
	}

	// Update displayed node results when selection changes (only in non-standalone mode)
	// Use selectedElement as primary source since it's updated immediately
	$: if (!standalone) {
		const nodeId = $selectedElement && 'type' in $selectedElement ? $selectedElement.id : null;
		displayNodeResults(nodeId);
	}

	onDestroy(() => {
		destroyCurrentChart();
		// Clean up all model data
		chartStates.clear();
	});
</script>

<div class="panel {standalone ? 'standalone' : ''}">
	<div class="chart-container">
		<canvas bind:this={chartCanvas}></canvas>
	</div>
	{#if !standalone && (!$selectedElement || !('type' in $selectedElement))}
		<div class="chart-overlay">
			<p>Select a node to filter results</p>
		</div>
	{/if}
</div>

<style>
	.panel {
		position: relative;
		display: flex;
		height: 25vh;
		width: 25vw;
	}

	.panel.standalone {
		height: 100%;
		width: 100%;
	}

	.chart-container {
		position: relative;
		flex-grow: 1;
		min-height: 0;
		padding: 0.2rem;
	}

	.chart-overlay {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		background: rgba(255, 255, 255, 0.9);
		padding: 1rem 2rem;
		border-radius: 0.375rem;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		pointer-events: none;
	}

	.chart-overlay p {
		margin: 0;
		color: #6b7280;
		font-size: 0.875rem;
	}
</style>
