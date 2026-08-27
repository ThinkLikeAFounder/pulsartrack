import { test, expect } from '@playwright/test';

test('app loads', async ({ page }) => {
  await page.goto('/');
  // Verify the page actually loaded by checking for a real element
  await expect(page.locator('body')).toBeVisible();
  // Optionally check for the page title or a key heading
  await expect(page).toHaveTitle(/.+/); // Assert title is not empty
});

test('wallet connect button exists', async ({ page }) => {
  await page.goto('/');
  // Check that wallet connect functionality is present
  const connectButton = page.getByRole('button', { name: /connect/i });
  await expect(connectButton).toBeVisible();
});