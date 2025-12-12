import { test, expect } from '@playwright/test';

test.describe('Users Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/users');
  });

  test('displays page header and add button', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Users');
    await expect(page.locator('button:has-text("Add User")')).toBeVisible();
  });

  test('displays empty state when no users exist', async ({ page }) => {
    // Wait for loading to complete
    await page.waitForSelector('.card, .table-container', { timeout: 10000 });

    // Check for empty state or table
    const emptyState = page.locator('text=No users yet');
    const table = page.locator('.table');

    // Either empty state or table should be visible
    const hasEmptyState = await emptyState.isVisible().catch(() => false);
    const hasTable = await table.isVisible().catch(() => false);

    expect(hasEmptyState || hasTable).toBe(true);
  });

  test('opens add user modal when clicking Add User button', async ({ page }) => {
    await page.click('button:has-text("Add User")');

    // Modal should open
    await expect(page.locator('.modal-content')).toBeVisible();
    await expect(page.locator('h2:has-text("Add User")')).toBeVisible();

    // Form fields should be present
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="number"]')).toBeVisible();
    await expect(page.locator('input[type="checkbox"]').first()).toBeVisible();
  });

  test('creates a new user', async ({ page }) => {
    // Open modal
    await page.click('button:has-text("Add User")');
    await expect(page.locator('.modal-content')).toBeVisible();

    // Fill form
    const testEmail = `test-${Date.now()}@example.com`;
    await page.fill('input[type="email"]', testEmail);
    await page.fill('input[type="number"]', '300');

    // Check the enabled checkbox
    const enabledCheckbox = page.locator('input[type="checkbox"]').first();
    if (!(await enabledCheckbox.isChecked())) {
      await enabledCheckbox.check();
    }

    // Submit form
    await page.click('button[type="submit"]');

    // Wait for modal to close
    await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });

    // Verify user appears in table
    await expect(page.locator(`text=${testEmail}`)).toBeVisible({ timeout: 10000 });
  });

  test('edits an existing user', async ({ page }) => {
    // First create a user if none exists
    const existingUser = page.locator('.table-row').first();
    const hasUsers = await existingUser.isVisible().catch(() => false);

    if (!hasUsers) {
      // Create a user first
      await page.click('button:has-text("Add User")');
      await page.fill('input[type="email"]', `edit-test-${Date.now()}@example.com`);
      await page.fill('input[type="number"]', '300');
      await page.click('button[type="submit"]');
      await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    }

    // Click edit button on first user
    await page.click('.table-row button:has-text("Edit")');

    // Modal should open with "Edit User" title
    await expect(page.locator('.modal-content')).toBeVisible();
    await expect(page.locator('h2:has-text("Edit User")')).toBeVisible();

    // Modify email
    const emailInput = page.locator('input[type="email"]');
    await emailInput.clear();
    const updatedEmail = `updated-${Date.now()}@example.com`;
    await emailInput.fill(updatedEmail);

    // Submit
    await page.click('button[type="submit"]');

    // Verify modal closes and changes are reflected
    await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });
    await expect(page.locator(`text=${updatedEmail}`)).toBeVisible({ timeout: 10000 });
  });

  test('deletes a user with confirmation', async ({ page }) => {
    // First create a user to delete
    await page.click('button:has-text("Add User")');
    const deleteTestEmail = `delete-test-${Date.now()}@example.com`;
    await page.fill('input[type="email"]', deleteTestEmail);
    await page.fill('input[type="number"]', '300');
    await page.click('button[type="submit"]');
    await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });

    // Wait for user to appear
    await expect(page.locator(`text=${deleteTestEmail}`)).toBeVisible({ timeout: 10000 });

    // Setup dialog handler to accept confirmation
    page.on('dialog', async dialog => {
      expect(dialog.message()).toContain('delete');
      await dialog.accept();
    });

    // Find the row with our email and click delete
    const userRow = page.locator(`.table-row:has-text("${deleteTestEmail}")`);
    await userRow.locator('button:has-text("Delete")').click();

    // Verify user is removed
    await expect(page.locator(`text=${deleteTestEmail}`)).not.toBeVisible({ timeout: 10000 });
  });

  test('closes modal when clicking Cancel', async ({ page }) => {
    // Open modal
    await page.click('button:has-text("Add User")');
    await expect(page.locator('.modal-content')).toBeVisible();

    // Click cancel
    await page.click('button:has-text("Cancel")');

    // Modal should close
    await expect(page.locator('.modal-content')).not.toBeVisible();
  });

  test('closes modal when clicking X button', async ({ page }) => {
    // Open modal
    await page.click('button:has-text("Add User")');
    await expect(page.locator('.modal-content')).toBeVisible();

    // Click X button (close button in header)
    await page.click('.modal-header button.btn-ghost');

    // Modal should close
    await expect(page.locator('.modal-content')).not.toBeVisible();
  });

  test('validates email field is required', async ({ page }) => {
    // Open modal
    await page.click('button:has-text("Add User")');
    await expect(page.locator('.modal-content')).toBeVisible();

    // Try to submit without email
    await page.fill('input[type="number"]', '300');
    await page.click('button[type="submit"]');

    // Form should not submit (modal should still be visible)
    await expect(page.locator('.modal-content')).toBeVisible();

    // HTML5 validation should show
    const emailInput = page.locator('input[type="email"]');
    await expect(emailInput).toHaveAttribute('required', '');
  });

  test('displays user status badges correctly', async ({ page }) => {
    // Create an enabled user
    await page.click('button:has-text("Add User")');
    const enabledEmail = `enabled-${Date.now()}@example.com`;
    await page.fill('input[type="email"]', enabledEmail);
    await page.fill('input[type="number"]', '300');

    // Make sure enabled is checked
    const enabledCheckbox = page.locator('input[type="checkbox"]').first();
    await enabledCheckbox.check();

    await page.click('button[type="submit"]');
    await expect(page.locator('.modal-content')).not.toBeVisible({ timeout: 10000 });

    // Verify status badge shows "Active"
    const userRow = page.locator(`.table-row:has-text("${enabledEmail}")`);
    await expect(userRow.locator('.badge-success')).toContainText('Active');
  });
});
