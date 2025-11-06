import { Client, cacheExchange, fetchExchange, subscriptionExchange } from '@urql/svelte';
import { createClient as createWSClient } from 'graphql-ws';
import { browser } from '$app/environment';

const GRAPHQL_URL = browser
	? import.meta.env.PUBLIC_GRAPHQL_URL || 'http://localhost:4000/graphql'
	: 'http://localhost:4000/graphql';

const WS_URL = browser
	? import.meta.env.PUBLIC_GRAPHQL_WS_URL || 'ws://localhost:4000/graphql'
	: 'ws://localhost:4000/graphql';

// WebSocket client for subscriptions
const wsClient = browser
	? createWSClient({
			url: WS_URL,
			connectionParams: () => {
				const token = localStorage.getItem('token');
				return {
					authorization: token ? `Bearer ${token}` : ''
				};
			}
	  })
	: null;

// Create URQL client
export const client = new Client({
	url: GRAPHQL_URL,
	exchanges: [
		cacheExchange,
		fetchExchange,
		...(wsClient
			? [
					subscriptionExchange({
						forwardSubscription(operation) {
							return {
								subscribe: sink => {
									const dispose = wsClient.subscribe(operation, sink);
									return {
										unsubscribe: dispose
									};
								}
							};
						}
					})
			  ]
			: [])
	],
	fetchOptions: () => {
		const token = browser ? localStorage.getItem('token') : null;
		return {
			headers: {
				authorization: token ? `Bearer ${token}` : ''
			}
		};
	}
});
