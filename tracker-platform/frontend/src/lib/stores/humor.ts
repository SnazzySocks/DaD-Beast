import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type HumorMode = 'normal' | 'dad';

function createHumorStore() {
	// Load initial value from localStorage if in browser
	const stored = browser ? localStorage.getItem('humorMode') : null;
	const initial: HumorMode = (stored as HumorMode) || 'dad'; // Default to dad humor

	const { subscribe, set, update } = writable<HumorMode>(initial);

	return {
		subscribe,
		toggle: () => {
			update(mode => {
				const newMode: HumorMode = mode === 'normal' ? 'dad' : 'normal';
				if (browser) {
					localStorage.setItem('humorMode', newMode);
				}
				return newMode;
			});
		},
		setMode: (mode: HumorMode) => {
			if (browser) {
				localStorage.setItem('humorMode', mode);
			}
			set(mode);
		},
		reset: () => {
			if (browser) {
				localStorage.removeItem('humorMode');
			}
			set('dad');
		}
	};
}

export const humorMode = createHumorStore();

// Helper function to get the appropriate text based on humor mode
export function getHumorText(normalText: string, dadText: string, mode: HumorMode): string {
	return mode === 'dad' ? dadText : normalText;
}
