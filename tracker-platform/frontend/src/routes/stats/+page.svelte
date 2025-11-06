<script lang="ts">
	import { query } from '@urql/svelte';
	import { STATS_QUERY } from '$lib/graphql/queries';
	import UserCard from '$lib/components/user/UserCard.svelte';
	import Loader from '$lib/components/common/Loader.svelte';

	const statsResult = query({
		query: STATS_QUERY
	});

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}
</script>

<svelte:head>
	<title>Statistics - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<h1 class="text-3xl font-bold text-primary mb-8">Platform Statistics</h1>

	{#if $statsResult.fetching}
		<div class="card p-12">
			<Loader text="Loading statistics..." />
		</div>
	{:else if $statsResult.error}
		<div class="card p-8 text-center">
			<p class="text-red-500">Error loading statistics</p>
		</div>
	{:else if $statsResult.data?.stats}
		{@const stats = $statsResult.data.stats}

		<!-- Overview Stats -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
			<div class="card p-6 text-center">
				<div class="w-12 h-12 bg-blue-500 rounded-lg flex items-center justify-center mx-auto mb-4">
					<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
					</svg>
				</div>
				<p class="text-3xl font-bold text-primary mb-2">{stats.totalTorrents.toLocaleString()}</p>
				<p class="text-sm text-muted">Total Torrents</p>
			</div>

			<div class="card p-6 text-center">
				<div class="w-12 h-12 bg-purple-500 rounded-lg flex items-center justify-center mx-auto mb-4">
					<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
					</svg>
				</div>
				<p class="text-3xl font-bold text-primary mb-2">{stats.totalUsers.toLocaleString()}</p>
				<p class="text-sm text-muted">Total Users</p>
			</div>

			<div class="card p-6 text-center">
				<div class="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center mx-auto mb-4">
					<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 11l5-5m0 0l5 5m-5-5v12" />
					</svg>
				</div>
				<p class="text-3xl font-bold text-primary mb-2">{stats.totalSeeders.toLocaleString()}</p>
				<p class="text-sm text-muted">Total Seeders</p>
			</div>

			<div class="card p-6 text-center">
				<div class="w-12 h-12 bg-red-500 rounded-lg flex items-center justify-center mx-auto mb-4">
					<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16l5 5m0 0l5-5m-5 5V4" />
					</svg>
				</div>
				<p class="text-3xl font-bold text-primary mb-2">{stats.totalLeechers.toLocaleString()}</p>
				<p class="text-sm text-muted">Total Leechers</p>
			</div>
		</div>

		<!-- Bandwidth Stats -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
			<div class="card p-6">
				<h2 class="text-xl font-bold text-primary mb-4">Total Uploaded</h2>
				<p class="text-4xl font-bold text-green-500">{formatBytes(stats.totalUploaded)}</p>
			</div>

			<div class="card p-6">
				<h2 class="text-xl font-bold text-primary mb-4">Total Downloaded</h2>
				<p class="text-4xl font-bold text-blue-500">{formatBytes(stats.totalDownloaded)}</p>
			</div>
		</div>

		<!-- Top Uploaders -->
		{#if stats.topUploaders && stats.topUploaders.length > 0}
			<div class="mb-8">
				<h2 class="text-2xl font-bold text-primary mb-6">Top Uploaders</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
					{#each stats.topUploaders as user}
						<UserCard {user} />
					{/each}
				</div>
			</div>
		{/if}

		<!-- Recent Users -->
		{#if stats.recentUsers && stats.recentUsers.length > 0}
			<div>
				<h2 class="text-2xl font-bold text-primary mb-6">Recent Users</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
					{#each stats.recentUsers as user}
						<div class="card p-4 flex items-center space-x-4">
							<img
								src={user.avatar || '/default-avatar.png'}
								alt={user.username}
								class="w-12 h-12 rounded-full"
							/>
							<div>
								<a href="/user/{user.id}" class="font-medium text-primary hover:text-blue-500">
									{user.username}
								</a>
								<p class="text-xs text-muted">
									Joined {new Date(user.joinedAt).toLocaleDateString()}
								</p>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>
