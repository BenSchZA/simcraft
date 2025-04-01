<script lang="ts">
	import {
		activeModelId,
		shouldResetChart,
		getInitialisedSimulation,
		isActiveSimulationRunning,
		setSimulationRunning
	} from '$lib/stores/simulation';
	import { get, writable } from 'svelte/store';
	import { SimulationError } from '$lib/stores/simulationManager';

	const stepDelay = writable(100);

	async function play() {
		if (!$activeModelId) return;

		try {
			const simulation = await getInitialisedSimulation($activeModelId);
			if (simulation.adapter.isRunning()) return;

			await simulation.adapter.play(get(stepDelay));
			setSimulationRunning($activeModelId, true);
		} catch (error) {
			console.error('Simulation error:', error);
			throw new SimulationError('Failed to start simulation', error);
		}
	}

	async function pause() {
		if (!$activeModelId) return;

		try {
			const simulation = await getInitialisedSimulation($activeModelId);
			await simulation.adapter.pause();
			setSimulationRunning($activeModelId, false);
		} catch (error) {
			console.error('Stop simulation error:', error);
			throw new SimulationError('Failed to pause simulation', error);
		}
	}

	async function step() {
		if (!$activeModelId) return;

		try {
			const simulation = await getInitialisedSimulation($activeModelId);
			if (simulation.adapter.isRunning()) return;

			await simulation.adapter.step();
		} catch (error) {
			console.error('Step error:', error);
			throw new SimulationError('Failed to step simulation', error);
		}
	}

	async function reset() {
		if (!$activeModelId) return;

		try {
			const simulation = await getInitialisedSimulation($activeModelId);
			await pause();
			await simulation.adapter.reset();
			setSimulationRunning($activeModelId, false);
			$shouldResetChart = true;
		} catch (error) {
			console.error('Reset error:', error);
			throw new SimulationError('Failed to reset simulation', error);
		}
	}

	async function updateStepDelay() {
		if ($activeModelId) {
			try {
				const simulation = await getInitialisedSimulation($activeModelId);
				simulation.stepDelay = $stepDelay;
			} catch (error) {
				console.error('Failed to update step delay:', error);
			}
		}
	}

	$: if ($activeModelId || $stepDelay) {
		updateStepDelay();
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
		<div class="delay-input">
			<label class="delay-label" for="stepDelay">Step Delay:</label>
			<input
				id="stepDelay"
				type="number"
				bind:value={$stepDelay}
				disabled={$isActiveSimulationRunning}
			/>
			<span class="delay-label">ms</span>
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
</style>
