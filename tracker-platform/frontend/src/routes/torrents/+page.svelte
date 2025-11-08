<script lang="ts">
	import { query } from '@urql/svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { TORRENTS_QUERY } from '$lib/graphql/queries';
	import { humorMode } from '$lib/stores/humor';
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

	// Category labels with dark dad alternatives
	$: categoryLabels: Record<string, string> = {
		'All': $humorMode === 'dad' ? "everything's terrible anyway" : 'All',
		'Movies': $humorMode === 'dad' ? 'escapism attempts' : 'Movies',
		'TV': $humorMode === 'dad' ? 'mind numbing' : 'TV Shows',
		'Music': $humorMode === 'dad' ? 'noise to drown it out' : 'Music',
		'Games': $humorMode === 'dad' ? 'wasting life points' : 'Games',
		'Software': $humorMode === 'dad' ? 'digital mistakes' : 'Software',
		'Books': $humorMode === 'dad' ? "words i won't read" : 'Books',
		'Other': $humorMode === 'dad' ? 'miscellaneous regrets' : 'Other'
	};

	// Sort options with both normal and dad humor variants
	$: sortOptions = [
		{
			value: 'newest',
			label: $humorMode === 'dad' ? 'fresh disappointments' : 'Newest'
		},
		{
			value: 'oldest',
			label: $humorMode === 'dad' ? 'ancient regrets' : 'Oldest'
		},
		{
			value: 'seeders',
			label: $humorMode === 'dad' ? 'popular mistakes' : 'Most Seeders'
		},
		{
			value: 'leechers',
			label: $humorMode === 'dad' ? 'shared misery' : 'Most Leechers'
		},
		{
			value: 'size',
			label: $humorMode === 'dad' ? 'biggest wastes of space' : 'Largest Size'
		},
		{
			value: 'name',
			label: $humorMode === 'dad' ? 'alphabetical failures' : 'Name (A-Z)'
		}
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
		<h1 class="text-3xl font-bold text-primary">
			{$humorMode === 'dad' ? 'wasting time like always' : 'Browse Torrents'}
		</h1>
		<a href="/upload" class="btn btn-primary">
			{$humorMode === 'dad' ? 'adding to my regrets' : 'Upload Torrent'}
		</a>
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
					placeholder={$humorMode === 'dad' ? 'looking for meaning...' : 'Search torrents...'}
				/>
				<button type="submit" class="btn btn-primary px-6">
					{$humorMode === 'dad' ? 'looking for meaning' : 'Search'}
				</button>
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
						{categoryLabels[cat]}
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
					{$humorMode === 'dad' ? 'back to worse' : 'Previous'}
				</button>

				{#each Array(Math.min(5, torrents.pages)) as _, i}
					{@const pageNum = currentPage <= 3 ? i + 1 : currentPage - 2 + i}
					{#if pageNum <= torrents.pages}
						<button
							on:click={() => changePage(pageNum)}
							class="btn {pageNum === currentPage ? 'btn-primary' : 'btn-secondary'}"
						>
							{$humorMode === 'dad' ? `level ${pageNum}` : pageNum}
						</button>
					{/if}
				{/each}

				<button
					on:click={() => changePage(currentPage + 1)}
					disabled={currentPage === torrents.pages}
					class="btn btn-secondary"
				>
					{$humorMode === 'dad' ? 'forward into nothing' : 'Next'}
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
