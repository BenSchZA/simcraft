<script lang="ts">
	import type { ModelMetadata } from '$lib/stores/simulation';
	import {
		createNewModel,
		loadRecentModels,
		openModel,
		getExampleModels,
		loadExampleModel,
		type ExampleModel
	} from '$lib/stores/modelManager';

	let recentModels: ModelMetadata[] = [];
	let exampleModels: ExampleModel[] = [];

	$: {
		loadRecentModels().then((models) => {
			recentModels = models;
		});
		exampleModels = getExampleModels();
	}
</script>

<div class="empty-state">
	<div class="content">
		<h2 class="title">Welcome to Simcraft</h2>
		<p class="description">Create a new model or open a recent one to get started</p>

		<button class="new-model-button" on:click={createNewModel}>New Model</button>

		{#if exampleModels.length > 0}
			<div class="example-models">
				<h3 class="section-title">Example Models</h3>
				<p class="section-description">
					Start with a pre-built example to explore Simcraft features
				</p>
				<div class="models-list">
					{#each exampleModels as example}
						<button class="model-item example-item" on:click={() => loadExampleModel(example)}>
							<div class="model-info">
								<span class="model-name">{example.name}</span>
								<span class="model-description">
									{example.description}
								</span>
							</div>
							<span class="load-button">Load</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}

		{#if recentModels.length > 0}
			<div class="recent-models">
				<h3 class="section-title">Recent Models</h3>
				<div class="models-list">
					{#each recentModels as model}
						<button class="model-item" on:click={() => openModel(model)}>
							<span class="model-name">{model.name}</span>
							<span class="model-date">
								{new Date(model.lastModified).toLocaleDateString()}
							</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.empty-state {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: linear-gradient(135deg, #fafafa 0%, #f5f5f5 100%);
	}

	.content {
		text-align: center;
		max-width: 600px;
		padding: 2.5rem;
	}

	.title {
		font-size: 1.875rem;
		font-weight: 600;
		margin-bottom: 0.75rem;
		color: #111827;
		letter-spacing: -0.02em;
	}

	.description {
		color: #6b7280;
		margin-bottom: 2rem;
		font-size: 1rem;
	}

	.new-model-button {
		padding: 0.875rem 2rem;
		border-radius: 8px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		margin-bottom: 2.5rem;
		background: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
		color: #374151;
		font-size: 0.9375rem;
	}

	.new-model-button:hover {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
		box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
		transform: translateY(-1px);
	}

	.new-model-button:active {
		transform: translateY(0);
		box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
	}

	.example-models,
	.recent-models {
		text-align: left;
		margin-bottom: 2rem;
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: #374151;
		margin-bottom: 0.625rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.section-description {
		font-size: 0.875rem;
		color: #9ca3af;
		margin-bottom: 1rem;
		line-height: 1.5;
	}

	.models-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.model-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.875rem 1rem;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: left;
		border: 1px solid rgba(0, 0, 0, 0.06);
		background: #fafafa;
		width: 100%;
	}

	.model-item:hover {
		background: rgba(59, 130, 246, 0.08);
		border-color: rgba(59, 130, 246, 0.2);
		box-shadow: 0 2px 8px rgba(59, 130, 246, 0.1);
	}

	.model-name {
		font-weight: 500;
		color: #111827;
		font-size: 0.9375rem;
	}

	.example-item {
		border-radius: 8px;
		padding: 1rem;
	}

	.model-info {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
		flex: 1;
	}

	.model-description {
		font-size: 0.8125rem;
		line-height: 1.4;
		color: #6b7280;
	}

	.load-button {
		font-weight: 500;
		white-space: nowrap;
		color: #3b82f6;
		font-size: 0.875rem;
	}

	.model-item:hover .load-button {
		color: #2563eb;
	}

	.model-date {
		color: #9ca3af;
	}
</style>
