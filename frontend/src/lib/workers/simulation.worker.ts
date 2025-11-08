import init, { Simulation as WasmSimulation } from 'simcraft_web';
import type { SimulationState, Event, ProcessState } from '../simcraft/base';

const MIN_BATCH_SIZE = 10;
const MAX_BATCH_SIZE = Infinity;
const TARGET_FRAME_TIME = 500; // Target FPS = (1000ms / X)
const BATCH_ADJUST_FACTOR = 0.9; // Adjust batch size by (1-X)% each time

let simulation: WasmSimulation | null = null;
let isRunning = false;
let runInterval: number | undefined;
let currentBatchSize = MIN_BATCH_SIZE;
let maxSpeedTimeout: number | undefined;
let wasmInitialized = false;

function adjustBatchSize(executionTime: number): number {
	if (executionTime > TARGET_FRAME_TIME) {
		return Math.max(MIN_BATCH_SIZE, Math.floor(currentBatchSize * BATCH_ADJUST_FACTOR));
	} else if (executionTime < TARGET_FRAME_TIME * 0.95) {
		return Math.min(MAX_BATCH_SIZE, Math.floor(currentBatchSize / BATCH_ADJUST_FACTOR));
	}
	return currentBatchSize;
}

function clearMaxSpeedTimeout() {
	if (maxSpeedTimeout !== undefined) {
		clearTimeout(maxSpeedTimeout);
		maxSpeedTimeout = undefined;
	}
}

async function runMaxSpeedStep() {
	if (!simulation || !isRunning) return;

	const startTime = performance.now();
	try {
		const currentStep = Number(simulation.current_step());
		const targetStep = currentStep + currentBatchSize;

		const events: Event[] = simulation.step_until(targetStep);
		const state = getState();
		self.postMessage({ type: 'stateUpdate', states: [state] });

		const executionTime = performance.now() - startTime;
		currentBatchSize = adjustBatchSize(executionTime);

		// Schedule next batch if still running
		if (isRunning) {
			// Use a small delay to allow message processing
			maxSpeedTimeout = setTimeout(runMaxSpeedStep, 0);
		}
	} catch (error) {
		console.error('Max speed step error:', error);
		isRunning = false;
		self.postMessage({ type: 'error', error: String(error) });
	}
}

async function initializeWasm() {
	if (wasmInitialized) return;
	try {
		await init();
		wasmInitialized = true;
		console.log('WASM initialized successfully');
	} catch (error) {
		console.error('Failed to initialize WASM:', error);
		throw error;
	}
}

initializeWasm()
	.then(() => {
		setupMessageHandler();
	})
	.catch(error => {
		console.error('Error initializing WASM:', error);
		self.postMessage({
			type: 'error',
			error: `WASM initialization failed: ${error instanceof Error ? error.message : String(error)}`
		});
	});

