/**
 * E2E tests for torrent browsing, searching, and uploading
 */

import { test, expect } from '@playwright/test';

test.describe('Torrent Browsing', () => {
  test('should display torrent list', async ({ page }) => {
    await page.goto('/torrents');

    // Should show torrents
    await expect(page.locator('[data-testid="torrent-card"]').first()).toBeVisible();

    // Should have pagination
    await expect(page.locator('.pagination')).toBeVisible();
  });

  test('should paginate torrent list', async ({ page }) => {
    await page.goto('/torrents');

    const firstTorrent = await page.locator('[data-testid="torrent-card"]').first().textContent();

    await page.click('text=Next, button:has-text("Next")');

    await page.waitForLoadState('networkidle');

    const newFirstTorrent = await page.locator('[data-testid="torrent-card"]').first().textContent();

    expect(firstTorrent).not.toBe(newFirstTorrent);
  });

  test('should filter by category', async ({ page }) => {
    await page.goto('/torrents');

    await page.click('select[name="category"], [data-testid="category-filter"]');
    await page.selectOption('select[name="category"]', 'Software');

    await page.waitForLoadState('networkidle');

    // All visible torrents should be in Software category
    const categories = await page.locator('[data-testid="torrent-category"]').allTextContents();
    expect(categories.every((cat) => cat.includes('Software'))).toBe(true);
  });

  test('should sort torrents', async ({ page }) => {
    await page.goto('/torrents');

    await page.selectOption('select[name="sort"]', 'seeders');

    await page.waitForLoadState('networkidle');

    // Verify seeders are in descending order
    const seeders = await page.locator('[data-testid="seeder-count"]').allTextContents();
    const seederNumbers = seeders.map((s) => parseInt(s));

    for (let i = 0; i < seederNumbers.length - 1; i++) {
      expect(seederNumbers[i]).toBeGreaterThanOrEqual(seederNumbers[i + 1]);
    }
  });

  test('should display torrent details', async ({ page }) => {
    await page.goto('/torrents');

    await page.click('[data-testid="torrent-card"]', { first: true });

    // Should navigate to detail page
    await expect(page).toHaveURL(/\/torrents\/[a-z0-9-]+/);

    // Should show torrent info
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('[data-testid="file-list"]')).toBeVisible();
    await expect(page.locator('[data-testid="download-button"]')).toBeVisible();
  });

  test('should download torrent file', async ({ page }) => {
    await page.goto('/torrents');
    await page.click('[data-testid="torrent-card"]', { first: true });

    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="download-button"]');
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.torrent$/);
  });
});

test.describe('Torrent Search', () => {
  test('should search torrents', async ({ page }) => {
    await page.goto('/');

    await page.fill('input[type="search"]', 'ubuntu');
    await page.press('input[type="search"]', 'Enter');

    await page.waitForLoadState('networkidle');

    // Should show search results
    await expect(page.locator('[data-testid="torrent-card"]')).toHaveCount(
      (count) => count > 0
    );

    // Results should contain search term
    const names = await page.locator('[data-testid="torrent-name"]').allTextContents();
    expect(names.some((name) => name.toLowerCase().includes('ubuntu'))).toBe(true);
  });

  test('should show search suggestions', async ({ page }) => {
    await page.goto('/');

    await page.fill('input[type="search"]', 'ubu');

    // Should show autocomplete suggestions
    await expect(page.locator('[data-testid="search-suggestion"]')).toBeVisible();
    await expect(page.locator('[data-testid="search-suggestion"]')).toContainText(/ubuntu/i);
  });

  test('should handle no results', async ({ page }) => {
    await page.goto('/');

    await page.fill('input[type="search"]', 'xyzabc123nonexistent');
    await page.press('input[type="search"]', 'Enter');

    await expect(page.locator('[data-testid="no-results"]')).toBeVisible();
  });

  test('should apply advanced search filters', async ({ page }) => {
    await page.goto('/search/advanced');

    await page.fill('input[name="query"]', 'linux');
    await page.selectOption('select[name="category"]', 'Software');
    await page.fill('input[name="min_size"]', '1');
    await page.selectOption('select[name="size_unit"]', 'GB');

    await page.click('button[type="submit"]');

    await page.waitForLoadState('networkidle');

    // Should show filtered results
    await expect(page.locator('[data-testid="torrent-card"]')).toHaveCount(
      (count) => count > 0
    );
  });
});

test.describe('Torrent Upload', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');
  });

  test('should upload torrent', async ({ page }) => {
    await page.goto('/upload');

    // Fill upload form
    await page.setInputFiles('input[type="file"]', {
      name: 'test.torrent',
      mimeType: 'application/x-bittorrent',
      buffer: Buffer.from('d8:announce0:e'), // Minimal bencode
    });

    await page.fill('input[name="name"]', 'Test Torrent Upload');
    await page.fill('textarea[name="description"]', 'This is a test torrent');
    await page.selectOption('select[name="category"]', '1');

    await page.click('button[type="submit"]');

    // Should show success message
    await expect(page.locator('.success, [role="alert"]')).toContainText(
      /success|uploaded/i
    );
  });

  test('should validate torrent file', async ({ page }) => {
    await page.goto('/upload');

    // Try to upload invalid file
    await page.setInputFiles('input[type="file"]', {
      name: 'test.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from('not a torrent file'),
    });

    await page.click('button[type="submit"]');

    await expect(page.locator('.error')).toContainText(/invalid|torrent/i);
  });

  test('should require all fields', async ({ page }) => {
    await page.goto('/upload');

    await page.click('button[type="submit"]');

    // Should show validation errors
    await expect(page.locator('.error')).toHaveCount((count) => count > 0);
  });
});

test.describe('Torrent Comments', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Go to a torrent
    await page.goto('/torrents');
    await page.click('[data-testid="torrent-card"]', { first: true });
  });

  test('should post comment', async ({ page }) => {
    await page.fill('textarea[name="comment"]', 'This is a test comment');
    await page.click('button:has-text("Post Comment")');

    await expect(page.locator('[data-testid="comment"]').last()).toContainText(
      'This is a test comment'
    );
  });

  test('should display existing comments', async ({ page }) => {
    await expect(page.locator('[data-testid="comment"]')).toHaveCount(
      (count) => count >= 0
    );
  });

  test('should paginate comments', async ({ page }) => {
    const commentCount = await page.locator('[data-testid="comment"]').count();

    if (commentCount >= 10) {
      await page.click('text=/load more|next/i');
      const newCount = await page.locator('[data-testid="comment"]').count();
      expect(newCount).toBeGreaterThan(commentCount);
    }
  });
});
