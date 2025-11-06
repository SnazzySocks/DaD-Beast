import { writable } from 'svelte/store';

export interface Notification {
	id: string;
	type: 'success' | 'error' | 'warning' | 'info';
	message: string;
	duration?: number;
}

function createNotificationStore() {
	const { subscribe, update } = writable<Notification[]>([]);

	let idCounter = 0;

	const send = (message: string, type: Notification['type'] = 'info', duration = 5000) => {
		const id = `notification-${++idCounter}`;
		const notification: Notification = { id, type, message, duration };

		update(notifications => [...notifications, notification]);

		if (duration > 0) {
			setTimeout(() => {
				remove(id);
			}, duration);
		}

		return id;
	};

	const remove = (id: string) => {
		update(notifications => notifications.filter(n => n.id !== id));
	};

	return {
		subscribe,
		send,
		success: (message: string, duration?: number) => send(message, 'success', duration),
		error: (message: string, duration?: number) => send(message, 'error', duration),
		warning: (message: string, duration?: number) => send(message, 'warning', duration),
		info: (message: string, duration?: number) => send(message, 'info', duration),
		remove,
		clear: () => update(() => [])
	};
}

export const notifications = createNotificationStore();
