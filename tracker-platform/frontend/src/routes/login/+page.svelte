<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth';
	import { notifications } from '$lib/stores/notifications';
	import { humorMode } from '$lib/stores/humor';
	import { mutation } from '@urql/svelte';
	import { LOGIN_MUTATION } from '$lib/graphql/mutations';
	import DadJoke from '$lib/components/common/DadJoke.svelte';
	import HumorToggle from '$lib/components/common/HumorToggle.svelte';

	let email = '';
	let password = '';
	let twoFactorCode = '';
	let show2FA = false;
	let loading = false;

	const loginMutation = mutation({ query: LOGIN_MUTATION });

	async function handleSubmit() {
		if (!email || !password) {
			notifications.error('Please fill in all fields');
			return;
		}

		loading = true;

		try {
			const result = await loginMutation({
				email,
				password,
				twoFactorCode: show2FA ? twoFactorCode : undefined
			});

			if (result.error) {
				if (result.error.message.includes('2FA')) {
					show2FA = true;
					notifications.info('Please enter your 2FA code');
				} else {
					notifications.error(result.error.message);
				}
			} else if (result.data?.login) {
				auth.login(result.data.login.user, result.data.login.token);
				notifications.success('Login successful!');
				goto('/');
			}
		} catch (error) {
			notifications.error('An error occurred during login');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Login - Tracker Platform</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center px-4 py-12">
	<div class="w-full max-w-md">
		<!-- Humor Mode Toggle -->
		<HumorToggle variant="default" />

		<!-- Dad Joke for UX Enhancement -->
		<DadJoke variant="default" />

		<div class="card p-8">
			<!-- Logo -->
			<div class="flex justify-center mb-8">
				<div class="w-16 h-16 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl flex items-center justify-center">
					<span class="text-white font-bold text-3xl">T</span>
				</div>
			</div>

			<h1 class="text-2xl font-bold text-center text-primary mb-8">
				{$humorMode === 'dad' ? 'guess i\'m back' : 'Welcome Back'}
			</h1>

			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
				<!-- Email -->
				<div>
					<label for="email" class="block text-sm font-medium text-primary mb-2">
						Email Address
					</label>
					<input
						id="email"
						type="email"
						bind:value={email}
						class="input"
						placeholder="you@example.com"
						required
						disabled={loading}
					/>
				</div>

				<!-- Password -->
				<div>
					<label for="password" class="block text-sm font-medium text-primary mb-2">
						Password
					</label>
					<input
						id="password"
						type="password"
						bind:value={password}
						class="input"
						placeholder="••••••••"
						required
						disabled={loading}
					/>
				</div>

				<!-- 2FA Code -->
				{#if show2FA}
					<div class="animate-fade-in">
						<label for="twoFactorCode" class="block text-sm font-medium text-primary mb-2">
							Two-Factor Code
						</label>
						<input
							id="twoFactorCode"
							type="text"
							bind:value={twoFactorCode}
							class="input"
							placeholder="123456"
							maxlength="6"
							required
							disabled={loading}
						/>
					</div>
				{/if}

				<!-- Remember & Forgot -->
				<div class="flex items-center justify-between">
					<label class="flex items-center">
						<input type="checkbox" class="rounded border-theme text-blue-500 focus:ring-blue-500" />
						<span class="ml-2 text-sm text-muted">
							{$humorMode === 'dad' ? 'unfortunately, yes' : 'Remember me'}
						</span>
					</label>

					<a href="/forgot-password" class="text-sm text-blue-500 hover:text-blue-600">
						{$humorMode === 'dad' ? 'forgot why i\'m here too' : 'Forgot password?'}
					</a>
				</div>

				<!-- Submit Button -->
				<button
					type="submit"
					class="w-full btn btn-primary"
					disabled={loading}
				>
					{#if loading}
						{$humorMode === 'dad' ? 'crawling back...' : 'Logging in...'}
					{:else}
						{$humorMode === 'dad' ? 'fine, i\'ll stay' : 'Login'}
					{/if}
				</button>
			</form>

			<!-- Register Link -->
			<p class="mt-6 text-center text-sm text-muted">
				Don't have an account?
				<a href="/register" class="text-blue-500 hover:text-blue-600 font-medium">
					{$humorMode === 'dad' ? 'why am i doing this' : 'Register now'}
				</a>
			</p>
		</div>
	</div>
</div>
