<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { websocket } from '$lib/stores/websocket';
	import { auth } from '$lib/stores/auth';
	import { browser } from '$app/environment';

	let message = '';
	let messagesContainer: HTMLDivElement;

	$: if (browser && messagesContainer && $websocket.messages) {
		setTimeout(() => {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}, 100);
	}

	onMount(() => {
		if ($auth.token) {
			const wsUrl = import.meta.env.PUBLIC_WS_URL || 'ws://localhost:4000';
			websocket.connect(wsUrl, $auth.token);
		}
	});

	onDestroy(() => {
		websocket.disconnect();
	});

	function handleSend() {
		if (message.trim()) {
			websocket.sendMessage(message);
			message = '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}

	function handleTyping() {
		websocket.sendTyping();
	}

	function formatTime(timestamp: string): string {
		return new Date(timestamp).toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Chat - Tracker Platform</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<div class="card h-[calc(100vh-12rem)] flex flex-col">
		<!-- Header -->
		<div class="p-4 border-b border-theme">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-2xl font-bold text-primary">Global Chat</h1>
					<p class="text-sm text-muted">
						{$websocket.connected ? '🟢 Connected' : '🔴 Disconnected'}
					</p>
				</div>

				{#if $websocket.typingUsers.size > 0}
					<p class="text-sm text-muted italic">
						{Array.from($websocket.typingUsers).join(', ')} {$websocket.typingUsers.size === 1 ? 'is' : 'are'} typing...
					</p>
				{/if}
			</div>
		</div>

		<!-- Messages -->
		<div bind:this={messagesContainer} class="flex-1 overflow-y-auto p-4 space-y-4">
			{#if $websocket.messages.length === 0}
				<div class="flex items-center justify-center h-full">
					<p class="text-muted">No messages yet. Start the conversation!</p>
				</div>
			{:else}
				{#each $websocket.messages as msg}
					<div class="flex items-start space-x-3 animate-fade-in">
						<img
							src={msg.avatar || '/default-avatar.png'}
							alt={msg.username}
							class="w-8 h-8 rounded-full flex-shrink-0"
						/>
						<div class="flex-1 min-w-0">
							<div class="flex items-baseline space-x-2">
								<span class="font-medium text-primary text-sm">{msg.username}</span>
								<span class="text-xs text-muted">{formatTime(msg.timestamp)}</span>
							</div>
							<p class="text-sm text-primary mt-1 break-words">{msg.message}</p>
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Input -->
		<div class="p-4 border-t border-theme">
			{#if $auth.isAuthenticated}
				<form on:submit|preventDefault={handleSend} class="flex gap-2">
					<input
						type="text"
						bind:value={message}
						on:keydown={handleKeydown}
						on:input={handleTyping}
						class="input flex-1"
						placeholder="Type a message..."
						disabled={!$websocket.connected}
					/>
					<button
						type="submit"
						class="btn btn-primary px-6"
						disabled={!$websocket.connected || !message.trim()}
					>
						Send
					</button>
				</form>
			{:else}
				<div class="text-center py-4">
					<p class="text-muted">
						<a href="/login" class="text-blue-500 hover:text-blue-600">Login</a> to join the chat
					</p>
				</div>
			{/if}
		</div>
	</div>
</div>
