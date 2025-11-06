/**
 * E2E tests for real-time chat
 */

import { test, expect } from '@playwright/test';

test.describe('Chat', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    await page.goto('/chat');
  });

  test('should display chat interface', async ({ page }) => {
    await expect(page.locator('[data-testid="chat-container"]')).toBeVisible();
    await expect(page.locator('[data-testid="chat-input"]')).toBeVisible();
    await expect(page.locator('[data-testid="send-button"]')).toBeVisible();
  });

  test('should display room list', async ({ page }) => {
    await expect(page.locator('[data-testid="chat-room"]')).toHaveCount(
      (count) => count > 0
    );
  });

  test('should join chat room', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    await expect(page.locator('[data-testid="room-title"]')).toBeVisible();
    await expect(page.locator('[data-testid="user-list"]')).toBeVisible();
  });

  test('should send message', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    const message = `Test message ${Date.now()}`;
    await page.fill('[data-testid="chat-input"]', message);
    await page.click('[data-testid="send-button"]');

    // Message should appear in chat
    await expect(page.locator('[data-testid="chat-message"]').last()).toContainText(
      message
    );
  });

  test('should send message with Enter key', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    const message = `Enter key test ${Date.now()}`;
    await page.fill('[data-testid="chat-input"]', message);
    await page.press('[data-testid="chat-input"]', 'Enter');

    await expect(page.locator('[data-testid="chat-message"]').last()).toContainText(
      message
    );
  });

  test('should display user list', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    await expect(page.locator('[data-testid="chat-user"]')).toHaveCount(
      (count) => count > 0
    );

    // Own username should be in list
    await expect(page.locator('[data-testid="chat-user"]:has-text("testuser")')).toBeVisible();
  });

  test('should scroll to bottom on new message', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    // Send message
    await page.fill('[data-testid="chat-input"]', 'Scroll test');
    await page.click('[data-testid="send-button"]');

    // Last message should be in viewport
    const lastMessage = page.locator('[data-testid="chat-message"]').last();
    await expect(lastMessage).toBeInViewport();
  });

  test('should format message links', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    await page.fill('[data-testid="chat-input"]', 'Check out https://example.com');
    await page.click('[data-testid="send-button"]');

    // Link should be clickable
    const link = page.locator('[data-testid="chat-message"]').last().locator('a');
    await expect(link).toHaveAttribute('href', 'https://example.com');
  });

  test('should show typing indicator', async ({ page, context }) => {
    // Open two pages (two users)
    const page2 = await context.newPage();

    // Login as different user
    await page2.goto('/login');
    await page2.fill('input[type="text"]', 'user2');
    await page2.fill('input[type="password"]', 'password123');
    await page2.click('button[type="submit"]');

    await page2.goto('/chat');
    await page2.click('[data-testid="chat-room"]', { first: true });

    // User 2 starts typing
    await page2.fill('[data-testid="chat-input"]', 'typing...');

    // User 1 should see typing indicator
    await expect(page.locator('[data-testid="typing-indicator"]')).toContainText(
      /user2.*typing/i
    );

    await page2.close();
  });

  test('should leave room', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    await page.click('button:has-text("Leave")');

    // Should return to room list
    await expect(page.locator('[data-testid="chat-room"]')).toBeVisible();
  });

  test('should handle rate limiting', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    // Send many messages rapidly
    for (let i = 0; i < 10; i++) {
      await page.fill('[data-testid="chat-input"]', `Message ${i}`);
      await page.click('[data-testid="send-button"]');
    }

    // Should show rate limit warning
    await expect(page.locator('[data-testid="rate-limit-warning"]')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should persist chat history', async ({ page }) => {
    await page.click('[data-testid="chat-room"]', { first: true });

    const message = `History test ${Date.now()}`;
    await page.fill('[data-testid="chat-input"]', message);
    await page.click('[data-testid="send-button"]');

    // Refresh page
    await page.reload();

    await page.click('[data-testid="chat-room"]', { first: true });

    // Message should still be there
    await expect(page.locator('[data-testid="chat-message"]')).toContainText(message);
  });
});
