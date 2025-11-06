import { gql } from '@urql/svelte';

// Auth Mutations
export const LOGIN_MUTATION = gql`
	mutation Login($email: String!, $password: String!, $twoFactorCode: String) {
		login(email: $email, password: $password, twoFactorCode: $twoFactorCode) {
			token
			user {
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
	}
`;

export const REGISTER_MUTATION = gql`
	mutation Register($username: String!, $email: String!, $password: String!, $inviteCode: String) {
		register(username: $username, email: $email, password: $password, inviteCode: $inviteCode) {
			token
			user {
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
	}
`;

export const FORGOT_PASSWORD_MUTATION = gql`
	mutation ForgotPassword($email: String!) {
		forgotPassword(email: $email) {
			success
			message
		}
	}
`;

export const RESET_PASSWORD_MUTATION = gql`
	mutation ResetPassword($token: String!, $password: String!) {
		resetPassword(token: $token, password: $password) {
			success
			message
		}
	}
`;

export const VERIFY_EMAIL_MUTATION = gql`
	mutation VerifyEmail($token: String!) {
		verifyEmail(token: $token) {
			success
			message
		}
	}
`;

// Torrent Mutations
export const UPLOAD_TORRENT_MUTATION = gql`
	mutation UploadTorrent($input: TorrentInput!) {
		uploadTorrent(input: $input) {
			id
			name
			description
			infoHash
			size
			category
			tags
			createdAt
		}
	}
`;

export const UPDATE_TORRENT_MUTATION = gql`
	mutation UpdateTorrent($id: ID!, $input: TorrentUpdateInput!) {
		updateTorrent(id: $id, input: $input) {
			id
			name
			description
			category
			tags
			updatedAt
		}
	}
`;

export const DELETE_TORRENT_MUTATION = gql`
	mutation DeleteTorrent($id: ID!) {
		deleteTorrent(id: $id) {
			success
			message
		}
	}
`;

export const ADD_COMMENT_MUTATION = gql`
	mutation AddComment($torrentId: ID!, $content: String!) {
		addComment(torrentId: $torrentId, content: $content) {
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
`;

// User Mutations
export const UPDATE_PROFILE_MUTATION = gql`
	mutation UpdateProfile($input: ProfileInput!) {
		updateProfile(input: $input) {
			id
			username
			email
			avatar
			bio
			location
			website
		}
	}
`;

export const CHANGE_PASSWORD_MUTATION = gql`
	mutation ChangePassword($currentPassword: String!, $newPassword: String!) {
		changePassword(currentPassword: $currentPassword, newPassword: $newPassword) {
			success
			message
		}
	}
`;

export const ENABLE_TWO_FACTOR_MUTATION = gql`
	mutation EnableTwoFactor {
		enableTwoFactor {
			secret
			qrCode
		}
	}
`;

export const VERIFY_TWO_FACTOR_MUTATION = gql`
	mutation VerifyTwoFactor($code: String!) {
		verifyTwoFactor(code: $code) {
			success
			backupCodes
		}
	}
`;

export const DISABLE_TWO_FACTOR_MUTATION = gql`
	mutation DisableTwoFactor($password: String!) {
		disableTwoFactor(password: $password) {
			success
			message
		}
	}
`;

// Forum Mutations
export const CREATE_TOPIC_MUTATION = gql`
	mutation CreateTopic($forumId: ID!, $title: String!, $content: String!) {
		createTopic(forumId: $forumId, title: $title, content: $content) {
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
`;

export const CREATE_POST_MUTATION = gql`
	mutation CreatePost($topicId: ID!, $content: String!) {
		createPost(topicId: $topicId, content: $content) {
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
`;

export const UPDATE_POST_MUTATION = gql`
	mutation UpdatePost($id: ID!, $content: String!) {
		updatePost(id: $id, content: $content) {
			id
			content
			updatedAt
		}
	}
`;

export const DELETE_POST_MUTATION = gql`
	mutation DeletePost($id: ID!) {
		deletePost(id: $id) {
			success
			message
		}
	}
`;

// Message Mutations
export const SEND_MESSAGE_MUTATION = gql`
	mutation SendMessage($recipientId: ID!, $content: String!) {
		sendMessage(recipientId: $recipientId, content: $content) {
			id
			content
			senderId
			recipientId
			read
			createdAt
		}
	}
`;

export const MARK_MESSAGE_READ_MUTATION = gql`
	mutation MarkMessageRead($messageId: ID!) {
		markMessageRead(messageId: $messageId) {
			success
		}
	}
`;

export const DELETE_MESSAGE_MUTATION = gql`
	mutation DeleteMessage($messageId: ID!) {
		deleteMessage(messageId: $messageId) {
			success
			message
		}
	}
`;
