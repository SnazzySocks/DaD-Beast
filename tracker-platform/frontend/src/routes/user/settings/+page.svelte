<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { notifications } from '$lib/stores/notifications';
	import { mutation } from '@urql/svelte';
	import { UPDATE_PROFILE_MUTATION, CHANGE_PASSWORD_MUTATION } from '$lib/graphql/mutations';

	let activeTab = 'profile';

	// Profile settings
	let username = $auth.user?.username || '';
	let email = $auth.user?.email || '';
	let bio = '';
	let location = '';
	let website = '';

	// Password settings
	let currentPassword = '';
	let newPassword = '';
	let confirmPassword = '';

	let updating = false;

	const updateProfileMutation = mutation({ query: UPDATE_PROFILE_MUTATION });
	const changePasswordMutation = mutation({ query: CHANGE_PASSWORD_MUTATION });

	async function handleUpdateProfile() {
		updating = true;

		try {
			const result = await updateProfileMutation({
				input: { username, email, bio, location, website }
			});

			if (result.error) {
				notifications.error(result.error.message);
			} else {
				notifications.success('Profile updated successfully');
				if (result.data?.updateProfile) {
					auth.updateUser(result.data.updateProfile);
				}
			}
		} catch (error) {
			notifications.error('Failed to update profile');
		} finally {
			updating = false;
		}
	}

	async function handleChangePassword() {
		if (newPassword !== confirmPassword) {
			notifications.error('Passwords do not match');
			return;
		}

		if (newPassword.length < 8) {
			notifications.error('Password must be at least 8 characters');
			return;
		}

		updating = true;

		try {
			const result = await changePasswordMutation({
				currentPassword,
				newPassword
			});

			if (result.error) {
				notifications.error(result.error.message);
			} else {
				notifications.success('Password changed successfully');
				currentPassword = '';
				newPassword = '';
				confirmPassword = '';
			}
		} catch (error) {
			notifications.error('Failed to change password');
		} finally {
			updating = false;
		}
	}
</script>

<svelte:head>
	<title>Settings - Tracker Platform</title>
</svelte:head>

<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	<h1 class="text-3xl font-bold text-primary mb-8">Settings</h1>

	<div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
		<!-- Sidebar -->
		<div class="lg:col-span-1">
			<div class="card p-4">
				<nav class="space-y-1">
					<button
						on:click={() => activeTab = 'profile'}
						class="w-full text-left px-4 py-2 rounded-lg transition-colors
							{activeTab === 'profile' ? 'bg-blue-500 text-white' : 'text-muted hover:bg-surface-light'}"
					>
						Profile
					</button>
					<button
						on:click={() => activeTab = 'security'}
						class="w-full text-left px-4 py-2 rounded-lg transition-colors
							{activeTab === 'security' ? 'bg-blue-500 text-white' : 'text-muted hover:bg-surface-light'}"
					>
						Security
					</button>
					<button
						on:click={() => activeTab = 'notifications'}
						class="w-full text-left px-4 py-2 rounded-lg transition-colors
							{activeTab === 'notifications' ? 'bg-blue-500 text-white' : 'text-muted hover:bg-surface-light'}"
					>
						Notifications
					</button>
				</nav>
			</div>
		</div>

		<!-- Content -->
		<div class="lg:col-span-3">
			{#if activeTab === 'profile'}
				<div class="card p-6">
					<h2 class="text-xl font-bold text-primary mb-6">Profile Settings</h2>

					<form on:submit|preventDefault={handleUpdateProfile} class="space-y-6">
						<div>
							<label for="username" class="block text-sm font-medium text-primary mb-2">
								Username
							</label>
							<input
								id="username"
								type="text"
								bind:value={username}
								class="input"
								disabled={updating}
							/>
						</div>

						<div>
							<label for="email" class="block text-sm font-medium text-primary mb-2">
								Email
							</label>
							<input
								id="email"
								type="email"
								bind:value={email}
								class="input"
								disabled={updating}
							/>
						</div>

						<div>
							<label for="bio" class="block text-sm font-medium text-primary mb-2">
								Bio
							</label>
							<textarea
								id="bio"
								bind:value={bio}
								class="input min-h-[100px] resize-none"
								placeholder="Tell us about yourself..."
								disabled={updating}
							></textarea>
						</div>

						<div>
							<label for="location" class="block text-sm font-medium text-primary mb-2">
								Location
							</label>
							<input
								id="location"
								type="text"
								bind:value={location}
								class="input"
								placeholder="City, Country"
								disabled={updating}
							/>
						</div>

						<div>
							<label for="website" class="block text-sm font-medium text-primary mb-2">
								Website
							</label>
							<input
								id="website"
								type="url"
								bind:value={website}
								class="input"
								placeholder="https://example.com"
								disabled={updating}
							/>
						</div>

						<button type="submit" class="btn btn-primary" disabled={updating}>
							{updating ? 'Saving...' : 'Save Changes'}
						</button>
					</form>
				</div>

			{:else if activeTab === 'security'}
				<div class="card p-6">
					<h2 class="text-xl font-bold text-primary mb-6">Security Settings</h2>

					<form on:submit|preventDefault={handleChangePassword} class="space-y-6 mb-8">
						<h3 class="text-lg font-semibold text-primary">Change Password</h3>

						<div>
							<label for="currentPassword" class="block text-sm font-medium text-primary mb-2">
								Current Password
							</label>
							<input
								id="currentPassword"
								type="password"
								bind:value={currentPassword}
								class="input"
								disabled={updating}
							/>
						</div>

						<div>
							<label for="newPassword" class="block text-sm font-medium text-primary mb-2">
								New Password
							</label>
							<input
								id="newPassword"
								type="password"
								bind:value={newPassword}
								class="input"
								minlength="8"
								disabled={updating}
							/>
						</div>

						<div>
							<label for="confirmPassword" class="block text-sm font-medium text-primary mb-2">
								Confirm Password
							</label>
							<input
								id="confirmPassword"
								type="password"
								bind:value={confirmPassword}
								class="input"
								disabled={updating}
							/>
						</div>

						<button type="submit" class="btn btn-primary" disabled={updating}>
							{updating ? 'Updating...' : 'Change Password'}
						</button>
					</form>

					<div class="border-t border-theme pt-8">
						<h3 class="text-lg font-semibold text-primary mb-4">Two-Factor Authentication</h3>
						<p class="text-sm text-muted mb-4">
							Add an extra layer of security to your account
						</p>
						<button class="btn btn-secondary">
							{$auth.user?.twoFactorEnabled ? 'Disable 2FA' : 'Enable 2FA'}
						</button>
					</div>
				</div>

			{:else if activeTab === 'notifications'}
				<div class="card p-6">
					<h2 class="text-xl font-bold text-primary mb-6">Notification Settings</h2>

					<div class="space-y-4">
						<label class="flex items-center justify-between">
							<span class="text-sm text-primary">Email notifications</span>
							<input type="checkbox" class="rounded border-theme" checked />
						</label>

						<label class="flex items-center justify-between">
							<span class="text-sm text-primary">New torrent uploads</span>
							<input type="checkbox" class="rounded border-theme" checked />
						</label>

						<label class="flex items-center justify-between">
							<span class="text-sm text-primary">Forum replies</span>
							<input type="checkbox" class="rounded border-theme" checked />
						</label>

						<label class="flex items-center justify-between">
							<span class="text-sm text-primary">Private messages</span>
							<input type="checkbox" class="rounded border-theme" checked />
						</label>
					</div>

					<button class="btn btn-primary mt-6">Save Preferences</button>
				</div>
			{/if}
		</div>
	</div>
</div>
