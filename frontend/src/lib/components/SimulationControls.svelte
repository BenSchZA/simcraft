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
		performanceInterval = setInterval(() => {
			if ($isActiveSimulationRunning && $activeSimulation?.adapter) {
				const now = performance.now();
				const timeDiff = now - lastStepTime;
				if (timeDiff > 0) {
					$activeSimulation.adapter.getCurrentStep().then((step) => {
						if (!$activeSimulation) return;
						if (!lastStep) {
							lastStep = step;
							return;
						}
						const stepDiff = step - lastStep;
						if (stepDiff > 0) {
							stepsPerSecond.set(Math.round((stepDiff / timeDiff) * 1000));
							lastStep = step;
						}
					});
				}
				lastStepTime = now;
			}
		}, 100);
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
	<!-- Primary Control Buttons -->
	<div class="control-buttons">
		<button
			class="control-btn"
			on:click={() => play().catch(console.error)}
			disabled={$isActiveSimulationRunning || !$activeModelId}
			title="Play"
			aria-label="Play simulation"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polygon points="5 3 19 12 5 21 5 3" />
			</svg>
		</button>
		<button
			class="control-btn"
			on:click={() => playMaxSpeed().catch(console.error)}
			disabled={$isActiveSimulationRunning || !$activeModelId}
			title="Play at Max Speed"
			aria-label="Play at maximum speed"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polygon points="5 3 13 12 5 21 5 3" />
				<polygon points="13 3 21 12 13 21 13 3" />
			</svg>
		</button>
		<button
			class="control-btn"
			on:click={() => pause().catch(console.error)}
			disabled={!$isActiveSimulationRunning}
			title="Pause"
			aria-label="Pause simulation"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<rect x="6" y="4" width="4" height="16" />
				<rect x="14" y="4" width="4" height="16" />
			</svg>
		</button>
		<button
			class="control-btn"
			on:click={() => step().catch(console.error)}
			disabled={$isActiveSimulationRunning || !$activeModelId}
			title="Step"
			aria-label="Step simulation"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polygon points="5 4 15 12 5 20 5 4" />
				<line x1="19" y1="5" x2="19" y2="19" />
			</svg>
		</button>
		<button
			class="control-btn"
			on:click={() => reset().catch(console.error)}
			disabled={!$activeModelId}
			title="Reset"
			aria-label="Reset simulation"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
				<path d="M21 3v5h-5" />
				<path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
			</svg>
		</button>
		<button
			class="control-btn"
			on:click={openChart}
			disabled={!$activeModelId}
			title="Open Chart"
			aria-label="Open chart"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<line x1="18" y1="20" x2="18" y2="10" />
				<line x1="12" y1="20" x2="12" y2="4" />
				<line x1="6" y1="20" x2="6" y2="14" />
			</svg>
		</button>
	</div>

	<!-- Settings and Performance -->
	<div class="settings-row">
		<div class="input-group">
			<label for="stepDelay">Delay</label>
			<div class="input-with-unit">
				<input
					id="stepDelay"
					type="number"
					bind:value={$stepDelay}
					min="0"
					disabled={$isActiveSimulationRunning}
				/>
				<span class="unit">ms</span>
			</div>
		</div>

		<!-- <div class="input-group">
			<label for="maxSteps">Max Steps</label>
			<input
				id="maxSteps"
				type="number"
				bind:value={$maxSteps}
				min="1"
				disabled={$isActiveSimulationRunning}
			/>
		</div> -->

		<div class="performance-badge">
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
			</svg>
			<span>{$stepsPerSecond}</span>
			<span class="unit">steps/s</span>
		</div>
	</div>
</div>

<style>
	.controls {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		padding: 1rem;
		background: linear-gradient(135deg, rgba(250, 250, 250, 0.95) 0%, rgba(245, 245, 245, 0.95) 100%);
		border-radius: 12px;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
	}

	/* Control Buttons Row */
	.control-buttons {
		display: flex;
		gap: 0.5rem;
		justify-content: center;
		flex-wrap: wrap;
	}

	.control-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		padding: 0;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
		color: #374151;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
	}

	.control-btn svg {
		width: 20px;
		height: 20px;
		stroke-width: 2.5;
	}

	.control-btn:hover:not(:disabled) {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
		transform: translateY(-1px);
	}

	.control-btn:active:not(:disabled) {
		transform: translateY(0);
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
	}

	.control-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
		background: #f9fafb;
	}

	/* Settings Row */
	.settings-row {
		display: flex;
		gap: 0.75rem;
		align-items: center;
		justify-content: center;
		flex-wrap: wrap;
		padding-top: 0.5rem;
		border-top: 1px solid rgba(0, 0, 0, 0.06);
	}

	/* Input Groups */
	.input-group {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		min-width: 100px;
	}

	.input-group label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.input-with-unit {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 6px;
		padding: 0 0.5rem;
		height: 36px;
	}

	.input-group input,
	.input-with-unit input {
		flex: 1;
		border: none;
		background: transparent;
		color: #111827;
		font-size: 0.875rem;
		font-weight: 500;
		padding: 0.5rem;
		min-width: 0;
		outline: none;
	}

	.input-group > input {
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 6px;
		background: white;
		height: 36px;
	}

	.input-group input:focus,
	.input-group > input:focus {
		outline: 2px solid #3b82f6;
		outline-offset: -1px;
	}

	.input-group input:disabled,
	.input-with-unit input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.unit {
		font-size: 0.75rem;
		color: #9ca3af;
		font-weight: 500;
		white-space: nowrap;
	}

	/* Performance Badge */
	.performance-badge {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
		color: white;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 600;
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.25);
		height: 36px;
	}

	.performance-badge svg {
		width: 18px;
		height: 18px;
		stroke-width: 2.5;
	}

	.performance-badge .unit {
		color: rgba(255, 255, 255, 0.8);
		font-weight: 500;
	}

	/* Responsive Design */
	@media (max-width: 640px) {
		.controls {
			padding: 0.75rem;
			gap: 0.5rem;
		}

		.control-buttons {
			gap: 0.375rem;
		}

		.control-btn {
			width: 40px;
			height: 40px;
		}

		.control-btn svg {
			width: 18px;
			height: 18px;
		}

		.settings-row {
			gap: 0.5rem;
			width: 100%;
		}

		.input-group {
			flex: 1;
			min-width: 80px;
		}

		.performance-badge {
			width: 100%;
			justify-content: center;
		}
	}

	@media (max-width: 480px) {
		.control-buttons {
			justify-content: space-between;
		}

		.control-btn {
			flex: 1;
			min-width: 36px;
			max-width: 52px;
		}

		.settings-row {
			flex-direction: column;
			width: 100%;
		}

		.input-group {
			width: 100%;
		}

		.input-with-unit {
			width: 100%;
		}
	}

	/* Touch device optimization */
	@media (hover: none) and (pointer: coarse) {
		.control-btn {
			width: 48px;
			height: 48px;
		}

		.control-btn svg {
			width: 22px;
			height: 22px;
		}
	}
</style>
