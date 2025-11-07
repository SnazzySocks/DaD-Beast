<script lang="ts">
	import { humorMode } from '$lib/stores/humor';
	import { fade } from 'svelte/transition';

	export let variant: 'default' | 'compact' = 'default';
	export let showLabel = true;

	let isAnimating = false;

	function handleToggle() {
		isAnimating = true;
		humorMode.toggle();
		setTimeout(() => {
			isAnimating = false;
		}, 300);
	}
</script>

{#if variant === 'default'}
	<div class="humor-toggle-container" in:fade={{ duration: 200 }}>
		<button
			on:click={handleToggle}
			class="humor-toggle-button"
			class:animating={isAnimating}
			aria-label="Toggle humor mode"
			title={$humorMode === 'dad' ? 'Switch to normal mode' : 'Switch to dad humor mode'}
		>
			<div class="toggle-track" class:active={$humorMode === 'dad'}>
				<div class="toggle-thumb">
					<span class="toggle-icon">
						{#if $humorMode === 'dad'}
							😏
						{:else}
							😊
						{/if}
					</span>
				</div>
			</div>
			{#if showLabel}
				<span class="toggle-label">
					{$humorMode === 'dad' ? 'Dark Dad Humor' : 'Normal Mode'}
				</span>
			{/if}
		</button>
		{#if showLabel}
			<p class="toggle-description">
				{$humorMode === 'dad'
					? 'Experiencing the resentful dad tone. Toggle for normal text.'
					: 'Using normal, professional text. Toggle for dark dad humor.'}
			</p>
		{/if}
	</div>
{:else}
	<button
		on:click={handleToggle}
		class="humor-toggle-compact"
		class:animating={isAnimating}
		aria-label="Toggle humor mode"
		title={$humorMode === 'dad' ? 'Switch to normal mode' : 'Switch to dad humor mode'}
	>
		<div class="compact-track" class:active={$humorMode === 'dad'}>
			<div class="compact-thumb">
				<span class="compact-icon">
					{#if $humorMode === 'dad'}
						😏
					{:else}
						😊
					{/if}
				</span>
			</div>
		</div>
		{#if showLabel}
			<span class="compact-label">
				{$humorMode === 'dad' ? 'Dad' : 'Normal'}
			</span>
		{/if}
	</button>
{/if}

<style>
	/* Default variant styles */
	.humor-toggle-container {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		padding: 1rem;
		background: var(--card-background, #ffffff);
		border: 1px solid var(--border-color, #e5e7eb);
		border-radius: 12px;
		margin-bottom: 1.5rem;
		transition: all 0.3s ease;
	}

	.humor-toggle-button {
		display: flex;
		align-items: center;
		gap: 1rem;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		text-align: left;
		width: 100%;
	}

	.humor-toggle-button.animating {
		animation: pulse 0.3s ease;
	}

	@keyframes pulse {
		0%, 100% {
			transform: scale(1);
		}
		50% {
			transform: scale(0.98);
		}
	}

	.toggle-track {
		position: relative;
		width: 56px;
		height: 32px;
		background: #d1d5db;
		border-radius: 16px;
		transition: background 0.3s ease;
		flex-shrink: 0;
	}

	.toggle-track.active {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
	}

	.toggle-thumb {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 28px;
		height: 28px;
		background: white;
		border-radius: 50%;
		transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.toggle-track.active .toggle-thumb {
		transform: translateX(24px);
	}

	.toggle-icon {
		font-size: 1rem;
		line-height: 1;
		user-select: none;
	}

	.toggle-label {
		font-size: 1rem;
		font-weight: 600;
		color: var(--text-primary, #111827);
		flex: 1;
	}

	.toggle-description {
		font-size: 0.875rem;
		color: var(--text-muted, #6b7280);
		margin: 0;
		padding-left: 72px;
	}

	/* Compact variant styles */
	.humor-toggle-compact {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: none;
		border: 1px solid var(--border-color, #e5e7eb);
		border-radius: 8px;
		padding: 0.5rem 0.75rem;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.humor-toggle-compact:hover {
		background: var(--hover-background, #f9fafb);
		border-color: var(--border-hover, #d1d5db);
	}

	.humor-toggle-compact.animating {
		animation: pulse 0.3s ease;
	}

	.compact-track {
		position: relative;
		width: 44px;
		height: 24px;
		background: #d1d5db;
		border-radius: 12px;
		transition: background 0.3s ease;
		flex-shrink: 0;
	}

	.compact-track.active {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
	}

	.compact-thumb {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.compact-track.active .compact-thumb {
		transform: translateX(20px);
	}

	.compact-icon {
		font-size: 0.75rem;
		line-height: 1;
		user-select: none;
	}

	.compact-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary, #111827);
	}

	/* Dark mode support */
	:global(.dark) .humor-toggle-container {
		background: var(--card-background-dark, #1f2937);
		border-color: var(--border-color-dark, #374151);
	}

	:global(.dark) .toggle-label,
	:global(.dark) .compact-label {
		color: var(--text-primary-dark, #f9fafb);
	}

	:global(.dark) .toggle-description {
		color: var(--text-muted-dark, #9ca3af);
	}

	:global(.dark) .humor-toggle-compact {
		border-color: var(--border-color-dark, #374151);
	}

	:global(.dark) .humor-toggle-compact:hover {
		background: var(--hover-background-dark, #111827);
		border-color: var(--border-hover-dark, #4b5563);
	}

	/* Responsive */
	@media (max-width: 640px) {
		.toggle-description {
			padding-left: 0;
			margin-top: 0.25rem;
		}

		.humor-toggle-button {
			flex-direction: column;
			align-items: flex-start;
			gap: 0.75rem;
		}

		.toggle-label {
			padding-left: 0;
		}
	}
</style>
