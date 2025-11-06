<script lang="ts">
	export let post: {
		id: string;
		content: string;
		user: {
			id: string;
			username: string;
			avatar?: string;
			role: string;
		};
		createdAt: string;
		updatedAt?: string;
	};

	function formatDate(date: string): string {
		return new Date(date).toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
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
	<div class="flex items-start space-x-4">
		<!-- User Info -->
		<div class="flex-shrink-0 text-center">
			<a href="/user/{post.user.id}">
				<img
					src={post.user.avatar || '/default-avatar.png'}
					alt={post.user.username}
					class="w-16 h-16 rounded-full mb-2"
				/>
			</a>
			<a href="/user/{post.user.id}" class="text-sm font-medium text-primary hover:text-blue-500">
				{post.user.username}
			</a>
			<span class="block mt-1 px-2 py-1 text-xs text-white rounded {roleColors[post.user.role.toLowerCase()] || roleColors.user}">
				{post.user.role}
			</span>
		</div>

		<!-- Post Content -->
		<div class="flex-1 min-w-0">
			<div class="flex items-center justify-between mb-4">
				<span class="text-sm text-muted">{formatDate(post.createdAt)}</span>
				{#if post.updatedAt && post.updatedAt !== post.createdAt}
					<span class="text-xs text-muted italic">
						Edited {formatDate(post.updatedAt)}
					</span>
				{/if}
			</div>

			<div class="prose prose-sm max-w-none text-primary">
				{post.content}
			</div>
		</div>
	</div>
</div>
