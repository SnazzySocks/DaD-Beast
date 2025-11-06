<script lang="ts">
	import { query } from '@urql/svelte';
	import { TORRENTS_QUERY, STATS_QUERY } from '$lib/graphql/queries';
	import TorrentCard from '$lib/components/torrent/TorrentCard.svelte';
	import Loader from '$lib/components/common/Loader.svelte';

	const torrentsResult = query({
		query: TORRENTS_QUERY,
		variables: { page: 1, limit: 6, sort: 'newest' }
	});

	const statsResult = query({
		query: STATS_QUERY
	});
</script>

<svelte:head>
	<title>Home - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<!-- Hero Section -->
	<div class="card p-8 mb-8 text-center">
		<h1 class="text-4xl font-bold text-primary mb-4">Welcome to Tracker Platform</h1>
		<p class="text-lg text-muted mb-6">
			Advanced BitTorrent tracker with modern features and elegant design
		</p>
		<div class="flex flex-col sm:flex-row justify-center gap-4">
			<a href="/torrents" class="btn btn-primary">Browse Torrents</a>
			<a href="/upload" class="btn btn-secondary">Upload Torrent</a>
		</div>
	</div>

	<!-- Stats Section -->
	{#if $statsResult.fetching}
		<div class="card p-8 mb-8">
			<Loader text="Loading stats..." />
		</div>
	{:else if $statsResult.data?.stats}
		<div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
			<div class="card p-6 text-center">
				<p class="text-3xl font-bold text-blue-500">
					{$statsResult.data.stats.totalTorrents.toLocaleString()}
				</p>
				<p class="text-sm text-muted mt-2">Torrents</p>
			</div>
			<div class="card p-6 text-center">
				<p class="text-3xl font-bold text-green-500">
					{$statsResult.data.stats.totalSeeders.toLocaleString()}
				</p>
				<p class="text-sm text-muted mt-2">Seeders</p>
			</div>
			<div class="card p-6 text-center">
				<p class="text-3xl font-bold text-purple-500">
					{$statsResult.data.stats.totalUsers.toLocaleString()}
				</p>
				<p class="text-sm text-muted mt-2">Users</p>
			</div>
			<div class="card p-6 text-center">
				<p class="text-3xl font-bold text-orange-500">
					{$statsResult.data.stats.totalPeers.toLocaleString()}
				</p>
				<p class="text-sm text-muted mt-2">Peers</p>
			</div>
		</div>
	{/if}

	<!-- Featured Torrents -->
	<div class="mb-8">
		<div class="flex items-center justify-between mb-6">
			<h2 class="text-2xl font-bold text-primary">Recent Uploads</h2>
			<a href="/torrents" class="text-sm text-blue-500 hover:text-blue-600">
				View all →
			</a>
		</div>

		{#if $torrentsResult.fetching}
			<div class="card p-8">
				<Loader text="Loading torrents..." />
			</div>
		{:else if $torrentsResult.error}
			<div class="card p-8 text-center">
				<p class="text-red-500">Error loading torrents</p>
			</div>
		{:else if $torrentsResult.data?.torrents?.items}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
				{#each $torrentsResult.data.torrents.items as torrent}
					<TorrentCard {torrent} />
				{/each}
			</div>
		{:else}
			<div class="card p-8 text-center">
				<p class="text-muted">No torrents available</p>
			</div>
		{/if}
	</div>

	<!-- Features -->
	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<div class="card p-6 text-center">
			<div class="w-12 h-12 bg-blue-500 rounded-lg flex items-center justify-center mx-auto mb-4">
				<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
				</svg>
			</div>
			<h3 class="text-lg font-semibold text-primary mb-2">Lightning Fast</h3>
			<p class="text-sm text-muted">Optimized for speed and performance</p>
		</div>

		<div class="card p-6 text-center">
			<div class="w-12 h-12 bg-purple-500 rounded-lg flex items-center justify-center mx-auto mb-4">
				<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
				</svg>
			</div>
			<h3 class="text-lg font-semibold text-primary mb-2">Secure</h3>
			<p class="text-sm text-muted">Advanced security features and 2FA</p>
		</div>

		<div class="card p-6 text-center">
			<div class="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center mx-auto mb-4">
				<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
				</svg>
			</div>
			<h3 class="text-lg font-semibold text-primary mb-2">Community</h3>
			<p class="text-sm text-muted">Active forums and chat rooms</p>
		</div>
	</div>
</div>
