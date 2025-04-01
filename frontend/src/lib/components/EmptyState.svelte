<script lang="ts">
	import type { ModelMetadata } from '$lib/stores/simulation';
	import { createNewModel, loadRecentModels, openModel } from '$lib/stores/modelManager';

	let recentModels: ModelMetadata[] = [];

	$: {
		loadRecentModels().then((models) => {
			recentModels = models;
		});
	}
</script>

<div class="empty-state">
	<div class="content">
		<h2 class="title">Welcome to Simcraft</h2>
		<p class="description">Create a new model or open a recent one to get started</p>

		<button
			class="new-model-button bg-accent/20 text-primary hover:bg-accent/30 border-accent/30 border"
			on:click={createNewModel}
		>
			New Model
		</button>

		{#if recentModels.length > 0}
			<div class="recent-models">
				<h3 class="recent-title">Recent Models</h3>
				<div class="models-list">
					{#each recentModels as model}
						<button
							class="model-item hover:bg-accent/10 text-primary"
							on:click={() => openModel(model)}
						>
							<span class="model-name">{model.name}</span>
							<span class="model-date text-secondary text-sm">
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
		background-color: var(--bg-primary);
	}

	.content {
		text-align: center;
		max-width: 400px;
		padding: 2rem;
	}

	.title {
		font-size: 1.5rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
		color: var(--text-primary);
	}

	.description {
		color: var(--text-secondary);
		margin-bottom: 2rem;
	}

	.new-model-button {
		padding: 0.75rem 1.5rem;
		border-radius: 4px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.3s ease;
		margin-bottom: 2rem;
	}

	.recent-models {
		text-align: left;
	}

	.recent-title {
		font-size: 0.9rem;
		font-weight: 500;
		color: var(--text-secondary);
		margin-bottom: 0.5rem;
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
		padding: 0.75rem;
		border-radius: 4px;
		cursor: pointer;
		transition: all 0.3s ease;
		text-align: left;
		border: none;
		background: none;
		width: 100%;
	}

	.model-name {
		font-weight: 500;
	}
</style>