function setupMessageHandler() {
	self.postMessage({ type: 'ready' });

	self.onmessage = async (e: MessageEvent) => {
		try {
			const { type, ...data } = e.data;

			switch (type) {
				case 'initialise': {
					const { processes, connections } = data;
					console.log('initialise', processes, connections);
					simulation = WasmSimulation.new(JSON.stringify(processes), JSON.stringify(connections));
					currentBatchSize = MIN_BATCH_SIZE;
					const state = getState();
					self.postMessage({ type: 'initialise', state });
					break;
				}

				case 'step': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const events: Event[] = simulation.step();
					const state = getState();
					self.postMessage({ type: 'stateUpdate', states: [state] });
					self.postMessage({ type: 'step', events, state });
					break;
				}

				case 'stepUntil': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const { until } = data;
					const events: Event[] = simulation.step_until(until);
					const state = getState();
					self.postMessage({ type: 'stateUpdate', states: [state] });
					self.postMessage({ type: 'stepUntil', events, state });
					break;
				}

				case 'play': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const { stepDelay } = data;
					isRunning = true;
					if (stepDelay === 0) {
						// Run at max speed using batch processing
						runMaxSpeedStep();
					} else {
						// Run with fixed delay
						runInterval = setInterval(() => {
							if (!isRunning) {
								stopInterval();
								return;
							}

							try {
								const events: Event[] = simulation!.step();
								const state = getState();
								self.postMessage({ type: 'stateUpdate', states: [state] });
							} catch (error) {
								console.error('Error in continuous simulation:', error);
								stopInterval();
								isRunning = false;
								self.postMessage({ type: 'error', error: String(error) });
							}
						}, stepDelay);
					}

					self.postMessage({ type: 'play', success: true });
					break;
				}

				case 'pause': {
					isRunning = false;
					clearMaxSpeedTimeout();
					stopInterval();
					self.postMessage({ type: 'pause', success: true });
					break;
				}

				case 'reset': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					simulation.reset();
					currentBatchSize = MIN_BATCH_SIZE;
					self.postMessage({ type: 'reset', success: true });
					break;
				}

				case 'getState': {
					const state = getState();
					self.postMessage({ type: 'getState', states: [state] });
					break;
				}

				case 'addProcess': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					console.log('addProcess', data);
					const { process } = data;
					simulation.add_process(JSON.stringify(process));
					self.postMessage({ type: 'addProcess', success: true });
					break;
				}

				case 'removeProcess': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					console.log('removeProcess', data);
					const { processId } = data;
					simulation.remove_process(processId);
					self.postMessage({ type: 'removeProcess', success: true });
					break;
				}

				case 'getProcesses': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const processes = simulation.get_processes();
					self.postMessage({ type: 'getProcesses', processes });
					break;
				}

				case 'updateProcess': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const { processId, process } = data;
					try {
						simulation.update_process(processId, JSON.stringify(process));
						self.postMessage({ type: 'updateProcess', success: true });
					} catch (error) {
						console.error('Failed to update process in simulation:', error);
						self.postMessage({
							type: 'updateProcess',
							success: false,
							error: error instanceof Error ? error.message : String(error)
						});
					}
					break;
				}

				case 'addConnection': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					console.log('addConnection', data);
					const { connection } = data;
					simulation.add_connection(JSON.stringify(connection));
					self.postMessage({ type: 'addConnection', success: true });
					break;
				}

				case 'removeConnection': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					console.log('removeConnection', data);
					const { connectionId } = data;
					simulation.remove_connection(connectionId);
					self.postMessage({ type: 'removeConnection', success: true });
					break;
				}

				case 'updateConnection': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const { connectionId, connection } = data;
					try {
						simulation.update_connection(connectionId, JSON.stringify(connection));
						self.postMessage({ type: 'updateConnection', success: true });
					} catch (error) {
						console.error('Failed to update connection in simulation:', error);
						self.postMessage({
							type: 'updateConnection',
							success: false,
							error: error instanceof Error ? error.message : String(error)
						});
					}
					break;
				}

				case 'getCurrentStep': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const step = simulation.current_step();
					self.postMessage({ type: 'getCurrentStep', step: Number(step) });
					break;
				}

				case 'getCurrentTime': {
					if (!simulation) {
						throw new Error('Simulation not initialised');
					}
					const time = simulation.current_time();
					self.postMessage({ type: 'getCurrentTime', time: Number(time) });
					break;
				}

				default:
					throw new Error(`Unknown message type: ${type}`);
			}
		} catch (error) {
			console.error('Worker error:', error);
			self.postMessage({
				type: 'error',
				error: error instanceof Error ? error.message : String(error)
			});
		}
	};
}

function getState(): SimulationState {
	if (!simulation) {
		throw new Error('Simulation not initialised');
	}

	const raw_state = simulation.get_simulation_state();
	return {
		...raw_state,
		process_states: Object.fromEntries(raw_state.process_states) as Record<string, ProcessState>
	};
}

function stopInterval() {
	if (runInterval !== undefined) {
		clearInterval(runInterval);
		runInterval = undefined;
	}
}
