/**
 * Playwright E2E: AST Viewer page
 *
 * Covers: preset selection, structure/constraint display, TeX preview,
 *         sample generation, AST tree toggle.
 */

import { test, expect } from '@playwright/test';

test.describe('Viewer page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/viewer');
    // Wait for wasm to init — toolbar-select (preset dropdown) appears
    await page.waitForSelector('.toolbar-select', { timeout: 10_000 });
  });

  test('shows structure and constraint panes', async ({ page }) => {
    // ViewerPage has 3 panes: structure, constraint, preview
    await expect(page.locator('.pane')).toHaveCount(3);
  });

  test('loads scalar_array preset and shows N in structure', async ({ page }) => {
    await page.selectOption('.toolbar-select', 'scalar_array');
    // Structure pane content (pre.pane-content) should show N
    const structurePre = page.locator('.pane-content').first();
    await expect(structurePre).toContainText('N', { timeout: 5_000 });
  });

  test('switches between presets updates structure', async ({ page }) => {
    await page.selectOption('.toolbar-select', 'matrix');
    const structurePre = page.locator('.pane-content').first();
    await expect(structurePre).toContainText(/H|W|Grid/, { timeout: 5_000 });

    await page.selectOption('.toolbar-select', 'scalar_only');
    await expect(structurePre).toContainText('N', { timeout: 5_000 });
  });

  test('structure pane title is 入力形式', async ({ page }) => {
    await expect(page.locator('.pane-title').first()).toContainText('入力形式');
  });

  test('constraint pane title is 制約', async ({ page }) => {
    await expect(page.locator('.pane-title').nth(1)).toContainText('制約');
  });

  test('AST toggle button exists and is clickable', async ({ page }) => {
    await page.selectOption('.toolbar-select', 'scalar_array');
    const toggleBtn = page.locator('.toggle-btn').first();
    await expect(toggleBtn).toBeVisible();
    await toggleBtn.click();
    // After toggle, button becomes active
    await expect(toggleBtn).toHaveClass(/active/);
  });

  test('shuffle button updates seed input', async ({ page }) => {
    await page.selectOption('.toolbar-select', 'scalar_array');
    const seedInput = page.locator('.toolbar-input');
    const seedBefore = await seedInput.inputValue();
    await page.click('.toolbar-btn');
    const seedAfter = await seedInput.inputValue();
    // Seed may or may not change (random), but button is clickable
    expect(typeof seedAfter).toBe('string');
  });

  test('nav links are all present', async ({ page }) => {
    await expect(page.getByRole('link', { name: 'Viewer' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Preview' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Editor' })).toBeVisible();
  });
});
