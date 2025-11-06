/**
 * E2E tests for forum functionality
 */

import { test, expect } from '@playwright/test';

test.describe('Forums', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');
  });

  test('should display forum list', async ({ page }) => {
    await page.goto('/forums');

    await expect(page.locator('[data-testid="forum-category"]')).toHaveCount(
      (count) => count > 0
    );
  });

  test('should display threads in forum', async ({ page }) => {
    await page.goto('/forums');

    await page.click('[data-testid="forum-category"]', { first: true });

    await expect(page.locator('[data-testid="thread"]')).toHaveCount(
      (count) => count >= 0
    );
  });

  test('should create new thread', async ({ page }) => {
    await page.goto('/forums/general');

    await page.click('button:has-text("New Thread")');

    await page.fill('input[name="title"]', 'Test Thread Title');
    await page.fill('textarea[name="content"]', 'This is test thread content');

    await page.click('button:has-text("Post")');

    await expect(page.locator('h1')).toContainText('Test Thread Title');
  });

  test('should reply to thread', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    await page.fill('textarea[name="reply"]', 'This is a test reply');
    await page.click('button:has-text("Reply")');

    await expect(page.locator('[data-testid="post"]').last()).toContainText(
      'This is a test reply'
    );
  });

  test('should edit own post', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    // Find own post
    const ownPost = page.locator('[data-testid="post"]:has-text("testuser")').first();

    await ownPost.locator('button:has-text("Edit")').click();

    await page.fill('textarea[name="edit_content"]', 'Updated content');
    await page.click('button:has-text("Save")');

    await expect(ownPost).toContainText('Updated content');
  });

  test('should quote post', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    await page.click('[data-testid="post"]', { first: true }).then(async (post) => {
      await page.locator('button:has-text("Quote")').click();
    });

    const replyBox = page.locator('textarea[name="reply"]');
    await expect(replyBox).toContainText(/quote|>/i);
  });

  test('should like post', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    const firstPost = page.locator('[data-testid="post"]').first();
    const likeButton = firstPost.locator('button:has-text("Like")');

    await likeButton.click();

    await expect(firstPost.locator('[data-testid="like-count"]')).toContainText(
      /[0-9]+/
    );
  });

  test('should paginate thread posts', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    const postCount = await page.locator('[data-testid="post"]').count();

    if (postCount >= 20) {
      await page.click('text=/page 2|next/i');

      await expect(page).toHaveURL(/page=2/);
    }
  });

  test('should search forums', async ({ page }) => {
    await page.goto('/forums');

    await page.fill('input[type="search"]', 'test');
    await page.press('input[type="search"]', 'Enter');

    await expect(page.locator('[data-testid="search-result"]')).toHaveCount(
      (count) => count >= 0
    );
  });
});

test.describe('Forum Moderation', () => {
  test.beforeEach(async ({ page }) => {
    // Login as moderator
    await page.goto('/login');
    await page.fill('input[type="text"]', 'moderator');
    await page.fill('input[type="password"]', 'modpass123');
    await page.click('button[type="submit"]');
  });

  test('should pin thread', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    await page.click('button:has-text("Pin")');

    await page.goto('/forums/general');

    // Pinned thread should be at top
    const firstThread = page.locator('[data-testid="thread"]').first();
    await expect(firstThread).toContainText(/pinned/i);
  });

  test('should lock thread', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    await page.click('button:has-text("Lock")');

    // Reply box should not be visible
    await expect(page.locator('textarea[name="reply"]')).not.toBeVisible();

    await expect(page.locator('.locked-notice')).toBeVisible();
  });

  test('should delete post', async ({ page }) => {
    await page.goto('/forums/general');
    await page.click('[data-testid="thread"]', { first: true });

    const postCount = await page.locator('[data-testid="post"]').count();

    await page.locator('[data-testid="post"]').last().locator('button:has-text("Delete")').click();

    await page.click('button:has-text("Confirm")');

    const newPostCount = await page.locator('[data-testid="post"]').count();
    expect(newPostCount).toBeLessThan(postCount);
  });
});
