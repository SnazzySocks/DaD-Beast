<script lang="ts">
	import { query } from '@urql/svelte';
	import { FORUMS_QUERY } from '$lib/graphql/queries';
	import Loader from '$lib/components/common/Loader.svelte';

	const forumsResult = query({
		query: FORUMS_QUERY
	});

	function formatDate(date: string): string {
		return new Date(date).toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Forums - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<h1 class="text-3xl font-bold text-primary mb-8">Forums</h1>

	{#if $forumsResult.fetching}
		<div class="card p-12">
			<Loader text="Loading forums..." />
		</div>
	{:else if $forumsResult.error}
		<div class="card p-8 text-center">
			<p class="text-red-500">Error loading forums</p>
		</div>
	{:else if $forumsResult.data?.forums}
		<div class="space-y-4">
			{#each $forumsResult.data.forums as forum}
				<a href="/forum/{forum.id}" class="block card p-6 hover:shadow-md transition-shadow">
					<div class="flex items-start justify-between">
						<div class="flex items-start space-x-4 flex-1">
							<!-- Icon -->
							<div class="w-12 h-12 bg-blue-500 rounded-lg flex items-center justify-center flex-shrink-0">
								<span class="text-2xl">{forum.icon || '💬'}</span>
							</div>

							<!-- Info -->
							<div class="flex-1 min-w-0">
								<h2 class="text-xl font-bold text-primary mb-1">{forum.name}</h2>
								<p class="text-sm text-muted">{forum.description}</p>

								<div class="flex items-center gap-4 mt-3 text-xs text-muted">
									<span>{forum.topicCount} topics</span>
									<span>{forum.postCount} posts</span>
								</div>
							</div>
						</div>

						<!-- Last Post -->
						{#if forum.lastPost}
							<div class="hidden md:block text-right ml-4">
								<p class="text-sm text-primary line-clamp-1">
									{forum.lastPost.title}
								</p>
								<div class="flex items-center justify-end space-x-2 mt-1">
									<img
										src={forum.lastPost.user.avatar || '/default-avatar.png'}
										alt={forum.lastPost.user.username}
										class="w-5 h-5 rounded-full"
									/>
									<span class="text-xs text-muted">
										{forum.lastPost.user.username}
									</span>
								</div>
								<p class="text-xs text-muted mt-1">
									{formatDate(forum.lastPost.createdAt)}
								</p>
							</div>
						{/if}
					</div>
				</a>
			{/each}
		</div>
	{:else}
		<div class="card p-12 text-center">
			<p class="text-muted">No forums available</p>
		</div>
	{/if}
</div>
