import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'dark' | 'grey' | 'light' | 'aero' | 'coffee';

export interface ThemeConfig {
	name: string;
	label: string;
	description: string;
	class: string;
}

export const themes: Record<Theme, ThemeConfig> = {
	dark: {
		name: 'dark',
		label: 'like my mood',
		description: 'emotional darkness',
		class: 'theme-dark'
	},
	grey: {
		name: 'grey',
		label: 'colorless existence',
		description: 'dull reality',
		class: 'theme-grey'
	},
	light: {
		name: 'light',
		label: 'too bright for my soul',
		description: "can't handle optimism",
		class: 'theme-light'
	},
	aero: {
		name: 'aero',
		label: 'trying to feel something',
		description: 'desperate for sensation',
		class: 'theme-aero'
	},
	coffee: {
		name: 'coffee',
		label: 'the only thing keeping me going',
		description: 'caffeine dependency',
		class: 'theme-coffee'
	}
};

function createThemeStore() {
	const defaultTheme: Theme = 'dark';

	// Get initial theme from localStorage or use default
	const initialTheme = browser
		? (localStorage.getItem('theme') as Theme) || defaultTheme
		: defaultTheme;

	const { subscribe, set, update } = writable<Theme>(initialTheme);

	return {
		subscribe,
		set: (theme: Theme) => {
			if (browser) {
				localStorage.setItem('theme', theme);
				// Update document class
				document.documentElement.className = themes[theme].class;
			}
			set(theme);
		},
		toggle: () => {
			update(currentTheme => {
				const themeKeys = Object.keys(themes) as Theme[];
				const currentIndex = themeKeys.indexOf(currentTheme);
				const nextIndex = (currentIndex + 1) % themeKeys.length;
				const nextTheme = themeKeys[nextIndex];

				if (browser) {
					localStorage.setItem('theme', nextTheme);
					document.documentElement.className = themes[nextTheme].class;
				}

				return nextTheme;
			});
		},
		init: () => {
			if (browser) {
				const theme = (localStorage.getItem('theme') as Theme) || defaultTheme;
				document.documentElement.className = themes[theme].class;
				set(theme);
			}
		}
	};
}

export const currentTheme = createThemeStore();
