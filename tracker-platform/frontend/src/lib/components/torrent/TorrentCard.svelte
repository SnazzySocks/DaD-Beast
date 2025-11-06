<script lang="ts">
	export let torrent: {
		id: string;
		name: string;
		description?: string;
		size: number;
		seeders: number;
		leechers: number;
		completed: number;
		category: string;
		tags?: string[];
		uploadedBy?: {
			id: string;
			username: string;
			avatar?: string;
		};
		createdAt: string;
	};

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatDate(date: string): string {
		return new Date(date).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	const categoryColors: Record<string, string> = {
		Movies: 'bg-purple-500',
		TV: 'bg-blue-500',
		Music: 'bg-pink-500',
		Games: 'bg-green-500',
		Software: 'bg-orange-500',
		Books: 'bg-yellow-500',
		Other: 'bg-gray-500'
	};
</script>

<div class="card p-4 hover:shadow-md transition-shadow">
	<div class="flex items-start justify-between mb-3">
		<div class="flex-1 min-w-0">
			<a href="/torrent/{torrent.id}" class="text-lg font-semibold text-primary hover:text-blue-500 line-clamp-2">
				{torrent.name}
			</a>

			{#if torrent.description}
				<p class="text-sm text-muted mt-1 line-clamp-2">{torrent.description}</p>
			{/if}
		</div>

		<span class="ml-3 px-2 py-1 text-xs font-medium text-white rounded {categoryColors[torrent.category] || categoryColors.Other}">
			{torrent.category}
		</span>
	</div>

	<!-- Stats -->
	<div class="grid grid-cols-4 gap-4 mb-3">
		<div class="text-center">
			<p class="text-xs text-muted">Size</p>
			<p class="text-sm font-medium text-primary">{formatBytes(torrent.size)}</p>
		</div>
		<div class="text-center">
			<p class="text-xs text-muted">Seeders</p>
			<p class="text-sm font-medium text-green-500">{torrent.seeders}</p>
		</div>
		<div class="text-center">
			<p class="text-xs text-muted">Leechers</p>
			<p class="text-sm font-medium text-red-500">{torrent.leechers}</p>
		</div>
		<div class="text-center">
			<p class="text-xs text-muted">Completed</p>
			<p class="text-sm font-medium text-primary">{torrent.completed}</p>
		</div>
	</div>

	<!-- Tags -->
	{#if torrent.tags && torrent.tags.length > 0}
		<div class="flex flex-wrap gap-1 mb-3">
			{#each torrent.tags.slice(0, 3) as tag}
				<span class="px-2 py-1 text-xs bg-surface-light text-muted rounded">
					{tag}
				</span>
			{/each}
		</div>
	{/if}

	<!-- Footer -->
	<div class="flex items-center justify-between pt-3 border-t border-theme">
		<div class="flex items-center space-x-2">
			{#if torrent.uploadedBy}
				<img
					src={torrent.uploadedBy.avatar || '/default-avatar.png'}
					alt={torrent.uploadedBy.username}
					class="w-6 h-6 rounded-full object-cover"
				/>
				<a href="/user/{torrent.uploadedBy.id}" class="text-sm text-muted hover:text-primary">
					{torrent.uploadedBy.username}
				</a>
			{/if}
		</div>

		<span class="text-xs text-muted">{formatDate(torrent.createdAt)}</span>
	</div>
</div>
