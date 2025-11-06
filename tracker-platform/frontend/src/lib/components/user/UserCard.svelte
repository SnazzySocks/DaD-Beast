<script lang="ts">
	export let user: {
		id: string;
		username: string;
		avatar?: string;
		role: string;
		uploaded: number;
		downloaded: number;
		ratio: number;
		joinedAt: string;
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

	const roleColors: Record<string, string> = {
		admin: 'bg-red-500',
		moderator: 'bg-blue-500',
		vip: 'bg-purple-500',
		user: 'bg-gray-500'
	};
</script>

<div class="card p-6">
	<div class="flex items-center space-x-4 mb-4">
		<img
			src={user.avatar || '/default-avatar.png'}
			alt={user.username}
			class="w-16 h-16 rounded-full object-cover"
		/>

		<div class="flex-1 min-w-0">
			<a href="/user/{user.id}" class="text-lg font-semibold text-primary hover:text-blue-500">
				{user.username}
			</a>

			<span class="inline-block mt-1 px-2 py-1 text-xs font-medium text-white rounded {roleColors[user.role.toLowerCase()] || roleColors.user}">
				{user.role}
			</span>
		</div>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<p class="text-xs text-muted">Uploaded</p>
			<p class="text-sm font-medium text-green-500">{formatBytes(user.uploaded)}</p>
		</div>
		<div>
			<p class="text-xs text-muted">Downloaded</p>
			<p class="text-sm font-medium text-blue-500">{formatBytes(user.downloaded)}</p>
		</div>
		<div>
			<p class="text-xs text-muted">Ratio</p>
			<p class="text-sm font-medium text-primary">{user.ratio.toFixed(2)}</p>
		</div>
		<div>
			<p class="text-xs text-muted">Joined</p>
			<p class="text-sm font-medium text-primary">{formatDate(user.joinedAt)}</p>
		</div>
	</div>
</div>
