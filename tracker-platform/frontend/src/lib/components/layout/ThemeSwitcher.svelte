<script lang="ts">
	import { currentTheme, themes, type Theme } from '$lib/stores/theme';
	import { humorMode } from '$lib/stores/humor';

	let isOpen = false;

	function toggleDropdown() {
		isOpen = !isOpen;
	}

	function selectTheme(theme: Theme) {
		currentTheme.set(theme);
		isOpen = false;
	}

	// Theme icons
	const themeIcons: Record<Theme, string> = {
		dark: '🌙',
		grey: '⚪',
		light: '☀️',
		aero: '💎',
		coffee: '☕'
	};

	// Dark dad alternatives for theme labels
	const dadThemeLabels: Record<Theme, { label: string; description: string }> = {
		dark: { label: 'like my mood', description: 'Emotional darkness' },
		grey: { label: 'colorless existence', description: 'Dull reality' },
		light: { label: 'too bright for my soul', description: "Can't handle optimism" },
		aero: { label: 'trying to feel something', description: 'Desperate for sensation' },
		coffee: { label: 'the only thing keeping me going', description: 'Caffeine dependency' }
	};
</script>

<div class="relative">
	<button
		on:click={toggleDropdown}
		class="p-2 text-muted hover:text-primary transition-colors rounded-lg hover:bg-surface-light"
		aria-label="Change theme"
	>
		<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"
			/>
		</svg>
	</button>

	{#if isOpen}
		<div class="absolute right-0 mt-2 w-64 bg-surface border border-theme rounded-lg shadow-lg py-2 animate-fade-in z-50">
			<div class="px-4 py-2 border-b border-theme">
				<p class="text-xs font-semibold text-muted uppercase">Select Theme</p>
			</div>

			{#each Object.entries(themes) as [key, theme]}
				{@const dadTheme = dadThemeLabels[key as Theme]}
				<button
					on:click={() => selectTheme(key as Theme)}
					class="w-full px-4 py-3 text-left hover:bg-surface-light transition-colors flex items-center justify-between
						{$currentTheme === key ? 'bg-surface-light' : ''}"
				>
					<div class="flex items-center space-x-3">
						<span class="text-2xl">{themeIcons[key as Theme]}</span>
						<div>
							<p class="text-sm font-medium text-primary">
								{$humorMode === 'dad' ? dadTheme.label : theme.label}
							</p>
							<p class="text-xs text-muted">
								{$humorMode === 'dad' ? dadTheme.description : theme.description}
							</p>
						</div>
					</div>
					{#if $currentTheme === key}
						<svg class="w-5 h-5 text-blue-500" fill="currentColor" viewBox="0 0 20 20">
							<path
								fill-rule="evenodd"
								d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
								clip-rule="evenodd"
							/>
						</svg>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<svelte:window on:click={(e) => {
	if (isOpen && !e.target.closest('.relative')) {
		isOpen = false;
	}
}} />
