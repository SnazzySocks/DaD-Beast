<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { query } from '@urql/svelte';
	import { SEARCH_QUERY } from '$lib/graphql/queries';
	import { humorMode } from '$lib/stores/humor';
	import TorrentCard from '$lib/components/torrent/TorrentCard.svelte';
	import Loader from '$lib/components/common/Loader.svelte';

	let searchQuery = $page.url.searchParams.get('q') || '';
	let searchType = 'all';

	$: searchResult = searchQuery ? query({
		query: SEARCH_QUERY,
		variables: {
			query: searchQuery,
			type: searchType === 'all' ? undefined : searchType
		}
	}) : null;

	function handleSearch(e: Event) {
		e.preventDefault();
		if (searchQuery.trim()) {
			goto(`/search?q=${encodeURIComponent(searchQuery)}`);
		}
	}

	// Search types with both normal and dad humor variants
	$: searchTypes = [
		{
			value: 'all',
			label: $humorMode === 'dad' ? "everything's terrible anyway" : 'All'
		},
		{
			value: 'torrents',
			label: $humorMode === 'dad' ? 'digital mistakes' : 'Torrents'
		},
		{
			value: 'users',
			label: $humorMode === 'dad' ? 'other lost souls' : 'Users'
		},
		{
			value: 'posts',
			label: $humorMode === 'dad' ? 'collective regrets' : 'Forum Posts'
		}
	];
</script>

<svelte:head>
	<title>Search - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<h1 class="text-3xl font-bold text-primary mb-8">
		{$humorMode === 'dad' ? 'looking for something better' : 'Search'}
	</h1>

	<!-- Search Form -->
	<div class="card p-6 mb-8">
		<form on:submit={handleSearch} class="space-y-4">
			<div class="flex gap-2">
				<input
					type="text"
					bind:value={searchQuery}
					class="input flex-1"
					placeholder={$humorMode === 'dad' ? 'looking for meaning...' : 'Search torrents, users, posts...'}
					autofocus
				/>
				<button type="submit" class="btn btn-primary px-8">
					{$humorMode === 'dad' ? 'looking for meaning' : 'Search'}
				</button>
			</div>

			<div class="flex gap-2">
				{#each searchTypes as type}
					<button
						type="button"
						on:click={() => searchType = type.value}
						class="px-4 py-2 rounded-lg text-sm font-medium transition-colors
							{searchType === type.value
							? 'bg-blue-500 text-white'
							: 'bg-surface-light text-muted hover:bg-surface'}"
					>
						{type.label}
					</button>
				{/each}
			</div>
		</form>
	</div>

	<!-- Search Results -->
	{#if searchResult}
		{#if $searchResult.fetching}
			<div class="card p-12">
				<Loader text="Searching..." />
			</div>
		{:else if $searchResult.error}
			<div class="card p-8 text-center">
				<p class="text-red-500">Error searching: {$searchResult.error.message}</p>
			</div>
		{:else if $searchResult.data?.search}
			{@const results = $searchResult.data.search}

			<!-- Torrents -->
			{#if (searchType === 'all' || searchType === 'torrents') && results.torrents?.items?.length > 0}
				<div class="mb-8">
					<h2 class="text-2xl font-bold text-primary mb-4">
						Torrents ({results.torrents.total})
					</h2>
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
						{#each results.torrents.items as torrent}
							<TorrentCard {torrent} />
						{/each}
					</div>
				</div>
			{/if}

			<!-- Users -->
			{#if (searchType === 'all' || searchType === 'users') && results.users?.items?.length > 0}
				<div class="mb-8">
					<h2 class="text-2xl font-bold text-primary mb-4">
						Users ({results.users.total})
					</h2>
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
						{#each results.users.items as user}
							<a href="/user/{user.id}" class="card p-4 hover:shadow-md transition-shadow">
								<div class="flex items-center space-x-4">
									<img
										src={user.avatar || '/default-avatar.png'}
										alt={user.username}
										class="w-12 h-12 rounded-full"
									/>
									<div>
										<p class="font-semibold text-primary">{user.username}</p>
										<p class="text-xs text-muted">{user.role}</p>
									</div>
								</div>
							</a>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Forum Posts -->
			{#if (searchType === 'all' || searchType === 'posts') && results.posts?.items?.length > 0}
				<div class="mb-8">
					<h2 class="text-2xl font-bold text-primary mb-4">
						Forum Posts ({results.posts.total})
					</h2>
					<div class="space-y-4">
						{#each results.posts.items as post}
							<a href="/topic/{post.topic.id}" class="block card p-4 hover:shadow-md transition-shadow">
								<h3 class="font-semibold text-primary mb-2">{post.topic.title}</h3>
								<p class="text-sm text-muted line-clamp-2 mb-2">{post.content}</p>
								<div class="flex items-center space-x-2 text-xs text-muted">
									<img
										src={post.user.avatar || '/default-avatar.png'}
										alt={post.user.username}
										class="w-5 h-5 rounded-full"
									/>
									<span>{post.user.username}</span>
									<span>•</span>
									<span>{new Date(post.createdAt).toLocaleDateString()}</span>
								</div>
							</a>
						{/each}
					</div>
				</div>
			{/if}

			<!-- No Results -->
			{#if (!results.torrents?.items?.length && !results.users?.items?.length && !results.posts?.items?.length)}
				<div class="card p-12 text-center">
					<p class="text-muted text-lg">No results found for "{searchQuery}"</p>
					<p class="text-sm text-muted mt-2">Try different keywords or check your spelling</p>
				</div>
			{/if}
		{/if}
	{:else}
		<div class="card p-12 text-center">
			<svg class="w-16 h-16 mx-auto text-muted mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
			</svg>
			<p class="text-muted text-lg">Start typing to search</p>
		</div>
	{/if}
</div>
