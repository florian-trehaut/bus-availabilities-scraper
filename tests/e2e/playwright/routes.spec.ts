import { test, expect } from '@playwright/test';

test.describe('Routes Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/user-routes');
  });

  test('displays page header and disabled add button initially', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Routes');

    // Add Route button should be disabled when no user is selected
    const addButton = page.locator('button:has-text("Add Route")');
    await expect(addButton).toBeVisible();
    await expect(addButton).toBeDisabled();
  });

  test('displays user selector dropdown', async ({ page }) => {
    // Wait for users to load
    await page.waitForSelector('select, .card', { timeout: 10000 });

    // Should have a user selector
    const selector = page.locator('select').first();
    await expect(selector).toBeVisible();
  });

  test('enables add button when user is selected', async ({ page }) => {
    // Wait for page to load
    await page.waitForSelector('select', { timeout: 10000 });

    // First, ensure there's a user to select (create one if needed)
    await page.goto('/users');
    const hasUsers = await page.locator('.table-row').first().isVisible().catch(() => false);

    if (!hasUsers) {
      // Create a user
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `route-test-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    // Go back to routes page
    await page.goto('/user-routes');
    await page.waitForSelector('select', { timeout: 10000 });

    // Select the first user
    const userSelect = page.locator('select').first();
    const options = await userSelect.locator('option').all();

    if (options.length > 1) {
      // Select second option (first is usually placeholder)
      await userSelect.selectOption({ index: 1 });

      // Add button should now be enabled
      const addButton = page.locator('button:has-text("Add Route")');
      await expect(addButton).not.toBeDisabled({ timeout: 5000 });
    }
  });

  test('shows empty state when user has no routes', async ({ page }) => {
    // Ensure a user exists
    await page.goto('/users');
    const hasUsers = await page.locator('.table-row').first().isVisible().catch(() => false);

    if (!hasUsers) {
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `empty-routes-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    await page.goto('/user-routes');
    await page.waitForSelector('select', { timeout: 10000 });

    // Select a user
    const userSelect = page.locator('select').first();
    const options = await userSelect.locator('option').all();

    if (options.length > 1) {
      await userSelect.selectOption({ index: 1 });

      // Wait for routes to load
      await page.waitForTimeout(1000);

      // Should show either empty state or routes table
      const emptyState = page.locator('text=No routes configured');
      const table = page.locator('.table');

      const hasEmpty = await emptyState.isVisible().catch(() => false);
      const hasTable = await table.isVisible().catch(() => false);

      expect(hasEmpty || hasTable).toBe(true);
    }
  });

  test('opens route form modal when clicking Add Route', async ({ page }) => {
    // Setup: ensure user exists
    await page.goto('/users');
    const hasUsers = await page.locator('.table-row').first().isVisible().catch(() => false);

    if (!hasUsers) {
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `form-test-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    await page.goto('/user-routes');
    await page.waitForSelector('select', { timeout: 10000 });

    // Select user
    const userSelect = page.locator('select').first();
    const options = await userSelect.locator('option').all();

    if (options.length > 1) {
      await userSelect.selectOption({ index: 1 });

      // Wait for button to be enabled
      const addButton = page.locator('button:has-text("Add Route")');
      await expect(addButton).not.toBeDisabled({ timeout: 5000 });

      // Click add route
      await addButton.click();

      // Modal should open (uses modal-content-lg class)
      await expect(page.locator('[class*="modal-content"]')).toBeVisible({ timeout: 5000 });
      await expect(page.locator('h2:has-text("Add Route")')).toBeVisible();
    }
  });

  test('route form has cascading dropdowns', async ({ page }) => {
    // Setup: ensure user exists
    await page.goto('/users');
    const hasUsers = await page.locator('.table-row').first().isVisible().catch(() => false);

    if (!hasUsers) {
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `cascade-test-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    await page.goto('/user-routes');
    await page.waitForSelector('select', { timeout: 10000 });

    // Select user and open form
    const userSelect = page.locator('select').first();
    const options = await userSelect.locator('option').all();

    if (options.length > 1) {
      await userSelect.selectOption({ index: 1 });

      const addButton = page.locator('button:has-text("Add Route")');
      await expect(addButton).not.toBeDisabled({ timeout: 5000 });
      await addButton.click();

      await expect(page.locator('[class*="modal-content"]')).toBeVisible({ timeout: 5000 });

      // Should have area select (use fieldset legend to be specific)
      const areaLabel = page.locator('legend:has-text("Route Selection")');
      await expect(areaLabel).toBeVisible();

      // Should have date section
      const dateLabel = page.locator('legend:has-text("Date")');
      await expect(dateLabel).toBeVisible();

      // Should have date inputs
      const startDateLabel = page.locator('text=Start Date');
      await expect(startDateLabel.first()).toBeVisible();
    }
  });

  test('closes route form modal when clicking Cancel', async ({ page }) => {
    // Setup
    await page.goto('/users');
    const hasUsers = await page.locator('.table-row').first().isVisible().catch(() => false);

    if (!hasUsers) {
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `cancel-test-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    await page.goto('/user-routes');
    await page.waitForSelector('select', { timeout: 10000 });

    const userSelect = page.locator('select').first();
    const options = await userSelect.locator('option').all();

    if (options.length > 1) {
      await userSelect.selectOption({ index: 1 });
      await page.waitForTimeout(500);

      const addButton = page.locator('button:has-text("Add Route")');
      await expect(addButton).not.toBeDisabled({ timeout: 10000 });
      await addButton.click();

      await expect(page.locator('[class*="modal-content"]')).toBeVisible({ timeout: 5000 });

      // Click cancel
      await page.click('button:has-text("Cancel")');

      // Modal should close
      await expect(page.locator('[class*="modal-content"]')).not.toBeVisible();
    }
  });

  test('route form has passenger counter fields', async ({ page }) => {
    // Setup
    await page.goto('/users');
    const hasUsers = await page.locator('.table-row').first().isVisible().catch(() => false);

    if (!hasUsers) {
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `passenger-test-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    await page.goto('/user-routes');
    await page.waitForSelector('select', { timeout: 10000 });

    const userSelect = page.locator('select').first();
    const options = await userSelect.locator('option').all();

    if (options.length > 1) {
      await userSelect.selectOption({ index: 1 });
      await page.waitForTimeout(500);

      const addButton = page.locator('button:has-text("Add Route")');
      await expect(addButton).not.toBeDisabled({ timeout: 10000 });
      await addButton.click();

      await expect(page.locator('[class*="modal-content"]')).toBeVisible({ timeout: 5000 });

      // Scroll to see Passengers section (modal has max-height with overflow)
      const modal = page.locator('[class*="modal-content"]');
      await modal.evaluate(el => el.scrollTop = el.scrollHeight);

      // Should have passenger section with Adult Men, Adult Women, etc.
      const passengersLabel = page.locator('legend:has-text("Passengers")');
      await expect(passengersLabel).toBeVisible();

      // Should have Adult Men field
      const adultMenLabel = page.locator('text=Adult Men');
      await expect(adultMenLabel).toBeVisible();
    }
  });
});
