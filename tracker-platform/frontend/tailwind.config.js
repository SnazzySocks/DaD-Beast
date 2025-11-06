/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				// Dark Theme
				dark: {
					bg: '#0f172a',
					surface: '#1e293b',
					'surface-light': '#334155',
					text: '#f1f5f9',
					'text-muted': '#94a3b8',
					primary: '#3b82f6',
					'primary-hover': '#2563eb',
					accent: '#8b5cf6',
					border: '#334155'
				},
				// Grey Theme
				grey: {
					bg: '#1a1a1a',
					surface: '#2a2a2a',
					'surface-light': '#3a3a3a',
					text: '#e5e5e5',
					'text-muted': '#a3a3a3',
					primary: '#737373',
					'primary-hover': '#525252',
					accent: '#d4d4d4',
					border: '#404040'
				},
				// Light Theme
				light: {
					bg: '#ffffff',
					surface: '#f8fafc',
					'surface-light': '#f1f5f9',
					text: '#0f172a',
					'text-muted': '#64748b',
					primary: '#3b82f6',
					'primary-hover': '#2563eb',
					accent: '#8b5cf6',
					border: '#e2e8f0'
				},
				// Frutiger Aero Theme
				aero: {
					bg: '#e6f4ff',
					surface: 'rgba(255, 255, 255, 0.7)',
					'surface-light': 'rgba(255, 255, 255, 0.5)',
					text: '#1a5490',
					'text-muted': '#5a8fc4',
					primary: '#4dc3ff',
					'primary-hover': '#3ab0ec',
					accent: '#a4e35e',
					border: 'rgba(77, 195, 255, 0.3)',
					glow: 'rgba(77, 195, 255, 0.5)'
				},
				// Global Coffeehouse Theme
				coffee: {
					bg: '#f5e6d3',
					surface: '#ffffff',
					'surface-light': '#faf3eb',
					text: '#3e2723',
					'text-muted': '#6f4e37',
					primary: '#6f4e37',
					'primary-hover': '#5d3e2b',
					accent: '#a67c52',
					border: '#d4b896',
					wood: '#8b6f47'
				}
			},
			fontFamily: {
				sans: ['Inter', 'system-ui', 'sans-serif'],
				serif: ['Georgia', 'Cambria', 'Times New Roman', 'serif']
			},
			boxShadow: {
				'glass': '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
				'aero': '0 4px 30px rgba(77, 195, 255, 0.3)',
				'coffee': '0 2px 8px rgba(62, 39, 35, 0.15)'
			},
			backdropBlur: {
				xs: '2px'
			}
		}
	},
	plugins: [
		require('@tailwindcss/forms')
	]
};
