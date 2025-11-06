<script lang="ts">
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let text = '';

	const sizes = {
		sm: 'w-6 h-6 border-2',
		md: 'w-10 h-10 border-3',
		lg: 'w-16 h-16 border-4'
	};
</script>

<div class="flex flex-col items-center justify-center space-y-4">
	<div class="spinner {sizes[size]}"></div>
	{#if text}
		<p class="text-sm text-muted">{text}</p>
	{/if}
</div>
