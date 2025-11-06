<script lang="ts">
	import { notifications } from '$lib/stores/notifications';
	import { mutation } from '@urql/svelte';
	import { FORGOT_PASSWORD_MUTATION } from '$lib/graphql/mutations';

	let email = '';
	let loading = false;
	let submitted = false;

	const forgotPasswordMutation = mutation({ query: FORGOT_PASSWORD_MUTATION });

	async function handleSubmit() {
		if (!email) {
			notifications.error('Please enter your email address');
			return;
		}

		loading = true;

		try {
			const result = await forgotPasswordMutation({ email });

			if (result.error) {
				notifications.error(result.error.message);
			} else {
				submitted = true;
				notifications.success('Password reset link sent to your email');
			}
		} catch (error) {
			notifications.error('An error occurred. Please try again.');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Forgot Password - Tracker Platform</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center px-4 py-12">
	<div class="w-full max-w-md">
		<div class="card p-8">
			<div class="flex justify-center mb-8">
				<div class="w-16 h-16 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl flex items-center justify-center">
					<span class="text-white font-bold text-3xl">T</span>
				</div>
			</div>

			{#if !submitted}
				<h1 class="text-2xl font-bold text-center text-primary mb-4">Reset Password</h1>
				<p class="text-center text-muted mb-8">
					Enter your email address and we'll send you a link to reset your password.
				</p>

				<form on:submit|preventDefault={handleSubmit} class="space-y-6">
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

					<button type="submit" class="w-full btn btn-primary" disabled={loading}>
						{loading ? 'Sending...' : 'Send Reset Link'}
					</button>
				</form>

				<p class="mt-6 text-center text-sm text-muted">
					Remember your password?
					<a href="/login" class="text-blue-500 hover:text-blue-600 font-medium">Sign in</a>
				</p>
			{:else}
				<div class="text-center">
					<div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
						<svg class="w-8 h-8 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
						</svg>
					</div>

					<h2 class="text-xl font-bold text-primary mb-2">Check Your Email</h2>
					<p class="text-muted mb-6">
						We've sent a password reset link to <strong>{email}</strong>
					</p>

					<a href="/login" class="btn btn-primary">Back to Login</a>
				</div>
			{/if}
		</div>
	</div>
</div>
