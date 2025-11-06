<script lang="ts">
	export let message: {
		id: string;
		userId: string;
		username: string;
		avatar?: string;
		message: string;
		timestamp: string;
	};

	export let isOwnMessage = false;

	function formatTime(timestamp: string): string {
		return new Date(timestamp).toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<div class="flex items-start space-x-3 {isOwnMessage ? 'flex-row-reverse space-x-reverse' : ''} animate-fade-in">
	{#if !isOwnMessage}
		<img
			src={message.avatar || '/default-avatar.png'}
			alt={message.username}
			class="w-8 h-8 rounded-full flex-shrink-0"
		/>
	{/if}

	<div class="flex-1 min-w-0 {isOwnMessage ? 'text-right' : ''}">
		<div class="flex items-baseline space-x-2 {isOwnMessage ? 'flex-row-reverse space-x-reverse' : ''}">
			<span class="font-medium text-primary text-sm">{message.username}</span>
			<span class="text-xs text-muted">{formatTime(message.timestamp)}</span>
		</div>

		<div class="mt-1 inline-block max-w-md">
			<div class="px-4 py-2 rounded-lg break-words
				{isOwnMessage
					? 'bg-blue-500 text-white'
					: 'bg-surface-light text-primary'}">
				{message.message}
			</div>
		</div>
	</div>
</div>
