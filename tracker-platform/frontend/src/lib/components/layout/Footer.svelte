<script lang="ts">
	const currentYear = new Date().getFullYear();

	const footerLinks = {
		about: [
			{ label: "why this exists (we don't know either)", href: '/about' },
			{ label: 'questions with no good answers', href: '/faq' },
			{ label: 'things that bind us', href: '/rules' },
			{ label: 'bother someone else', href: '/contact' }
		],
		community: [
			{ label: 'where i go to argue', href: '/forums' },
			{ label: 'words i can\'t take back', href: '/chat' },
			{ label: 'collective procrastination', href: '/wiki' },
			{ label: 'rambling nobody reads', href: '/blog' }
		],
		help: [
			{ label: 'first steps toward regret', href: '/help/getting-started' },
			{ label: 'how to contribute to the mess', href: '/help/upload-guide' },
			{ label: 'taking without giving back', href: '/help/download-guide' },
			{ label: 'technical excuses', href: '/api/docs' }
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
					how we'll fail to protect you
				</a>
				<a href="/terms" class="text-sm text-muted hover:text-primary transition-colors">
					legal reasons to be stuck here
				</a>
				<a href="/dmca" class="text-sm text-muted hover:text-primary transition-colors">
					lawyers ruining everything
				</a>
			</div>
		</div>
	</div>
</footer>
