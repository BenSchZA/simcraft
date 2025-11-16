<script lang="ts">
	import {
		activeModelId,
		shouldResetChart,
		isActiveSimulationRunning,
		setSimulationRunning,
		activeSimulation,
		SimulationError,
		openModels,
		getOrCreateSimulationInstance
	} from '$lib/stores/simulation';
	import { get, writable } from 'svelte/store';
	import { onMount, onDestroy } from 'svelte';
	import { openChartForModel } from '$lib/stores/panelLayout';

	const stepDelay = writable(100);
	const maxSteps = writable(Infinity);
	const stepsPerSecond = writable(0);
	let lastStep = 0;
	let lastStepTime = 0;
	let performanceInterval: NodeJS.Timeout | undefined;

	$: if ($isActiveSimulationRunning) {
		lastStepTime = performance.now();
	} else {
		stepsPerSecond.set(0);
	}

	async function playMaxSpeed() {
		if (!$activeModelId) return;

		try {
			const instance = await getOrCreateSimulationInstance($activeModelId);
			if (!instance?.adapter) {
				throw new SimulationError('Failed to initialize simulation');
			}

			lastStepTime = performance.now();
			setSimulationRunning($activeModelId, true);

			// Open chart when simulation starts
			const model = $openModels.get($activeModelId);
			if (model) {
				openChartForModel($activeModelId, model.name);
			}

			await instance.adapter.play(0); // Use zero delay for max speed
		} catch (error) {
			console.error('Simulation error:', error);
			throw new SimulationError('Failed to start simulation at max speed', error);
		}
	}

	onMount(() => {
		// Update performance metrics every second
		performanceInterval = setInterval(() => {
			if ($isActiveSimulationRunning && $activeSimulation?.adapter) {
				const now = performance.now();
				const timeDiff = now - lastStepTime;
				if (timeDiff > 0) {
					$activeSimulation.adapter.getCurrentStep().then((step) => {
						if (!$activeSimulation) return;
						const stepDiff = step - lastStep;
						stepsPerSecond.set(Math.round((stepDiff / timeDiff) * 1000));
						lastStep = step;
					});
				}
				lastStepTime = now;
			}
		}, 1000);
	});

	onDestroy(() => {
		if (performanceInterval) {
			clearInterval(performanceInterval);
		}
	});

	async function play() {
		if (!$activeModelId) return;

		try {
			const instance = await getOrCreateSimulationInstance($activeModelId);
			if (!instance?.adapter) {
				throw new SimulationError('Failed to initialize simulation');
			}

			lastStepTime = performance.now();
			setSimulationRunning($activeModelId, true);

			// Open chart when simulation starts
			const model = $openModels.get($activeModelId);
			if (model) {
				openChartForModel($activeModelId, model.name);
			}

			await instance.adapter.play(get(stepDelay));
		} catch (error) {
			console.error('Simulation error:', error);
			throw new SimulationError('Failed to start simulation', error);
		}
	}

	async function pause() {
		if (!$activeModelId || !$activeSimulation?.adapter) return;

		try {
			console.log('Pausing simulation');
			await $activeSimulation.adapter.pause();
			setSimulationRunning($activeModelId, false);
			console.log('Simulation paused');

			// Reset performance tracking
			stepsPerSecond.set(0);
			lastStep = 0;
			lastStepTime = 0;
		} catch (error) {
			console.error('Stop simulation error:', error);
			throw new SimulationError('Failed to pause simulation', error);
		}
	}

	async function step() {
		if (!$activeModelId) return;

		try {
			if ($isActiveSimulationRunning) return;

			const instance = await getOrCreateSimulationInstance($activeModelId);
			if (!instance?.adapter) {
				throw new SimulationError('Failed to initialize simulation');
			}

			await instance.adapter.step();
		} catch (error) {
			console.error('Step error:', error);
			throw new SimulationError('Failed to step simulation', error);
		}
	}

	async function reset() {
		if (!$activeModelId || !$activeSimulation?.adapter) return;

		try {
			await pause();
			await $activeSimulation.adapter.reset();
			setSimulationRunning($activeModelId, false);
			$shouldResetChart = true;
		} catch (error) {
			console.error('Reset error:', error);
			throw new SimulationError('Failed to reset simulation', error);
		}
	}

	function openChart() {
		if (!$activeModelId) return;

		const model = $openModels.get($activeModelId);
		if (model) {
			openChartForModel($activeModelId, model.name);
		}
	}
