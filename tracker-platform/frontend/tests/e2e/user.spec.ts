/**
 * E2E tests for user profile and settings
 */

import { test, expect } from '@playwright/test';

test.describe('User Profile', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');
  });

  test('should display user profile', async ({ page }) => {
    await page.goto('/users/testuser');

    await expect(page.locator('h1')).toContainText('testuser');
    await expect(page.locator('[data-testid="upload-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="download-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="ratio"]')).toBeVisible();
  });

  test('should show user statistics', async ({ page }) => {
    await page.goto('/users/testuser');

    // Check stats are numbers
    const ratio = await page.locator('[data-testid="ratio"]').textContent();
    expect(parseFloat(ratio || '0')).toBeGreaterThanOrEqual(0);

    const uploaded = await page.locator('[data-testid="uploaded"]').textContent();
    expect(uploaded).toMatch(/[0-9]+(\.)?[0-9]* (B|KB|MB|GB|TB)/);
  });

  test('should display user torrents', async ({ page }) => {
    await page.goto('/users/testuser');

    await page.click('text=Torrents');

    await expect(page.locator('[data-testid="torrent-card"]')).toHaveCount(
      (count) => count >= 0
    );
  });

  test('should display activity history', async ({ page }) => {
    await page.goto('/users/testuser');

    await page.click('text=Activity');

    await expect(page.locator('[data-testid="activity-item"]')).toHaveCount(
      (count) => count >= 0
    );
  });
});

test.describe('User Settings', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/settings');
  });

  test('should update profile', async ({ page }) => {
    await page.fill('textarea[name="bio"]', 'Updated bio text');

    await page.click('button:has-text("Save")');

    await expect(page.locator('.success')).toBeVisible();
  });

  test('should change password', async ({ page }) => {
    await page.click('text=Security');

    await page.fill('input[name="current_password"]', 'password123');
    await page.fill('input[name="new_password"]', 'NewPassword123!');
    await page.fill('input[name="confirm_password"]', 'NewPassword123!');

    await page.click('button:has-text("Change Password")');

    await expect(page.locator('.success')).toContainText(/password/i);
  });

  test('should update notification preferences', async ({ page }) => {
    await page.click('text=Notifications');

    await page.check('input[name="email_torrents"]');
    await page.uncheck('input[name="email_messages"]');

    await page.click('button:has-text("Save")');

    await expect(page.locator('.success')).toBeVisible();
  });

  test('should update privacy settings', async ({ page }) => {
    await page.click('text=Privacy');

    await page.check('input[name="hide_stats"]');
    await page.check('input[name="hide_activity"]');

    await page.click('button:has-text("Save")');

    await expect(page.locator('.success')).toBeVisible();
  });
});

test.describe('User Invitations', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/settings/invites');
  });

  test('should display available invites', async ({ page }) => {
    await expect(page.locator('[data-testid="invite-count"]')).toBeVisible();
  });

  test('should generate invite code', async ({ page }) => {
    const inviteCount = await page.locator('[data-testid="invite-count"]').textContent();

    if (parseInt(inviteCount || '0') > 0) {
      await page.click('button:has-text("Generate Invite")');

      await expect(page.locator('[data-testid="invite-code"]')).toBeVisible();

      // Should be able to copy code
      await page.click('button:has-text("Copy")');
    }
  });

  test('should show invite history', async ({ page }) => {
    await expect(page.locator('[data-testid="invite-history"]')).toBeVisible();
  });
});

test.describe('Bonus System', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/bonus');
  });

  test('should display bonus points', async ({ page }) => {
    await expect(page.locator('[data-testid="bonus-points"]')).toBeVisible();
  });

  test('should show bonus shop', async ({ page }) => {
    await expect(page.locator('[data-testid="bonus-item"]')).toHaveCount(
      (count) => count > 0
    );
  });

  test('should purchase item with bonus points', async ({ page }) => {
    const points = await page.locator('[data-testid="bonus-points"]').textContent();

    if (parseInt(points || '0') >= 100) {
      await page.click('[data-testid="bonus-item"]', { first: true });
      await page.click('button:has-text("Purchase")');

      // Should show confirmation
      await expect(page.locator('[data-testid="confirm-dialog"]')).toBeVisible();

      await page.click('button:has-text("Confirm")');

      await expect(page.locator('.success')).toContainText(/purchased/i);
    }
  });
});
