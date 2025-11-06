<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { createEventDispatcher } from 'svelte';

	export let isOpen = false;
	export let title = '';
	export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';

	const dispatch = createEventDispatcher();

	const sizes = {
		sm: 'max-w-md',
		md: 'max-w-lg',
		lg: 'max-w-2xl',
		xl: 'max-w-4xl'
	};

	function close() {
		isOpen = false;
		dispatch('close');
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 overflow-y-auto"
		transition:fade={{ duration: 200 }}
		on:click={handleBackdropClick}
		role="dialog"
		aria-modal="true"
	>
		<!-- Backdrop -->
		<div class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm"></div>

		<!-- Modal -->
		<div class="flex min-h-screen items-center justify-center p-4">
			<div
				class="relative bg-surface border border-theme rounded-lg shadow-xl w-full {sizes[size]} animate-fade-in"
				transition:scale={{ duration: 200, start: 0.95 }}
			>
				<!-- Header -->
				{#if title || $$slots.header}
					<div class="px-6 py-4 border-b border-theme flex items-center justify-between">
						{#if $$slots.header}
							<slot name="header" />
						{:else}
							<h3 class="text-lg font-semibold text-primary">{title}</h3>
						{/if}

						<button
							on:click={close}
							class="text-muted hover:text-primary transition-colors"
							aria-label="Close"
						>
							<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
							</svg>
						</button>
					</div>
				{/if}

				<!-- Content -->
				<div class="px-6 py-4">
					<slot />
				</div>

				<!-- Footer -->
				{#if $$slots.footer}
					<div class="px-6 py-4 border-t border-theme bg-surface-light rounded-b-lg">
						<slot name="footer" />
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
