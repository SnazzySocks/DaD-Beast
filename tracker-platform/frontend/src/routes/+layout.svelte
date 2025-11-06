<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { currentTheme } from '$lib/stores/theme';
	import { setContextClient } from '@urql/svelte';
	import { client } from '$lib/graphql/client';
	import Header from '$lib/components/layout/Header.svelte';
	import Footer from '$lib/components/layout/Footer.svelte';
	import Notification from '$lib/components/common/Notification.svelte';

	// Set GraphQL client context
	setContextClient(client);

	onMount(() => {
		// Initialize theme
		currentTheme.init();
	});
</script>

<div class="min-h-screen flex flex-col">
	<Header />

	<main class="flex-1">
		<slot />
	</main>

	<Footer />

	<Notification />
</div>
