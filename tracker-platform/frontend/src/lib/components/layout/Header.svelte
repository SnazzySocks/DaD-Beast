<script lang="ts">
	import { page } from '$app/stores';
	import { auth, isAuthenticated } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import ThemeSwitcher from './ThemeSwitcher.svelte';

	let mobileMenuOpen = false;
	let userMenuOpen = false;

	function toggleMobileMenu() {
		mobileMenuOpen = !mobileMenuOpen;
	}

	function toggleUserMenu() {
		userMenuOpen = !userMenuOpen;
	}

	function handleLogout() {
		auth.logout();
		goto('/login');
	}

	const navItems = [
		{ href: '/', label: 'Home' },
		{ href: '/torrents', label: 'Torrents' },
		{ href: '/forums', label: 'Forums' },
		{ href: '/chat', label: 'Chat' },
		{ href: '/stats', label: 'Stats' }
	];
</script>

<header class="bg-surface border-b border-theme sticky top-0 z-50 backdrop-blur-sm">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="flex justify-between items-center h-16">
			<!-- Logo -->
			<div class="flex items-center">
				<a href="/" class="flex items-center space-x-2">
					<div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
						<span class="text-white font-bold text-xl">T</span>
					</div>
					<span class="text-xl font-bold text-primary hidden sm:block">Tracker</span>
				</a>
			</div>

			<!-- Desktop Navigation -->
			<nav class="hidden md:flex items-center space-x-1">
				{#each navItems as item}
					<a
						href={item.href}
						class="px-3 py-2 rounded-lg text-sm font-medium transition-colors
							{$page.url.pathname === item.href
							? 'bg-surface-light text-primary'
							: 'text-muted hover:bg-surface-light hover:text-primary'}"
					>
						{item.label}
					</a>
				{/each}
			</nav>

			<!-- Right Side -->
			<div class="flex items-center space-x-4">
				<!-- Search -->
				<a
					href="/search"
					class="p-2 text-muted hover:text-primary transition-colors"
					aria-label="Search"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
						/>
					</svg>
				</a>

				<!-- Theme Switcher -->
				<ThemeSwitcher />

				{#if $isAuthenticated}
					<!-- User Menu -->
					<div class="relative">
						<button
							on:click={toggleUserMenu}
							class="flex items-center space-x-2 p-1 rounded-lg hover:bg-surface-light transition-colors"
						>
							<img
								src={$auth.user?.avatar || '/default-avatar.png'}
								alt={$auth.user?.username}
								class="w-8 h-8 rounded-full object-cover"
							/>
							<span class="hidden sm:block text-sm font-medium text-primary">
								{$auth.user?.username}
							</span>
							<svg class="w-4 h-4 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
							</svg>
						</button>

						{#if userMenuOpen}
							<div class="absolute right-0 mt-2 w-48 bg-surface border border-theme rounded-lg shadow-lg py-1 animate-fade-in">
								<a href="/user/{$auth.user?.id}" class="block px-4 py-2 text-sm text-primary hover:bg-surface-light">
									Profile
								</a>
								<a href="/user/settings" class="block px-4 py-2 text-sm text-primary hover:bg-surface-light">
									Settings
								</a>
								<a href="/messages" class="block px-4 py-2 text-sm text-primary hover:bg-surface-light">
									Messages
								</a>
								<a href="/upload" class="block px-4 py-2 text-sm text-primary hover:bg-surface-light">
									Upload
								</a>
								<hr class="my-1 border-theme" />
								<button
									on:click={handleLogout}
									class="block w-full text-left px-4 py-2 text-sm text-red-500 hover:bg-surface-light"
								>
									Logout
								</button>
							</div>
						{/if}
					</div>
				{:else}
					<!-- Auth Buttons -->
					<div class="hidden md:flex items-center space-x-2">
						<a href="/login" class="btn btn-secondary text-sm">Login</a>
						<a href="/register" class="btn btn-primary text-sm">Register</a>
					</div>
				{/if}

				<!-- Mobile Menu Button -->
				<button
					on:click={toggleMobileMenu}
					class="md:hidden p-2 text-muted hover:text-primary transition-colors"
					aria-label="Menu"
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						{#if mobileMenuOpen}
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						{:else}
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
						{/if}
					</svg>
				</button>
			</div>
		</div>

		<!-- Mobile Menu -->
		{#if mobileMenuOpen}
			<div class="md:hidden py-4 border-t border-theme animate-slide-in">
				<nav class="space-y-1">
					{#each navItems as item}
						<a
							href={item.href}
							class="block px-3 py-2 rounded-lg text-sm font-medium
								{$page.url.pathname === item.href
								? 'bg-surface-light text-primary'
								: 'text-muted hover:bg-surface-light hover:text-primary'}"
						>
							{item.label}
						</a>
					{/each}
				</nav>

				{#if !$isAuthenticated}
					<div class="mt-4 space-y-2">
						<a href="/login" class="block btn btn-secondary text-sm text-center">Login</a>
						<a href="/register" class="block btn btn-primary text-sm text-center">Register</a>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</header>
