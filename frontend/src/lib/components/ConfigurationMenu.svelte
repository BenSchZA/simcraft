<script lang="ts">
	import type {
		ProcessNode,
		ConnectionEdge,
		SourceSettings,
		PoolSettings,
		DrainSettings,
		DelaySettings,
		StepperSettings,
		ProcessSettingsType,
		ConnectionSettings
	} from '$lib/simcraft/base';
	import {
		ProcessType,
		TriggerMode,
		Action,
		DelayAction,
		Overflow,
		isSourceSettings,
		isPoolSettings,
		isDrainSettings,
		isDelaySettings,
		isStepperSettings,
		getValidActionsForProcessType,
		getValidDelayActions
	} from '$lib/simcraft/base';
	import {
		updateProcessCommand,
		updateConnectionCommand
	} from '$lib/stores/simulationManager';

	export let selectedElement: ProcessNode | ConnectionEdge | null = null;

	$: isProcess = selectedElement && 'type' in selectedElement;
	$: isConnection = selectedElement && !isProcess;

	type ProcessNodeWithSettings<T extends ProcessSettingsType> = ProcessNode & {
		data: { settings: T };
	};

	function isProcessNode(element: ProcessNode | ConnectionEdge | null): element is ProcessNode {
		return element !== null && 'type' in element;
	}

	function createSourceSettingsUpdate(
		field: keyof Omit<SourceSettings, 'type' | 'id'>,
		value: string | number
	) {
		return { [field]: value } as Partial<Omit<SourceSettings, 'type' | 'id'>>;
	}

	function createPoolSettingsUpdate(
		field: keyof Omit<PoolSettings, 'type' | 'id'>,
		value: string | number
	) {
		return { [field]: value } as Partial<Omit<PoolSettings, 'type' | 'id'>>;
	}

	function createDrainSettingsUpdate(
		field: keyof Omit<DrainSettings, 'type' | 'id'>,
		value: string | number
	) {
		return { [field]: value } as Partial<Omit<DrainSettings, 'type' | 'id'>>;
	}

	function createDelaySettingsUpdate(
		field: keyof Omit<DelaySettings, 'type' | 'id'>,
		value: string | number
	) {
		return { [field]: value } as Partial<Omit<DelaySettings, 'type' | 'id'>>;
	}

	function createConnectionSettingsUpdate(
		field: keyof Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>,
		value: string | number
	) {
		return { [field]: value } as Partial<Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>>;
	}

	async function handleSourceUpdate(
		event: Event,
		field: keyof Omit<SourceSettings, 'type' | 'id'>
	) {
		if (!isProcessNode(selectedElement) || !isSourceSettings(selectedElement.data.settings)) return;
		const target = event.target as HTMLInputElement | HTMLSelectElement;
		const value = target.type === 'number' ? parseFloat(target.value) : target.value;
		const update = { [field]: value } as Partial<Omit<SourceSettings, 'type' | 'id'>>;
		try {
			await updateProcessCommand(selectedElement.id, update);
		} catch (error) {
			console.error('Failed to update source settings:', error);
		}
	}

	async function handlePoolUpdate(event: Event, field: keyof Omit<PoolSettings, 'type' | 'id'>) {
		if (!isProcessNode(selectedElement) || !isPoolSettings(selectedElement.data.settings)) return;
		const target = event.target as HTMLInputElement | HTMLSelectElement;
		const value = target.type === 'number' ? parseFloat(target.value) : target.value;
		const update = { [field]: value } as Partial<Omit<PoolSettings, 'type' | 'id'>>;
		try {
			await updateProcessCommand(selectedElement.id, update);
		} catch (error) {
			console.error('Failed to update pool settings:', error);
		}
	}

	async function handleDrainUpdate(event: Event, field: keyof Omit<DrainSettings, 'type' | 'id'>) {
		if (!isProcessNode(selectedElement) || !isDrainSettings(selectedElement.data.settings)) return;
		const target = event.target as HTMLInputElement | HTMLSelectElement;
		const value = target.type === 'number' ? parseFloat(target.value) : target.value;
		const update = { [field]: value } as Partial<Omit<DrainSettings, 'type' | 'id'>>;
		try {
			await updateProcessCommand(selectedElement.id, update);
		} catch (error) {
			console.error('Failed to update drain settings:', error);
		}
	}

	async function handleDelayUpdate(event: Event, field: keyof Omit<DelaySettings, 'type' | 'id'>) {
		if (!isProcessNode(selectedElement) || !isDelaySettings(selectedElement.data.settings)) return;
		const target = event.target as HTMLInputElement | HTMLSelectElement;
		const value = target.type === 'number' ? parseFloat(target.value) : target.value;
		const update = { [field]: value } as Partial<Omit<DelaySettings, 'type' | 'id'>>;
		try {
			await updateProcessCommand(selectedElement.id, update);
		} catch (error) {
			console.error('Failed to update delay settings:', error);
		}
	}

	async function handleStepperUpdate(
		event: Event,
		field: keyof Omit<StepperSettings, 'type' | 'id'>
	) {
		if (!isProcessNode(selectedElement) || !isStepperSettings(selectedElement.data.settings))
			return;
		const target = event.target as HTMLInputElement | HTMLSelectElement;
		const value = target.type === 'number' ? parseFloat(target.value) : target.value;
		const update = { [field]: value } as Partial<Omit<StepperSettings, 'type' | 'id'>>;
		try {
			await updateProcessCommand(selectedElement.id, update);
		} catch (error) {
			console.error('Failed to update stepper settings:', error);
		}
	}

	async function handleConnectionUpdate(
		event: Event,
		field: keyof Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>
	) {
		if (isProcessNode(selectedElement)) return;
		const edge = selectedElement as ConnectionEdge;
		const target = event.target as HTMLInputElement | HTMLSelectElement;
		const value = target.type === 'number' ? parseFloat(target.value) : target.value;
		const update = { [field]: value } as Partial<
			Omit<ConnectionSettings, 'id' | 'sourceId' | 'targetId'>
		>;
		try {
			await updateConnectionCommand(edge.id, update);
		} catch (error) {
			console.error('Failed to update connection settings:', error);
		}
	}
