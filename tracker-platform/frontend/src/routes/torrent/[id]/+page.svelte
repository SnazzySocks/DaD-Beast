<script lang="ts">
	import { page } from '$app/stores';
	import { query, mutation } from '@urql/svelte';
	import { TORRENT_QUERY } from '$lib/graphql/queries';
	import { ADD_COMMENT_MUTATION } from '$lib/graphql/mutations';
	import { notifications } from '$lib/stores/notifications';
	import { isAuthenticated } from '$lib/stores/auth';
	import Loader from '$lib/components/common/Loader.svelte';

	$: torrentId = $page.params.id;

	$: torrentResult = query({
		query: TORRENT_QUERY,
		variables: { id: torrentId }
	});

	const addCommentMutation = mutation({ query: ADD_COMMENT_MUTATION });

	let commentContent = '';
	let addingComment = false;

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatDate(date: string): string {
		return new Date(date).toLocaleString();
	}

	async function handleAddComment() {
		if (!commentContent.trim()) {
			notifications.error('Comment cannot be empty');
			return;
		}

		addingComment = true;

		try {
			const result = await addCommentMutation({
				torrentId,
				content: commentContent
			});

			if (result.error) {
				notifications.error(result.error.message);
			} else {
				notifications.success('Comment added');
				commentContent = '';
				// Refresh torrent data
				torrentResult = query({
					query: TORRENT_QUERY,
					variables: { id: torrentId }
				});
			}
		} catch (error) {
			notifications.error('Failed to add comment');
		} finally {
			addingComment = false;
		}
	}
</script>

<svelte:head>
	<title>{$torrentResult.data?.torrent?.name || 'Torrent'} - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	{#if $torrentResult.fetching}
		<div class="card p-12">
			<Loader text="Loading torrent..." />
		</div>
	{:else if $torrentResult.error}
		<div class="card p-8 text-center">
			<p class="text-red-500">Error loading torrent</p>
		</div>
	{:else if $torrentResult.data?.torrent}
		{@const torrent = $torrentResult.data.torrent}

		<!-- Header -->
		<div class="card p-6 mb-6">
			<div class="flex items-start justify-between mb-4">
				<div class="flex-1">
					<h1 class="text-3xl font-bold text-primary mb-2">{torrent.name}</h1>
					<p class="text-muted">{torrent.description || 'No description'}</p>
				</div>
				<span class="ml-4 px-3 py-1 bg-blue-500 text-white text-sm font-medium rounded">
					{torrent.category}
				</span>
			</div>

			<!-- Stats -->
			<div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
				<div class="bg-surface-light p-4 rounded-lg">
					<p class="text-xs text-muted mb-1">Size</p>
					<p class="text-lg font-semibold text-primary">{formatBytes(torrent.size)}</p>
				</div>
				<div class="bg-surface-light p-4 rounded-lg">
					<p class="text-xs text-muted mb-1">Seeders</p>
					<p class="text-lg font-semibold text-green-500">{torrent.seeders}</p>
				</div>
				<div class="bg-surface-light p-4 rounded-lg">
					<p class="text-xs text-muted mb-1">Leechers</p>
					<p class="text-lg font-semibold text-red-500">{torrent.leechers}</p>
				</div>
				<div class="bg-surface-light p-4 rounded-lg">
					<p class="text-xs text-muted mb-1">Completed</p>
					<p class="text-lg font-semibold text-primary">{torrent.completed}</p>
				</div>
			</div>

			<!-- Actions -->
			<div class="flex gap-4">
				<button class="btn btn-primary flex-1">
					<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
					</svg>
					Download
				</button>
				<button class="btn btn-secondary">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
					</svg>
				</button>
			</div>
		</div>

		<!-- Uploader Info -->
		{#if torrent.uploadedBy}
			<div class="card p-6 mb-6">
				<h2 class="text-xl font-bold text-primary mb-4">Uploader</h2>
				<div class="flex items-center space-x-4">
					<img
						src={torrent.uploadedBy.avatar || '/default-avatar.png'}
						alt={torrent.uploadedBy.username}
						class="w-12 h-12 rounded-full"
					/>
					<div>
						<a href="/user/{torrent.uploadedBy.id}" class="text-lg font-medium text-primary hover:text-blue-500">
							{torrent.uploadedBy.username}
						</a>
						<p class="text-sm text-muted">Uploaded on {formatDate(torrent.createdAt)}</p>
					</div>
				</div>
			</div>
		{/if}

		<!-- Files -->
		{#if torrent.files && torrent.files.length > 0}
			<div class="card p-6 mb-6">
				<h2 class="text-xl font-bold text-primary mb-4">Files</h2>
				<div class="space-y-2">
					{#each torrent.files as file}
						<div class="flex justify-between items-center p-3 bg-surface-light rounded">
							<span class="text-sm text-primary truncate">{file.path}</span>
							<span class="text-sm text-muted ml-4">{formatBytes(file.size)}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Comments -->
		<div class="card p-6">
			<h2 class="text-xl font-bold text-primary mb-4">
				Comments ({torrent.comments?.length || 0})
			</h2>

			{#if $isAuthenticated}
				<div class="mb-6">
					<textarea
						bind:value={commentContent}
						class="input min-h-[100px] resize-none"
						placeholder="Add a comment..."
						disabled={addingComment}
					></textarea>
					<button
						on:click={handleAddComment}
						class="btn btn-primary mt-2"
						disabled={addingComment}
					>
						{addingComment ? 'Posting...' : 'Post Comment'}
					</button>
				</div>
			{:else}
				<p class="text-muted mb-6">
					<a href="/login" class="text-blue-500 hover:text-blue-600">Login</a> to comment
				</p>
			{/if}

			{#if torrent.comments && torrent.comments.length > 0}
				<div class="space-y-4">
					{#each torrent.comments as comment}
						<div class="flex space-x-4">
							<img
								src={comment.user.avatar || '/default-avatar.png'}
								alt={comment.user.username}
								class="w-10 h-10 rounded-full"
							/>
							<div class="flex-1">
								<div class="flex items-center space-x-2 mb-1">
									<a href="/user/{comment.user.id}" class="font-medium text-primary hover:text-blue-500">
										{comment.user.username}
									</a>
									<span class="text-xs text-muted">{formatDate(comment.createdAt)}</span>
								</div>
								<p class="text-sm text-primary">{comment.content}</p>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<p class="text-center text-muted py-8">No comments yet</p>
			{/if}
		</div>
	{/if}
</div>
