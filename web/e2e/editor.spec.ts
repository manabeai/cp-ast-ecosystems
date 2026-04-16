/**
 * Playwright E2E: AST Editor page
 *
 * Covers: page navigation, header bar, structure pane with Sequence root,
 *         node selection, add-node actions, constraint diagnostics.
 */

import { test, expect } from '@playwright/test';

test.describe('Editor page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/editor');
    // Wait for editor to initialize — HeaderBar title should appear
    await expect(page.locator('.header-bar-title, h2').filter({ hasText: /Editor/i })).toBeVisible({ timeout: 10_000 });
  });

  test('renders editor layout with three panes', async ({ page }) => {
    // Structure, Detail, Constraint panes
    await expect(page.locator('.pane')).toHaveCount(3, { timeout: 5_000 });
  });

  test('header bar shows completeness indicator', async ({ page }) => {
    const indicator = page.locator('.completeness-indicator');
    await expect(indicator).toBeVisible();
    // Should show either ✓ (complete) or ⚠ (incomplete)
    await expect(indicator).toContainText(/✓|⚠/);
  });

  test('structure pane shows Sequence root node', async ({ page }) => {
    const structurePane = page.locator('.pane').first();
    await expect(structurePane.getByText('📝 Structure')).toBeVisible();
    // Root Sequence node appears — use first() to avoid strict mode violation
    await expect(structurePane.getByText('Sequence').first()).toBeVisible();
  });

  test('clicking a node in structure pane selects it', async ({ page }) => {
    const nodeContent = page.locator('.node-content').first();
    await nodeContent.click();
    // Selected class applied
    await expect(nodeContent).toHaveClass(/selected/);
  });

  test('detail pane shows content when node selected', async ({ page }) => {
    // Click the Sequence root
    await page.locator('.node-content').first().click();
    const detailPane = page.locator('.pane').nth(1);
    await expect(detailPane).toBeVisible();
    // Detail pane title
    await expect(detailPane.locator('.pane-title')).toBeVisible();
  });

  test('structure pane has AddNodeMenu', async ({ page }) => {
    const addMenu = page.locator('.structure-actions, .add-node-menu, [class*="add"]').first();
    await expect(addMenu).toBeVisible();
  });

  test('constraint pane is visible', async ({ page }) => {
    const constraintPane = page.locator('.pane').last();
    await expect(constraintPane).toBeVisible();
  });

  test('bottom panel is visible', async ({ page }) => {
    await expect(page.locator('.bottom-panel, [class*="bottom"]').first()).toBeVisible();
  });

  test('navigation back to viewer works', async ({ page }) => {
    await page.click('a[href="#/viewer"]');
    await expect(page).toHaveURL(/#\/viewer/);
    await expect(page.locator('.viewer-page, .preset-selector').first()).toBeVisible({ timeout: 5_000 });
  });
});