</script>

<div class="controls">
	<div class="button-group">
		<button
			class="control-button"
			on:click={() => play().catch(console.error)}
			disabled={$isActiveSimulationRunning || !$activeModelId}
			title="Play"
		>
			▶
		</button>
		<button
			class="control-button"
			on:click={() => playMaxSpeed().catch(console.error)}
			disabled={$isActiveSimulationRunning || !$activeModelId}
			title="Play at Max Speed"
		>
			⏩
		</button>
		<button
			class="control-button"
			on:click={() => pause().catch(console.error)}
			disabled={!$isActiveSimulationRunning}
			title="Pause"
		>
			⏸
		</button>
		<button
			class="control-button"
			on:click={() => step().catch(console.error)}
			disabled={$isActiveSimulationRunning || !$activeModelId}
			title="Step"
		>
			⏭
		</button>
		<button
			class="control-button"
			on:click={() => reset().catch(console.error)}
			disabled={!$activeModelId}
			title="Reset"
		>
			↺
		</button>
		<button
			class="control-button chart-button"
			on:click={openChart}
			disabled={!$activeModelId}
			title="Open Chart"
		>
			📊
		</button>
		<div class="delay-input">
			<label class="delay-label" for="stepDelay">Step Delay:</label>
			<input
				id="stepDelay"
				type="number"
				bind:value={$stepDelay}
				min="0"
				disabled={$isActiveSimulationRunning}
			/>
			<span class="delay-label">ms</span>
		</div>
		<div class="delay-input">
			<label class="delay-label" for="maxSteps">Max Steps:</label>
			<input
				id="maxSteps"
				type="number"
				bind:value={$maxSteps}
				min="1"
				disabled={$isActiveSimulationRunning}
			/>
		</div>
		<div class="performance-counter">
			<span class="performance-label">{$stepsPerSecond} steps/s</span>
		</div>
	</div>
</div>

<style>
	.controls {
		padding: 0.5rem;
	}

	.button-group {
		display: flex;
		gap: 0.5rem;
		justify-content: center;
		align-items: center;
	}

	.control-button {
		background-color: rgba(45, 45, 45, 0.8);
		color: #fff;
		border: 1px solid rgba(64, 64, 64, 0.8);
		border-radius: 4px;
		padding: 0.5rem 0.5rem;
		font-size: 1rem;
		cursor: pointer;
		min-width: 3rem;
		backdrop-filter: blur(4px);
	}

	.control-button:hover:not(:disabled) {
		background-color: rgba(51, 51, 51, 0.8);
	}

	.control-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.delay-input {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-left: 1rem;
		color: #fff;
	}

	.delay-input input {
		width: 5rem;
		padding: 0.25rem;
		background-color: rgba(45, 45, 45, 0.8);
		border: 1px solid rgba(64, 64, 64, 0.8);
		color: #fff;
		border-radius: 4px;
		text-align: right;
		backdrop-filter: blur(4px);
	}

	.delay-input input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.delay-label {
		color: black;
		font-size: 0.9rem;
	}

	.performance-counter {
		display: flex;
		align-items: center;
		margin-left: 1rem;
		padding: 0.25rem 0.5rem;
		background-color: rgba(45, 45, 45, 0.8);
		border: 1px solid rgba(64, 64, 64, 0.8);
		border-radius: 4px;
		backdrop-filter: blur(4px);
	}

	.performance-label {
		color: #fff;
		font-size: 0.9rem;
	}
</style>
