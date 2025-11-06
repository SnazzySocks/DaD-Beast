import { gql } from '@urql/svelte';

// Torrent Subscriptions
export const TORRENT_UPDATED_SUBSCRIPTION = gql`
	subscription TorrentUpdated($id: ID!) {
		torrentUpdated(id: $id) {
			id
			seeders
			leechers
			completed
			updatedAt
		}
	}
`;

export const NEW_TORRENT_SUBSCRIPTION = gql`
	subscription NewTorrent {
		newTorrent {
			id
			name
			description
			size
			seeders
			leechers
			category
			uploadedBy {
				id
				username
				avatar
			}
			createdAt
		}
	}
`;

// Notification Subscriptions
export const NOTIFICATION_SUBSCRIPTION = gql`
	subscription Notification {
		notification {
			id
			type
			title
			message
			link
			read
			createdAt
		}
	}
`;

// Message Subscriptions
export const NEW_MESSAGE_SUBSCRIPTION = gql`
	subscription NewMessage {
		newMessage {
			id
			content
			senderId
			sender {
				id
				username
				avatar
			}
			createdAt
		}
	}
`;

// Chat Subscriptions
export const CHAT_MESSAGE_SUBSCRIPTION = gql`
	subscription ChatMessage($roomId: String) {
		chatMessage(roomId: $roomId) {
			id
			userId
			username
			avatar
			message
			timestamp
			roomId
		}
	}
`;

export const USER_TYPING_SUBSCRIPTION = gql`
	subscription UserTyping($roomId: String) {
		userTyping(roomId: $roomId) {
			userId
			username
			isTyping
		}
	}
`;

// Stats Subscriptions
export const STATS_UPDATED_SUBSCRIPTION = gql`
	subscription StatsUpdated {
		statsUpdated {
			totalUsers
			totalTorrents
			totalPeers
			totalSeeders
			totalLeechers
		}
	}
`;
