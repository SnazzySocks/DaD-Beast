/**
 * E2E tests for authentication flows
 */

import { test, expect } from '@playwright/test';
import { injectAxe, checkA11y } from 'axe-playwright';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display login page', async ({ page }) => {
    await page.goto('/login');

    await expect(page.locator('h1')).toContainText(/login|sign in/i);
    await expect(page.locator('input[type="text"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should show validation errors on empty submit', async ({ page }) => {
    await page.goto('/login');

    await page.click('button[type="submit"]');

    await expect(page.locator('.error')).toBeVisible();
  });

  test('should login with valid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');

    await page.click('button[type="submit"]');

    // Should redirect to dashboard or home
    await expect(page).toHaveURL(/\/(dashboard|home)?/);
  });

  test('should show error with invalid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[type="text"]', 'wronguser');
    await page.fill('input[type="password"]', 'wrongpass');

    await page.click('button[type="submit"]');

    await expect(page.locator('.error, [role="alert"]')).toContainText(
      /invalid|incorrect|wrong/i
    );
  });

  test('should navigate to registration page', async ({ page }) => {
    await page.goto('/login');

    await page.click('text=/sign up|register/i');

    await expect(page).toHaveURL(/\/register/);
    await expect(page.locator('h1')).toContainText(/register|sign up/i);
  });

  test('should register new user', async ({ page }) => {
    await page.goto('/register');

    const timestamp = Date.now();
    await page.fill('input[name="username"]', `testuser${timestamp}`);
    await page.fill('input[name="email"]', `test${timestamp}@example.com`);
    await page.fill('input[name="password"]', 'SecurePassword123!');
    await page.fill('input[name="password_confirmation"]', 'SecurePassword123!');

    await page.click('button[type="submit"]');

    // Should show success message or redirect
    await expect(
      page.locator('.success, [role="alert"]')
    ).toBeVisible({ timeout: 10000 });
  });

  test('should validate password strength', async ({ page }) => {
    await page.goto('/register');

    await page.fill('input[name="password"]', 'weak');

    await expect(page.locator('.password-strength')).toContainText(/weak/i);
  });

  test('should validate password confirmation match', async ({ page }) => {
    await page.goto('/register');

    await page.fill('input[name="password"]', 'Password123!');
    await page.fill('input[name="password_confirmation"]', 'Password456!');

    await page.click('button[type="submit"]');

    await expect(page.locator('.error')).toContainText(/match|same/i);
  });

  test('should enable 2FA', async ({ page, context }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Navigate to settings
    await page.goto('/settings/security');

    // Enable 2FA
    await page.click('button:has-text("Enable 2FA")');

    // Should display QR code
    await expect(page.locator('img[alt*="QR"]')).toBeVisible();
    await expect(page.locator('code')).toBeVisible(); // Secret code

    // Enter verification code (in real test, you'd generate this)
    await page.fill('input[name="verification_code"]', '123456');
    await page.click('button:has-text("Verify")');
  });

  test('should login with 2FA', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[type="text"]', 'user_with_2fa');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Should show 2FA input
    await expect(page.locator('input[name="totp_code"]')).toBeVisible();

    await page.fill('input[name="totp_code"]', '123456');
    await page.click('button[type="submit"]');
  });

  test('should logout successfully', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Logout
    await page.click('button:has-text("Logout"), a:has-text("Logout")');

    // Should redirect to home/login
    await expect(page).toHaveURL(/\/(login)?/);
  });

  test('should remember login with checkbox', async ({ page, context }) => {
    await page.goto('/login');

    await page.check('input[type="checkbox"][name="remember"]');
    await page.fill('input[type="text"]', 'testuser');
    await page.fill('input[type="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Check that a persistent cookie was set
    const cookies = await context.cookies();
    const rememberCookie = cookies.find((c) => c.name.includes('remember'));
    expect(rememberCookie).toBeDefined();
  });

  test('should be accessible', async ({ page }) => {
    await page.goto('/login');
    await injectAxe(page);
    await checkA11y(page);
  });
});
