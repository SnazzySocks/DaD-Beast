import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { io, Socket } from 'socket.io-client';

export interface ChatMessage {
	id: string;
	userId: string;
	username: string;
	avatar?: string;
	message: string;
	timestamp: string;
	roomId?: string;
}

export interface WebSocketState {
	connected: boolean;
	socket: Socket | null;
	messages: ChatMessage[];
	typingUsers: Set<string>;
}

function createWebSocketStore() {
	const initialState: WebSocketState = {
		connected: false,
		socket: null,
		messages: [],
		typingUsers: new Set()
	};

	const { subscribe, set, update } = writable<WebSocketState>(initialState);

	let socket: Socket | null = null;

	const connect = (url: string, token?: string) => {
		if (!browser) return;

		socket = io(url, {
			auth: {
				token
			},
			transports: ['websocket', 'polling']
		});

		socket.on('connect', () => {
			console.log('WebSocket connected');
			update(state => ({ ...state, connected: true, socket }));
		});

		socket.on('disconnect', () => {
			console.log('WebSocket disconnected');
			update(state => ({ ...state, connected: false }));
		});

		socket.on('message', (message: ChatMessage) => {
			update(state => ({
				...state,
				messages: [...state.messages, message]
			}));
		});

		socket.on('typing', ({ userId, username }: { userId: string; username: string }) => {
			update(state => {
				const typingUsers = new Set(state.typingUsers);
				typingUsers.add(username);
				return { ...state, typingUsers };
			});

			// Remove typing indicator after 3 seconds
			setTimeout(() => {
				update(state => {
					const typingUsers = new Set(state.typingUsers);
					typingUsers.delete(username);
					return { ...state, typingUsers };
				});
			}, 3000);
		});

		socket.on('user:joined', ({ username }: { username: string }) => {
			console.log(`${username} joined the chat`);
		});

		socket.on('user:left', ({ username }: { username: string }) => {
			console.log(`${username} left the chat`);
		});
	};

	const disconnect = () => {
		if (socket) {
			socket.disconnect();
			socket = null;
			update(state => ({ ...state, connected: false, socket: null }));
		}
	};

	const sendMessage = (message: string, roomId?: string) => {
		if (socket && socket.connected) {
			socket.emit('message', { message, roomId });
		}
	};

	const joinRoom = (roomId: string) => {
		if (socket && socket.connected) {
			socket.emit('join:room', { roomId });
		}
	};

	const leaveRoom = (roomId: string) => {
		if (socket && socket.connected) {
			socket.emit('leave:room', { roomId });
		}
	};

	const sendTyping = () => {
		if (socket && socket.connected) {
			socket.emit('typing');
		}
	};

	return {
		subscribe,
		connect,
		disconnect,
		sendMessage,
		joinRoom,
		leaveRoom,
		sendTyping
	};
}

export const websocket = createWebSocketStore();
