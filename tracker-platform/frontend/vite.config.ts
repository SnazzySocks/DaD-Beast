import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
	plugins: [
		sveltekit(),
		VitePWA({
			registerType: 'autoUpdate',
			manifest: {
				name: 'Tracker Platform',
				short_name: 'Tracker',
				description: 'Advanced BitTorrent Tracker Platform',
				theme_color: '#1e293b',
				background_color: '#0f172a',
				display: 'standalone',
				icons: [
					{
						src: '/icon-192x192.png',
						sizes: '192x192',
						type: 'image/png'
					},
					{
						src: '/icon-512x512.png',
						sizes: '512x512',
						type: 'image/png'
					}
				]
			},
			workbox: {
				globPatterns: ['**/*.{js,css,html,svg,png,woff,woff2}'],
				cleanupOutdatedCaches: true,
				clientsClaim: true
			},
			devOptions: {
				enabled: true
			}
		})
	],
	server: {
		port: 3000,
		proxy: {
			'/api': {
				target: 'http://localhost:4000',
				changeOrigin: true
			},
			'/graphql': {
				target: 'http://localhost:4000',
				changeOrigin: true,
				ws: true
			}
		}
	}
});
