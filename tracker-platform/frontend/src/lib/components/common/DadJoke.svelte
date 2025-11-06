<script lang="ts">
	import { query } from '@urql/svelte';
	import { DAD_JOKE_QUERY } from '$lib/graphql/queries';
	import { onMount } from 'svelte';
	import { fade, fly } from 'svelte/transition';

	export let variant: 'default' | 'compact' = 'default';
	export let showRefresh = true;

	let jokeKey = 0;
	let isRefreshing = false;

	const result = query({
		query: DAD_JOKE_QUERY
	});

	async function refreshJoke() {
		if (isRefreshing) return;

		isRefreshing = true;
		jokeKey += 1;

		$result.reexecute({ requestPolicy: 'network-only' });

		setTimeout(() => {
			isRefreshing = false;
		}, 500);
	}

	onMount(() => {
		const interval = setInterval(() => {
			refreshJoke();
		}, 60000);

		return () => clearInterval(interval);
	});
</script>

{#if variant === 'default'}
	<div class="dad-joke-container" in:fade={{ duration: 300 }}>
		<div class="joke-header">
			<div class="joke-icon">😄</div>
			<h3 class="joke-title">Dad Joke of the Moment</h3>
			{#if showRefresh}
				<button
					on:click={refreshJoke}
					class="refresh-button"
					disabled={isRefreshing}
					aria-label="Get new joke"
				>
					<svg
						class:spinning={isRefreshing}
						xmlns="http://www.w3.org/2000/svg"
						width="18"
						height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<polyline points="23 4 23 10 17 10"></polyline>
						<polyline points="1 20 1 14 7 14"></polyline>
						<path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
					</svg>
				</button>
			{/if}
		</div>

		{#if $result.fetching}
			<div class="joke-loading" in:fade={{ duration: 200 }}>
				<div class="loading-spinner"></div>
				<p>Loading a fresh joke...</p>
			</div>
		{:else if $result.error}
			<div class="joke-error" in:fade={{ duration: 200 }}>
				<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<circle cx="12" cy="12" r="10"></circle>
					<line x1="12" y1="8" x2="12" y2="12"></line>
					<line x1="12" y1="16" x2="12.01" y2="16"></line>
				</svg>
				<p>Couldn't load a joke right now. Try again later!</p>
			</div>
		{:else if $result.data?.dadJoke}
			{#key jokeKey}
				<div class="joke-content" in:fly={{ y: 10, duration: 400, delay: 100 }}>
					<p class="joke-text">{$result.data.dadJoke.joke}</p>
				</div>
			{/key}
		{/if}
	</div>
{:else}
	<div class="dad-joke-compact" in:fade={{ duration: 300 }}>
		{#if $result.fetching}
			<div class="compact-loading">
				<div class="compact-spinner"></div>
				<span>Loading joke...</span>
			</div>
		{:else if $result.error}
			<div class="compact-error">
				<span>😅 Joke unavailable</span>
			</div>
		{:else if $result.data?.dadJoke}
			{#key jokeKey}
				<div class="compact-content" in:fly={{ x: -10, duration: 300 }}>
					<span class="compact-icon">😄</span>
					<p class="compact-text">{$result.data.dadJoke.joke}</p>
					{#if showRefresh}
						<button
							on:click={refreshJoke}
							class="compact-refresh"
							disabled={isRefreshing}
							aria-label="Get new joke"
						>
							<svg
								class:spinning={isRefreshing}
								xmlns="http://www.w3.org/2000/svg"
								width="14"
								height="14"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<polyline points="23 4 23 10 17 10"></polyline>
								<polyline points="1 20 1 14 7 14"></polyline>
								<path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
							</svg>
						</button>
					{/if}
				</div>
			{/key}
		{/if}
	</div>
{/if}

<style>
	.dad-joke-container {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1), 0 1px 3px rgba(0, 0, 0, 0.08);
		margin-bottom: 1.5rem;
		transition: transform 0.2s ease, box-shadow 0.2s ease;
	}

	.dad-joke-container:hover {
		transform: translateY(-2px);
		box-shadow: 0 8px 12px rgba(0, 0, 0, 0.15), 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	.joke-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}

	.joke-icon {
		font-size: 1.75rem;
		line-height: 1;
	}

	.joke-title {
		color: white;
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0;
		flex: 1;
	}

	.refresh-button {
		background: rgba(255, 255, 255, 0.2);
		border: none;
		border-radius: 8px;
		padding: 0.5rem;
		cursor: pointer;
		color: white;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.2s ease, transform 0.2s ease;
	}

	.refresh-button:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.3);
		transform: scale(1.05);
	}

	.refresh-button:disabled {
		cursor: not-allowed;
		opacity: 0.6;
	}

	.refresh-button svg {
		transition: transform 0.3s ease;
	}

	.refresh-button svg.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.joke-loading,
	.joke-error {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		color: white;
		padding: 1rem 0;
	}

	.loading-spinner {
		width: 20px;
		height: 20px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.joke-loading p,
	.joke-error p {
		margin: 0;
		font-size: 0.875rem;
		opacity: 0.95;
	}

	.joke-content {
		background: rgba(255, 255, 255, 0.15);
		border-radius: 8px;
		padding: 1.25rem;
		backdrop-filter: blur(10px);
	}

	.joke-text {
		color: white;
		font-size: 1rem;
		line-height: 1.6;
		margin: 0;
		font-style: italic;
	}

	.dad-joke-compact {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		border-radius: 8px;
		padding: 0.75rem 1rem;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		margin-bottom: 1rem;
	}

	.compact-loading,
	.compact-error {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: white;
		font-size: 0.875rem;
	}

	.compact-spinner {
		width: 14px;
		height: 14px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.compact-content {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.compact-icon {
		font-size: 1.25rem;
		line-height: 1;
		flex-shrink: 0;
	}

	.compact-text {
		color: white;
		font-size: 0.875rem;
		line-height: 1.4;
		margin: 0;
		flex: 1;
		font-style: italic;
	}

	.compact-refresh {
		background: rgba(255, 255, 255, 0.2);
		border: none;
		border-radius: 6px;
		padding: 0.375rem;
		cursor: pointer;
		color: white;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.2s ease;
		flex-shrink: 0;
	}

	.compact-refresh:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.3);
	}

	.compact-refresh:disabled {
		cursor: not-allowed;
		opacity: 0.6;
	}

	@media (max-width: 640px) {
		.dad-joke-container {
			padding: 1.25rem;
		}

		.joke-title {
			font-size: 1rem;
		}

		.joke-text {
			font-size: 0.9375rem;
		}

		.compact-text {
			font-size: 0.8125rem;
		}
	}
</style>
