import { gql } from '@urql/svelte';

// Auth Queries
export const ME_QUERY = gql`
	query Me {
		me {
			id
			username
			email
			avatar
			role
			uploaded
			downloaded
			ratio
			joinedAt
			twoFactorEnabled
		}
	}
`;

// Torrent Queries
export const TORRENTS_QUERY = gql`
	query Torrents($page: Int, $limit: Int, $search: String, $category: String, $sort: String) {
		torrents(page: $page, limit: $limit, search: $search, category: $category, sort: $sort) {
			items {
				id
				name
				description
				infoHash
				size
				seeders
				leechers
				completed
				category
				tags
				uploadedBy {
					id
					username
					avatar
				}
				createdAt
				updatedAt
			}
			total
			page
			limit
			pages
		}
	}
`;

export const TORRENT_QUERY = gql`
	query Torrent($id: ID!) {
		torrent(id: $id) {
			id
			name
			description
			infoHash
			size
			seeders
			leechers
			completed
			category
			tags
			files {
				path
				size
			}
			uploadedBy {
				id
				username
				avatar
				role
			}
			createdAt
			updatedAt
			comments {
				id
				content
				user {
					id
					username
					avatar
				}
				createdAt
			}
		}
	}
`;

// User Queries
export const USER_QUERY = gql`
	query User($id: ID!) {
		user(id: $id) {
			id
			username
			email
			avatar
			role
			uploaded
			downloaded
			ratio
			joinedAt
			bio
			location
			website
			lastSeen
			stats {
				torrentsUploaded
				commentsPosted
				forumPosts
			}
		}
	}
`;

export const USERS_QUERY = gql`
	query Users($page: Int, $limit: Int, $search: String) {
		users(page: $page, limit: $limit, search: $search) {
			items {
				id
				username
				avatar
				role
				uploaded
				downloaded
				ratio
				joinedAt
			}
			total
			page
			limit
			pages
		}
	}
`;

// Forum Queries
export const FORUMS_QUERY = gql`
	query Forums {
		forums {
			id
			name
			description
			icon
			topicCount
			postCount
			lastPost {
				id
				title
				user {
					id
					username
					avatar
				}
				createdAt
			}
		}
	}
`;

export const FORUM_QUERY = gql`
	query Forum($id: ID!, $page: Int, $limit: Int) {
		forum(id: $id) {
			id
			name
			description
			icon
			topics(page: $page, limit: $limit) {
				items {
					id
					title
					isPinned
					isLocked
					views
					replyCount
					user {
						id
						username
						avatar
					}
					lastReply {
						user {
							id
							username
							avatar
						}
						createdAt
					}
					createdAt
				}
				total
				page
				limit
				pages
			}
		}
	}
`;

export const TOPIC_QUERY = gql`
	query Topic($id: ID!, $page: Int, $limit: Int) {
		topic(id: $id) {
			id
			title
			isPinned
			isLocked
			views
			forum {
				id
				name
			}
			user {
				id
				username
				avatar
				role
			}
			posts(page: $page, limit: $limit) {
				items {
					id
					content
					user {
						id
						username
						avatar
						role
						uploaded
						downloaded
						ratio
						joinedAt
					}
					createdAt
					updatedAt
				}
				total
				page
				limit
				pages
			}
			createdAt
		}
	}
`;

// Message Queries
export const CONVERSATIONS_QUERY = gql`
	query Conversations($page: Int, $limit: Int) {
		conversations(page: $page, limit: $limit) {
			items {
				id
				participant {
					id
					username
					avatar
					lastSeen
				}
				lastMessage {
					id
					content
					read
					createdAt
				}
				unreadCount
			}
			total
			page
			limit
			pages
		}
	}
`;

export const MESSAGES_QUERY = gql`
	query Messages($conversationId: ID!, $page: Int, $limit: Int) {
		messages(conversationId: $conversationId, page: $page, limit: $limit) {
			items {
				id
				content
				senderId
				read
				createdAt
			}
			total
			page
			limit
			pages
		}
	}
`;

// Stats Queries
export const STATS_QUERY = gql`
	query Stats {
		stats {
			totalUsers
			totalTorrents
			totalPeers
			totalSeeders
			totalLeechers
			totalUploaded
			totalDownloaded
			recentUsers {
				id
				username
				avatar
				joinedAt
			}
			topUploaders {
				id
				username
				avatar
				uploaded
			}
			topDownloaders {
				id
				username
				avatar
				downloaded
			}
		}
	}
`;

// Search Query
export const SEARCH_QUERY = gql`
	query Search($query: String!, $type: String, $page: Int, $limit: Int) {
		search(query: $query, type: $type, page: $page, limit: $limit) {
			torrents {
				items {
					id
					name
					description
					size
					seeders
					leechers
					category
					createdAt
				}
				total
			}
			users {
				items {
					id
					username
					avatar
					role
				}
				total
			}
			posts {
				items {
					id
					content
					topic {
						id
						title
					}
					user {
						id
						username
						avatar
					}
					createdAt
				}
				total
			}
		}
	}
`;
