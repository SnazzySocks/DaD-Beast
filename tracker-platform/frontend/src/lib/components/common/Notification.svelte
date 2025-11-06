<script lang="ts">
	import { notifications } from '$lib/stores/notifications';
	import { fly } from 'svelte/transition';

	const icons = {
		success: '✓',
		error: '✕',
		warning: '⚠',
		info: 'ℹ'
	};

	const colors = {
		success: 'bg-green-500',
		error: 'bg-red-500',
		warning: 'bg-yellow-500',
		info: 'bg-blue-500'
	};
</script>

<div class="fixed top-4 right-4 z-50 space-y-2 max-w-sm">
	{#each $notifications as notification (notification.id)}
		<div
			transition:fly={{ x: 300, duration: 300 }}
			class="bg-surface border border-theme rounded-lg shadow-lg p-4 flex items-start space-x-3 animate-fade-in"
		>
			<div class="{colors[notification.type]} text-white w-6 h-6 rounded-full flex items-center justify-center flex-shrink-0">
				<span class="text-sm font-bold">{icons[notification.type]}</span>
			</div>

			<div class="flex-1 min-w-0">
				<p class="text-sm text-primary break-words">
					{notification.message}
				</p>
			</div>

			<button
				on:click={() => notifications.remove(notification.id)}
				class="text-muted hover:text-primary transition-colors flex-shrink-0"
			>
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		</div>
	{/each}
</div>
