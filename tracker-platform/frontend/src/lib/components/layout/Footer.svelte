<script lang="ts">
	const currentYear = new Date().getFullYear();

	const footerLinks = {
		about: [
			{ label: 'About Us', href: '/about' },
			{ label: 'FAQ', href: '/faq' },
			{ label: 'Rules', href: '/rules' },
			{ label: 'Contact', href: '/contact' }
		],
		community: [
			{ label: 'Forums', href: '/forums' },
			{ label: 'Chat', href: '/chat' },
			{ label: 'Wiki', href: '/wiki' },
			{ label: 'Blog', href: '/blog' }
		],
		help: [
			{ label: 'Getting Started', href: '/help/getting-started' },
			{ label: 'Upload Guide', href: '/help/upload-guide' },
			{ label: 'Download Guide', href: '/help/download-guide' },
			{ label: 'API Docs', href: '/api/docs' }
		]
	};
</script>

<footer class="bg-surface border-t border-theme mt-auto">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
		<div class="grid grid-cols-1 md:grid-cols-4 gap-8">
			<!-- Brand -->
			<div class="col-span-1">
				<div class="flex items-center space-x-2 mb-4">
					<div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
						<span class="text-white font-bold text-xl">T</span>
					</div>
					<span class="text-xl font-bold text-primary">Tracker</span>
				</div>
				<p class="text-sm text-muted">
					Advanced BitTorrent tracker platform with modern features and elegant design.
				</p>
			</div>

			<!-- About -->
			<div>
				<h3 class="text-sm font-semibold text-primary uppercase mb-4">About</h3>
				<ul class="space-y-2">
					{#each footerLinks.about as link}
						<li>
							<a href={link.href} class="text-sm text-muted hover:text-primary transition-colors">
								{link.label}
							</a>
						</li>
					{/each}
				</ul>
			</div>

			<!-- Community -->
			<div>
				<h3 class="text-sm font-semibold text-primary uppercase mb-4">Community</h3>
				<ul class="space-y-2">
					{#each footerLinks.community as link}
						<li>
							<a href={link.href} class="text-sm text-muted hover:text-primary transition-colors">
								{link.label}
							</a>
						</li>
					{/each}
				</ul>
			</div>

			<!-- Help -->
			<div>
				<h3 class="text-sm font-semibold text-primary uppercase mb-4">Help</h3>
				<ul class="space-y-2">
					{#each footerLinks.help as link}
						<li>
							<a href={link.href} class="text-sm text-muted hover:text-primary transition-colors">
								{link.label}
							</a>
						</li>
					{/each}
				</ul>
			</div>
		</div>

		<!-- Bottom Bar -->
		<div class="mt-8 pt-8 border-t border-theme flex flex-col sm:flex-row justify-between items-center">
			<p class="text-sm text-muted">
				&copy; {currentYear} Tracker Platform. All rights reserved.
			</p>

			<div class="flex space-x-6 mt-4 sm:mt-0">
				<a href="/privacy" class="text-sm text-muted hover:text-primary transition-colors">
					Privacy Policy
				</a>
				<a href="/terms" class="text-sm text-muted hover:text-primary transition-colors">
					Terms of Service
				</a>
				<a href="/dmca" class="text-sm text-muted hover:text-primary transition-colors">
					DMCA
				</a>
			</div>
		</div>
	</div>
</footer>
