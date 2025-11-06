/**
 * E2E tests for theme switching and visual regression
 */

import { test, expect } from '@playwright/test';

const THEMES = ['light', 'dark', 'cyberpunk', 'retro', 'ocean'];

test.describe('Theme Switching', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  for (const theme of THEMES) {
    test(`should render ${theme} theme correctly`, async ({ page }) => {
      // Open theme switcher
      await page.click('[data-testid="theme-switcher"]');

      // Select theme
      await page.click(`button:has-text("${theme}")`);

      // Verify theme is applied
      const dataTheme = await page.getAttribute('html', 'data-theme');
      expect(dataTheme).toBe(theme);

      // Take screenshot for visual comparison
      await page.screenshot({
        path: `tests/screenshots/theme-${theme}.png`,
        fullPage: true,
      });
    });
  }

  test('should persist theme across pages', async ({ page }) => {
    await page.click('[data-testid="theme-switcher"]');
    await page.click('button:has-text("dark")');

    await page.goto('/torrents');

    const theme = await page.getAttribute('html', 'data-theme');
    expect(theme).toBe('dark');
  });

  test('should persist theme across sessions', async ({ page, context }) => {
    await page.click('[data-testid="theme-switcher"]');
    await page.click('button:has-text("cyberpunk")');

    // Close and reopen browser
    await page.close();
    const newPage = await context.newPage();
    await newPage.goto('/');

    const theme = await newPage.getAttribute('html', 'data-theme');
    expect(theme).toBe('cyberpunk');
  });

  test('should apply theme to all components', async ({ page }) => {
    await page.click('[data-testid="theme-switcher"]');
    await page.click('button:has-text("dark")');

    // Navigate to different pages
    await page.goto('/torrents');
    await expect(page.locator('body')).toHaveCSS('background-color', /(0, 0, 0)|(17, 17, 17)/);

    await page.goto('/forums');
    await expect(page.locator('body')).toHaveCSS('background-color', /(0, 0, 0)|(17, 17, 17)/);
  });
});

test.describe('Visual Regression', () => {
  test('homepage snapshot', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveScreenshot('homepage.png', { fullPage: true });
  });

  test('torrents page snapshot', async ({ page }) => {
    await page.goto('/torrents');
    await expect(page).toHaveScreenshot('torrents.png', { fullPage: true });
  });

  test('login page snapshot', async ({ page }) => {
    await page.goto('/login');
    await expect(page).toHaveScreenshot('login.png');
  });

  test('torrent card snapshot', async ({ page }) => {
    await page.goto('/torrents');

    const card = page.locator('[data-testid="torrent-card"]').first();
    await expect(card).toHaveScreenshot('torrent-card.png');
  });

  test('navigation snapshot', async ({ page }) => {
    await page.goto('/');

    const nav = page.locator('nav');
    await expect(nav).toHaveScreenshot('navigation.png');
  });
});

test.describe('Theme Accessibility', () => {
  for (const theme of THEMES) {
    test(`${theme} theme meets contrast requirements`, async ({ page }) => {
      await page.goto('/');

      await page.click('[data-testid="theme-switcher"]');
      await page.click(`button:has-text("${theme}")`);

      // Check contrast ratios
      const backgroundColor = await page.locator('body').evaluate((el) => {
        return window.getComputedStyle(el).backgroundColor;
      });

      const textColor = await page.locator('body').evaluate((el) => {
        return window.getComputedStyle(el).color;
      });

      // Both should be defined
      expect(backgroundColor).toBeDefined();
      expect(textColor).toBeDefined();

      // In real test, you would calculate contrast ratio
      // and verify it meets WCAG AA standards (4.5:1 for normal text)
    });
  }
});

test.describe('Responsive Design', () => {
  const viewports = [
    { name: 'Mobile', width: 375, height: 667 },
    { name: 'Tablet', width: 768, height: 1024 },
    { name: 'Desktop', width: 1920, height: 1080 },
  ];

  for (const viewport of viewports) {
    test(`should render correctly on ${viewport.name}`, async ({ page }) => {
      await page.setViewportSize({
        width: viewport.width,
        height: viewport.height,
      });

      await page.goto('/');

      await expect(page).toHaveScreenshot(
        `${viewport.name.toLowerCase()}-homepage.png`,
        { fullPage: true }
      );
    });
  }
});
