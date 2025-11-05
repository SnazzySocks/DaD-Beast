//! Permission-based authorization system
//!
//! This module implements a comprehensive permission system inspired by Gazelle,
//! supporting role-based access control (RBAC) for the tracker platform.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// User permissions for fine-grained access control
///
/// Each permission grants specific capabilities within the tracker platform.
/// Multiple permissions can be combined to create user roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    // Site administration
    /// Full administrative access to all site functions
    SiteAdmin,
    /// Access to view site logs and analytics
    SiteLogs,
    /// Ability to edit site settings and configuration
    SiteSettings,
    /// Manage user accounts (create, edit, delete)
    ManageUsers,
    /// View user sessions and login history
    ViewUserSessions,

    // Moderation
    /// General forum and torrent moderation
    ForumModerator,
    /// Delete forum posts and threads
    DeleteForumPosts,
    /// Edit forum posts by other users
    EditForumPosts,
    /// Moderate torrent uploads (approve/reject)
    TorrentModerator,
    /// Delete torrents from the tracker
    DeleteTorrents,
    /// Edit torrent metadata
    EditTorrents,

    // User management
    /// Ban or suspend user accounts
    BanUsers,
    /// Issue warnings to users
    WarnUsers,
    /// View user IP addresses
    ViewUserIPs,
    /// Manage user invitations
    ManageInvites,
    /// Grant or revoke user permissions
    ManagePermissions,

    // Content management
    /// Upload new torrents
    UploadTorrent,
    /// Create forum threads and posts
    CreateForumPost,
    /// Create and manage requests
    CreateRequest,
    /// Vote on requests
    VoteRequest,
    /// Fill torrent requests
    FillRequest,

    // Advanced features
    /// Access to API endpoints
    ApiAccess,
    /// View advanced site statistics
    ViewStatistics,
    /// Download torrents
    Download,
    /// Automatic download privileges (no ratio requirements)
    AutoDownload,
    /// Bypass upload waiting period
    BypassUploadWait,

    // Special privileges
    /// Immunity from automated systems (ratio watch, etc.)
    ImmunityAutomated,
    /// Donor status (cosmetic and perks)
    Donor,
    /// Access to invite system
    SendInvites,
}

