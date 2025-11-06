<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth';
	import { notifications } from '$lib/stores/notifications';
	import { mutation } from '@urql/svelte';
	import { REGISTER_MUTATION } from '$lib/graphql/mutations';

	let username = '';
	let email = '';
	let password = '';
	let confirmPassword = '';
	let inviteCode = '';
	let agreedToTerms = false;
	let loading = false;

	const registerMutation = mutation({ query: REGISTER_MUTATION });

	async function handleSubmit() {
		if (!username || !email || !password || !confirmPassword) {
			notifications.error('Please fill in all required fields');
			return;
		}

		if (password !== confirmPassword) {
			notifications.error('Passwords do not match');
			return;
		}

		if (password.length < 8) {
			notifications.error('Password must be at least 8 characters');
			return;
		}

		if (!agreedToTerms) {
			notifications.error('You must agree to the terms and conditions');
			return;
		}

		loading = true;

		try {
			const result = await registerMutation({
				username,
				email,
				password,
				inviteCode: inviteCode || undefined
			});

			if (result.error) {
				notifications.error(result.error.message);
			} else if (result.data?.register) {
				auth.login(result.data.register.user, result.data.register.token);
				notifications.success('Registration successful! Welcome!');
				goto('/');
			}
		} catch (error) {
			notifications.error('An error occurred during registration');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Register - Tracker Platform</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center px-4 py-12">
	<div class="w-full max-w-md">
		<div class="card p-8">
			<!-- Logo -->
			<div class="flex justify-center mb-8">
				<div class="w-16 h-16 bg-gradient-to-br from-blue-500 to-purple-600 rounded-xl flex items-center justify-center">
					<span class="text-white font-bold text-3xl">T</span>
				</div>
			</div>

			<h1 class="text-2xl font-bold text-center text-primary mb-8">Create Account</h1>

			<form on:submit|preventDefault={handleSubmit} class="space-y-6">
				<!-- Username -->
				<div>
					<label for="username" class="block text-sm font-medium text-primary mb-2">
						Username
					</label>
					<input
						id="username"
						type="text"
						bind:value={username}
						class="input"
						placeholder="johndoe"
						required
						disabled={loading}
					/>
				</div>

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
						minlength="8"
						required
						disabled={loading}
					/>
					<p class="mt-1 text-xs text-muted">At least 8 characters</p>
				</div>

				<!-- Confirm Password -->
				<div>
					<label for="confirmPassword" class="block text-sm font-medium text-primary mb-2">
						Confirm Password
					</label>
					<input
						id="confirmPassword"
						type="password"
						bind:value={confirmPassword}
						class="input"
						placeholder="••••••••"
						required
						disabled={loading}
					/>
				</div>

				<!-- Invite Code (Optional) -->
				<div>
					<label for="inviteCode" class="block text-sm font-medium text-primary mb-2">
						Invite Code <span class="text-muted">(Optional)</span>
					</label>
					<input
						id="inviteCode"
						type="text"
						bind:value={inviteCode}
						class="input"
						placeholder="XXXXXXXX"
						disabled={loading}
					/>
				</div>

				<!-- Terms -->
				<div class="flex items-start">
					<input
						id="terms"
						type="checkbox"
						bind:checked={agreedToTerms}
						class="mt-1 rounded border-theme text-blue-500 focus:ring-blue-500"
						required
						disabled={loading}
					/>
					<label for="terms" class="ml-2 text-sm text-muted">
						I agree to the
						<a href="/terms" class="text-blue-500 hover:text-blue-600">Terms of Service</a>
						and
						<a href="/privacy" class="text-blue-500 hover:text-blue-600">Privacy Policy</a>
					</label>
				</div>

				<!-- Submit Button -->
				<button
					type="submit"
					class="w-full btn btn-primary"
					disabled={loading}
				>
					{loading ? 'Creating account...' : 'Create Account'}
				</button>
			</form>

			<!-- Login Link -->
			<p class="mt-6 text-center text-sm text-muted">
				Already have an account?
				<a href="/login" class="text-blue-500 hover:text-blue-600 font-medium">
					Sign in
				</a>
			</p>
		</div>
	</div>
</div>
