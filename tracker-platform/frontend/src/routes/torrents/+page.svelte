<script lang="ts">
	import { query } from '@urql/svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { TORRENTS_QUERY } from '$lib/graphql/queries';
	import TorrentCard from '$lib/components/torrent/TorrentCard.svelte';
	import Loader from '$lib/components/common/Loader.svelte';

	let currentPage = 1;
	let search = '';
	let category = '';
	let sort = 'newest';

	$: torrentsResult = query({
		query: TORRENTS_QUERY,
		variables: {
			page: currentPage,
			limit: 24,
			search: search || undefined,
			category: category || undefined,
			sort
		}
	});

	const categories = [
		'All',
		'Movies',
		'TV',
		'Music',
		'Games',
		'Software',
		'Books',
		'Other'
	];

	const sortOptions = [
		{ value: 'newest', label: 'fresh disappointments' },
		{ value: 'oldest', label: 'ancient regrets' },
		{ value: 'seeders', label: 'popular mistakes' },
		{ value: 'leechers', label: 'shared misery' },
		{ value: 'size', label: 'biggest wastes of space' },
		{ value: 'name', label: 'alphabetical failures' }
	];

	function handleSearch(e: Event) {
		e.preventDefault();
		currentPage = 1;
		torrentsResult = query({
			query: TORRENTS_QUERY,
			variables: {
				page: currentPage,
				limit: 24,
				search: search || undefined,
				category: category || undefined,
				sort
			}
		});
	}

	function selectCategory(cat: string) {
		category = cat === 'All' ? '' : cat;
		currentPage = 1;
	}

	function changePage(newPage: number) {
		currentPage = newPage;
		window.scrollTo({ top: 0, behavior: 'smooth' });
	}
</script>

<svelte:head>
	<title>Browse Torrents - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<div class="flex items-center justify-between mb-8">
		<h1 class="text-3xl font-bold text-primary">wasting time like always</h1>
		<a href="/upload" class="btn btn-primary">adding to my regrets</a>
	</div>

	<!-- Filters -->
	<div class="card p-6 mb-6">
		<!-- Search -->
		<form on:submit={handleSearch} class="mb-4">
			<div class="flex gap-2">
				<input
					type="text"
					bind:value={search}
					class="input flex-1"
					placeholder="looking for meaning..."
				/>
				<button type="submit" class="btn btn-primary px-6">looking for meaning</button>
			</div>
		</form>

		<!-- Categories -->
		<div class="mb-4">
			<p class="text-sm font-medium text-muted mb-2">Category</p>
			<div class="flex flex-wrap gap-2">
				{#each categories as cat}
					<button
						on:click={() => selectCategory(cat)}
						class="px-4 py-2 rounded-lg text-sm font-medium transition-colors
							{(cat === 'All' && !category) || cat === category
							? 'bg-blue-500 text-white'
							: 'bg-surface-light text-muted hover:bg-surface hover:text-primary'}"
					>
						{cat}
					</button>
				{/each}
			</div>
		</div>

		<!-- Sort -->
		<div>
			<p class="text-sm font-medium text-muted mb-2">Sort By</p>
			<select bind:value={sort} class="input w-full sm:w-64">
				{#each sortOptions as option}
					<option value={option.value}>{option.label}</option>
				{/each}
			</select>
		</div>
	</div>

	<!-- Results -->
	{#if $torrentsResult.fetching}
		<div class="card p-12">
			<Loader text="Loading torrents..." />
		</div>
	{:else if $torrentsResult.error}
		<div class="card p-8 text-center">
			<p class="text-red-500">Error loading torrents: {$torrentsResult.error.message}</p>
		</div>
	{:else if $torrentsResult.data?.torrents?.items}
		{@const torrents = $torrentsResult.data.torrents}

		<!-- Results Info -->
		<div class="mb-4">
			<p class="text-sm text-muted">
				Showing {torrents.items.length} of {torrents.total.toLocaleString()} torrents
			</p>
		</div>

		<!-- Torrent Grid -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
			{#each torrents.items as torrent}
				<TorrentCard {torrent} />
			{/each}
		</div>

		<!-- Pagination -->
		{#if torrents.pages > 1}
			<div class="flex justify-center gap-2">
				<button
					on:click={() => changePage(currentPage - 1)}
					disabled={currentPage === 1}
					class="btn btn-secondary"
				>
					back to worse
				</button>

				{#each Array(Math.min(5, torrents.pages)) as _, i}
					{@const pageNum = currentPage <= 3 ? i + 1 : currentPage - 2 + i}
					{#if pageNum <= torrents.pages}
						<button
							on:click={() => changePage(pageNum)}
							class="btn {pageNum === currentPage ? 'btn-primary' : 'btn-secondary'}"
						>
							level {pageNum}
						</button>
					{/if}
				{/each}

				<button
					on:click={() => changePage(currentPage + 1)}
					disabled={currentPage === torrents.pages}
					class="btn btn-secondary"
				>
					forward into nothing
				</button>
			</div>
		{/if}
	{:else}
		<div class="card p-12 text-center">
			<p class="text-muted text-lg">No torrents found</p>
			<p class="text-sm text-muted mt-2">Try adjusting your search or filters</p>
		</div>
	{/if}
</div>