impl Permission {
    /// Returns all available permissions
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::SiteAdmin,
            Permission::SiteLogs,
            Permission::SiteSettings,
            Permission::ManageUsers,
            Permission::ViewUserSessions,
            Permission::ForumModerator,
            Permission::DeleteForumPosts,
            Permission::EditForumPosts,
            Permission::TorrentModerator,
            Permission::DeleteTorrents,
            Permission::EditTorrents,
            Permission::BanUsers,
            Permission::WarnUsers,
            Permission::ViewUserIPs,
            Permission::ManageInvites,
            Permission::ManagePermissions,
            Permission::UploadTorrent,
            Permission::CreateForumPost,
            Permission::CreateRequest,
            Permission::VoteRequest,
            Permission::FillRequest,
            Permission::ApiAccess,
            Permission::ViewStatistics,
            Permission::Download,
            Permission::AutoDownload,
            Permission::BypassUploadWait,
            Permission::ImmunityAutomated,
            Permission::Donor,
            Permission::SendInvites,
        ]
    }

    /// Returns a human-readable description of the permission
    pub fn description(&self) -> &'static str {
        match self {
            Permission::SiteAdmin => "Full administrative access to all site functions",
            Permission::SiteLogs => "Access to view site logs and analytics",
            Permission::SiteSettings => "Ability to edit site settings and configuration",
            Permission::ManageUsers => "Manage user accounts (create, edit, delete)",
            Permission::ViewUserSessions => "View user sessions and login history",
            Permission::ForumModerator => "General forum and torrent moderation",
            Permission::DeleteForumPosts => "Delete forum posts and threads",
            Permission::EditForumPosts => "Edit forum posts by other users",
            Permission::TorrentModerator => "Moderate torrent uploads (approve/reject)",
            Permission::DeleteTorrents => "Delete torrents from the tracker",
            Permission::EditTorrents => "Edit torrent metadata",
            Permission::BanUsers => "Ban or suspend user accounts",
            Permission::WarnUsers => "Issue warnings to users",
            Permission::ViewUserIPs => "View user IP addresses",
            Permission::ManageInvites => "Manage user invitations",
            Permission::ManagePermissions => "Grant or revoke user permissions",
            Permission::UploadTorrent => "Upload new torrents",
            Permission::CreateForumPost => "Create forum threads and posts",
            Permission::CreateRequest => "Create and manage requests",
            Permission::VoteRequest => "Vote on requests",
            Permission::FillRequest => "Fill torrent requests",
            Permission::ApiAccess => "Access to API endpoints",
            Permission::ViewStatistics => "View advanced site statistics",
            Permission::Download => "Download torrents",
            Permission::AutoDownload => "Automatic download privileges (no ratio requirements)",
            Permission::BypassUploadWait => "Bypass upload waiting period",
            Permission::ImmunityAutomated => "Immunity from automated systems (ratio watch, etc.)",
            Permission::Donor => "Donor status (cosmetic and perks)",
            Permission::SendInvites => "Access to invite system",
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Permission::SiteAdmin => "Site Admin",
            Permission::SiteLogs => "Site Logs",
            Permission::SiteSettings => "Site Settings",
            Permission::ManageUsers => "Manage Users",
            Permission::ViewUserSessions => "View User Sessions",
            Permission::ForumModerator => "Forum Moderator",
            Permission::DeleteForumPosts => "Delete Forum Posts",
            Permission::EditForumPosts => "Edit Forum Posts",
            Permission::TorrentModerator => "Torrent Moderator",
            Permission::DeleteTorrents => "Delete Torrents",
            Permission::EditTorrents => "Edit Torrents",
            Permission::BanUsers => "Ban Users",
            Permission::WarnUsers => "Warn Users",
            Permission::ViewUserIPs => "View User IPs",
            Permission::ManageInvites => "Manage Invites",
            Permission::ManagePermissions => "Manage Permissions",
            Permission::UploadTorrent => "Upload Torrent",
            Permission::CreateForumPost => "Create Forum Post",
            Permission::CreateRequest => "Create Request",
            Permission::VoteRequest => "Vote Request",
            Permission::FillRequest => "Fill Request",
            Permission::ApiAccess => "API Access",
            Permission::ViewStatistics => "View Statistics",
            Permission::Download => "Download",
            Permission::AutoDownload => "Auto Download",
            Permission::BypassUploadWait => "Bypass Upload Wait",
            Permission::ImmunityAutomated => "Immunity Automated",
            Permission::Donor => "Donor",
            Permission::SendInvites => "Send Invites",
        };
        write!(f, "{}", name)
    }
}

/// A set of permissions for a user
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    /// Create a permission set from a vector of permissions
    pub fn from_vec(permissions: Vec<Permission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    /// Add a permission to the set
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission from the set
    pub fn remove(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
    }

    /// Check if the set contains a specific permission
    pub fn has(&self, permission: Permission) -> bool {
        // Site admins have all permissions
        if self.permissions.contains(&Permission::SiteAdmin) {
            return true;
        }
        self.permissions.contains(&permission)
    }

    /// Check if the set contains any of the specified permissions
    pub fn has_any(&self, permissions: &[Permission]) -> bool {
        // Site admins have all permissions
        if self.permissions.contains(&Permission::SiteAdmin) {
            return true;
        }
        permissions.iter().any(|p| self.permissions.contains(p))
    }

    /// Check if the set contains all of the specified permissions
    pub fn has_all(&self, permissions: &[Permission]) -> bool {
        // Site admins have all permissions
        if self.permissions.contains(&Permission::SiteAdmin) {
            return true;
        }
        permissions.iter().all(|p| self.permissions.contains(p))
    }

    /// Get all permissions in the set
    pub fn all(&self) -> Vec<Permission> {
        self.permissions.iter().copied().collect()
    }

    /// Check if the permission set is empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Get the number of permissions in the set
    pub fn len(&self) -> usize {
        self.permissions.len()
    }
}

/// Predefined user roles with standard permission sets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// System administrator with full access
    Admin,
    /// Site moderator with moderation capabilities
    Moderator,
    /// Trusted user with extended privileges
    PowerUser,
    /// Standard user with basic access
    User,
    /// New user with restricted access
    NewUser,
    /// Disabled user with no access
    Disabled,
}