</script>

<div class="config-menu">
	{#if selectedElement}
		<h3>{isProcess ? 'Process Settings' : 'Connection Settings'}</h3>

		{#if isProcessNode(selectedElement)}
			<div class="settings-group">
				<p class="type-label">Type: {selectedElement.type}</p>

				{#if selectedElement.data?.settings && isSourceSettings(selectedElement.data.settings)}
					<div class="setting">
						<label for="triggerMode">Trigger Mode:</label>
						<select
							id="triggerMode"
							value={selectedElement.data.settings.triggerMode}
							on:change={(e) => handleSourceUpdate(e, 'triggerMode')}
						>
							{#each Object.values(TriggerMode) as mode}
								<option value={mode}>{mode}</option>
							{/each}
						</select>
					</div>
					<div class="setting">
						<label for="action">Action:</label>
						<select
							id="action"
							value={selectedElement.data.settings.action}
							on:change={(e) => handleSourceUpdate(e, 'action')}
						>
							{#each getValidActionsForProcessType(ProcessType.Source) as action}
								<option value={action}>{action}</option>
							{/each}
						</select>
						<small class="help-text">Sources can only push resources to downstream processes</small>
					</div>
				{:else if selectedElement.data?.settings && isPoolSettings(selectedElement.data.settings)}
					<div class="setting">
						<label for="triggerMode">Trigger Mode:</label>
						<select
							id="triggerMode"
							value={selectedElement.data.settings.triggerMode}
							on:change={(e) => handlePoolUpdate(e, 'triggerMode')}
						>
							{#each Object.values(TriggerMode) as mode}
								<option value={mode}>{mode}</option>
							{/each}
						</select>
					</div>
					<div class="setting">
						<label for="action">Action:</label>
						<select
							id="action"
							value={selectedElement.data.settings.action}
							on:change={(e) => handlePoolUpdate(e, 'action')}
						>
							{#each getValidActionsForProcessType(ProcessType.Pool) as action}
								<option value={action}>{action}</option>
							{/each}
						</select>
						<small class="help-text">Pools can push or pull resources in different modes</small>
					</div>
					<div class="setting">
						<label for="overflow">Overflow:</label>
						<select
							id="overflow"
							value={selectedElement.data.settings.overflow}
							on:change={(e) => handlePoolUpdate(e, 'overflow')}
						>
							{#each Object.values(Overflow) as overflow}
								<option value={overflow}>{overflow}</option>
							{/each}
						</select>
					</div>
					<div class="setting">
						<label for="capacity">Capacity:</label>
						<input
							type="number"
							id="capacity"
							value={selectedElement.data.settings.capacity}
							on:change={(e) => handlePoolUpdate(e, 'capacity')}
						/>
					</div>
				{:else if selectedElement.data?.settings && isDrainSettings(selectedElement.data.settings)}
					<div class="setting">
						<label for="triggerMode">Trigger Mode:</label>
						<select
							id="triggerMode"
							value={selectedElement.data.settings.triggerMode}
							on:change={(e) => handleDrainUpdate(e, 'triggerMode')}
						>
							{#each Object.values(TriggerMode) as mode}
								<option value={mode}>{mode}</option>
							{/each}
						</select>
					</div>
					<div class="setting">
						<label for="action">Action:</label>
						<select
							id="action"
							value={selectedElement.data.settings.action}
							on:change={(e) => handleDrainUpdate(e, 'action')}
						>
							{#each getValidActionsForProcessType(ProcessType.Drain) as action}
								<option value={action}>{action}</option>
							{/each}
						</select>
						<small class="help-text">Drains can only pull resources from upstream processes</small>
					</div>
				{:else if selectedElement.data?.settings && isDelaySettings(selectedElement.data.settings)}
					<div class="setting">
						<label for="triggerMode">Trigger Mode:</label>
						<select
							id="triggerMode"
							value={selectedElement.data.settings.triggerMode}
							on:change={(e) => handleDelayUpdate(e, 'triggerMode')}
						>
							{#each Object.values(TriggerMode) as mode}
								<option value={mode}>{mode}</option>
							{/each}
						</select>
					</div>
					<div class="setting">
						<label for="action">Action:</label>
						<select
							id="action"
							value={selectedElement.data.settings.action}
							on:change={(e) => handleDelayUpdate(e, 'action')}
						>
							{#each getValidDelayActions() as action}
								<option value={action}>{action}</option>
							{/each}
						</select>
					</div>
					<div class="setting">
						<label for="releaseAmount">Release Amount:</label>
						<input
							type="number"
							id="releaseAmount"
							value={selectedElement.data.settings.releaseAmount}
							on:change={(e) => handleDelayUpdate(e, 'releaseAmount')}
						/>
					</div>
				{:else if selectedElement.data?.settings && isStepperSettings(selectedElement.data.settings)}
					<div class="setting">
						<label for="triggerMode">Trigger Mode:</label>
						<select id="triggerMode" value={selectedElement.data.settings.triggerMode} disabled>
							<option value={TriggerMode.Automatic}>Automatic (Fixed)</option>
						</select>
					</div>
					<div class="setting">
						<label for="dt">Time Step (dt):</label>
						<input
							type="number"
							id="dt"
							value={selectedElement.data.settings.dt}
							on:change={(e) => handleStepperUpdate(e, 'dt')}
							min="0.1"
							step="0.1"
						/>
					</div>
				{/if}
			</div>
		{:else if selectedElement && !isProcessNode(selectedElement)}
			<div class="settings-group">
				<div class="setting">
					<label for="flowRate">Flow Rate:</label>
					<input
						type="number"
						id="flowRate"
						value={selectedElement.data.settings.flowRate}
						on:change={(e) => handleConnectionUpdate(e, 'flowRate')}
					/>
				</div>
				<div class="setting">
					<label for="sourcePort">Source Port:</label>
					<input
						type="text"
						id="sourcePort"
						value={selectedElement.data.settings.sourcePort}
						on:change={(e) => handleConnectionUpdate(e, 'sourcePort')}
					/>
				</div>
				<div class="setting">
					<label for="targetPort">Target Port:</label>
					<input
						type="text"
						id="targetPort"
						value={selectedElement.data.settings.targetPort}
						on:change={(e) => handleConnectionUpdate(e, 'targetPort')}
					/>
				</div>
			</div>
		{/if}
	{:else}
		<div class="config-menu">
			<p>Select a process or connection to configure</p>
		</div>
	{/if}
</div>

<style>
	.config-menu {
		background: transparent;
		padding: 0;
		max-width: none;
	}

	.settings-group {
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
	}

	.setting {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.type-label {
		font-weight: 600;
		margin-bottom: 1.25rem;
		color: #374151;
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		border-bottom: 1px solid rgba(0, 0, 0, 0.08);
		padding-bottom: 0.625rem;
	}

	label {
		font-weight: 500;
		color: #6b7280;
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	input,
	select {
		padding: 0.625rem 0.75rem;
		border: 1px solid rgba(0, 0, 0, 0.1);
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s ease;
		background: white;
		color: #111827;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
	}

	input:hover,
	select:hover {
		border-color: rgba(0, 0, 0, 0.15);
	}

	input:focus,
	select:focus {
		outline: 2px solid #3b82f6;
		outline-offset: -1px;
		border-color: transparent;
		box-shadow: 0 2px 8px rgba(59, 130, 246, 0.15);
	}

	input[type='number'] {
		width: 100%;
	}

	select {
		width: 100%;
		cursor: pointer;
	}

	select:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		background: #f9fafb;
	}

	h3 {
		margin: 0 0 1.5rem 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		letter-spacing: -0.01em;
	}

	.help-text {
		color: #9ca3af;
		font-size: 0.75rem;
		margin-top: 0.375rem;
		display: block;
		font-style: italic;
		line-height: 1.5;
		padding: 0.5rem 0.75rem;
		background: rgba(59, 130, 246, 0.04);
		border-left: 2px solid rgba(59, 130, 246, 0.3);
		border-radius: 4px;
	}
</style>
