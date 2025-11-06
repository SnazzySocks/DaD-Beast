import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

export interface User {
	id: string;
	username: string;
	email: string;
	avatar?: string;
	role: string;
	uploaded: number;
	downloaded: number;
	ratio: number;
	joinedAt: string;
	twoFactorEnabled: boolean;
}

export interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;
}

function createAuthStore() {
	const initialState: AuthState = {
		user: null,
		token: browser ? localStorage.getItem('token') : null,
		isAuthenticated: false,
		isLoading: false
	};

	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,
		login: (user: User, token: string) => {
			if (browser) {
				localStorage.setItem('token', token);
			}
			set({
				user,
				token,
				isAuthenticated: true,
				isLoading: false
			});
		},
		logout: () => {
			if (browser) {
				localStorage.removeItem('token');
			}
			set({
				user: null,
				token: null,
				isAuthenticated: false,
				isLoading: false
			});
		},
		updateUser: (user: User) => {
			update(state => ({
				...state,
				user
			}));
		},
		setLoading: (isLoading: boolean) => {
			update(state => ({
				...state,
				isLoading
			}));
		},
		checkAuth: async () => {
			const token = browser ? localStorage.getItem('token') : null;
			if (!token) {
				return false;
			}

			// TODO: Implement token validation with backend
			// For now, just return true if token exists
			return true;
		}
	};
}

export const auth = createAuthStore();
export const user = derived(auth, $auth => $auth.user);
export const isAuthenticated = derived(auth, $auth => $auth.isAuthenticated);