impl Role {
    /// Get the default permissions for a role
    pub fn permissions(&self) -> PermissionSet {
        match self {
            Role::Admin => PermissionSet::from_vec(vec![
                Permission::SiteAdmin,
            ]),
            Role::Moderator => PermissionSet::from_vec(vec![
                Permission::SiteLogs,
                Permission::ViewUserSessions,
                Permission::ForumModerator,
                Permission::DeleteForumPosts,
                Permission::EditForumPosts,
                Permission::TorrentModerator,
                Permission::DeleteTorrents,
                Permission::EditTorrents,
                Permission::BanUsers,
                Permission::WarnUsers,
                Permission::ViewUserIPs,
                Permission::UploadTorrent,
                Permission::CreateForumPost,
                Permission::CreateRequest,
                Permission::VoteRequest,
                Permission::FillRequest,
                Permission::ApiAccess,
                Permission::ViewStatistics,
                Permission::Download,
                Permission::SendInvites,
            ]),
            Role::PowerUser => PermissionSet::from_vec(vec![
                Permission::UploadTorrent,
                Permission::CreateForumPost,
                Permission::CreateRequest,
                Permission::VoteRequest,
                Permission::FillRequest,
                Permission::ApiAccess,
                Permission::Download,
                Permission::AutoDownload,
                Permission::BypassUploadWait,
                Permission::SendInvites,
            ]),
            Role::User => PermissionSet::from_vec(vec![
                Permission::UploadTorrent,
                Permission::CreateForumPost,
                Permission::CreateRequest,
                Permission::VoteRequest,
                Permission::FillRequest,
                Permission::Download,
            ]),
            Role::NewUser => PermissionSet::from_vec(vec![
                Permission::CreateForumPost,
                Permission::VoteRequest,
                Permission::Download,
            ]),
            Role::Disabled => PermissionSet::new(),
        }
    }

    /// Get a human-readable name for the role
    pub fn name(&self) -> &'static str {
        match self {
            Role::Admin => "Administrator",
            Role::Moderator => "Moderator",
            Role::PowerUser => "Power User",
            Role::User => "User",
            Role::NewUser => "New User",
            Role::Disabled => "Disabled",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_basic() {
        let mut perms = PermissionSet::new();
        assert!(perms.is_empty());

        perms.add(Permission::Download);
        assert!(perms.has(Permission::Download));
        assert_eq!(perms.len(), 1);

        perms.remove(Permission::Download);
        assert!(!perms.has(Permission::Download));
        assert!(perms.is_empty());
    }

    #[test]
    fn test_site_admin_has_all_permissions() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::SiteAdmin);

        // Site admin should have access to any permission
        assert!(perms.has(Permission::Download));
        assert!(perms.has(Permission::BanUsers));
        assert!(perms.has(Permission::ManageUsers));
    }

    #[test]
    fn test_permission_set_has_any() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::Download);
        perms.add(Permission::UploadTorrent);

        assert!(perms.has_any(&[Permission::Download, Permission::BanUsers]));
        assert!(!perms.has_any(&[Permission::BanUsers, Permission::ManageUsers]));
    }

    #[test]
    fn test_permission_set_has_all() {
        let mut perms = PermissionSet::new();
        perms.add(Permission::Download);
        perms.add(Permission::UploadTorrent);

        assert!(perms.has_all(&[Permission::Download, Permission::UploadTorrent]));
        assert!(!perms.has_all(&[Permission::Download, Permission::BanUsers]));
    }

    #[test]
    fn test_role_permissions() {
        let admin_perms = Role::Admin.permissions();
        assert!(admin_perms.has(Permission::SiteAdmin));
        assert!(admin_perms.has(Permission::Download)); // Via SiteAdmin

        let user_perms = Role::User.permissions();
        assert!(user_perms.has(Permission::Download));
        assert!(!user_perms.has(Permission::BanUsers));

        let disabled_perms = Role::Disabled.permissions();
        assert!(disabled_perms.is_empty());
    }

    #[test]
    fn test_permission_description() {
        assert!(!Permission::Download.description().is_empty());
        assert!(!Permission::SiteAdmin.description().is_empty());
    }

    #[test]
    fn test_role_name() {
        assert_eq!(Role::Admin.name(), "Administrator");
        assert_eq!(Role::User.name(), "User");
    }
}
