<script lang="ts">
	import { humorMode } from '$lib/stores/humor';

	const currentYear = new Date().getFullYear();

	// Footer links with both normal and dad humor variants
	$: footerLinks = {
		about: [
			{
				normalLabel: 'About Us',
				dadLabel: "why this exists (we don't know either)",
				href: '/about'
			},
			{
				normalLabel: 'FAQ',
				dadLabel: 'questions with no good answers',
				href: '/faq'
			},
			{
				normalLabel: 'Rules',
				dadLabel: 'things that bind us',
				href: '/rules'
			},
			{
				normalLabel: 'Contact',
				dadLabel: 'bother someone else',
				href: '/contact'
			}
		],
		community: [
			{
				normalLabel: 'Forums',
				dadLabel: 'where i go to argue',
				href: '/forums'
			},
			{
				normalLabel: 'Chat',
				dadLabel: 'words i can\'t take back',
				href: '/chat'
			},
			{
				normalLabel: 'Wiki',
				dadLabel: 'collective procrastination',
				href: '/wiki'
			},
			{
				normalLabel: 'Blog',
				dadLabel: 'rambling nobody reads',
				href: '/blog'
			}
		],
		help: [
			{
				normalLabel: 'Getting Started',
				dadLabel: 'first steps toward regret',
				href: '/help/getting-started'
			},
			{
				normalLabel: 'Upload Guide',
				dadLabel: 'how to contribute to the mess',
				href: '/help/upload-guide'
			},
			{
				normalLabel: 'Download Guide',
				dadLabel: 'taking without giving back',
				href: '/help/download-guide'
			},
			{
				normalLabel: 'API Documentation',
				dadLabel: 'technical excuses',
				href: '/api/docs'
			}
		]
	};

	$: privacyLabel = $humorMode === 'dad' ? 'how we\'ll fail to protect you' : 'Privacy Policy';
	$: termsLabel = $humorMode === 'dad' ? 'legal reasons to be stuck here' : 'Terms of Service';
	$: dmcaLabel = $humorMode === 'dad' ? 'lawyers ruining everything' : 'DMCA';
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
								{$humorMode === 'dad' ? link.dadLabel : link.normalLabel}
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
								{$humorMode === 'dad' ? link.dadLabel : link.normalLabel}
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
								{$humorMode === 'dad' ? link.dadLabel : link.normalLabel}
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
					{privacyLabel}
				</a>
				<a href="/terms" class="text-sm text-muted hover:text-primary transition-colors">
					{termsLabel}
				</a>
				<a href="/dmca" class="text-sm text-muted hover:text-primary transition-colors">
					{dmcaLabel}
				</a>
			</div>
		</div>
	</div>
</footer>
