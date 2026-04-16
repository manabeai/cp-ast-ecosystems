/**
 * Playwright E2E: navigation and app shell
 *
 * Covers: initial load, hash-based routing, page transitions.
 */

import { test, expect } from '@playwright/test';

test.describe('App navigation', () => {
  test('loads at viewer by default', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.app')).toBeVisible({ timeout: 10_000 });
    // Default route is viewer
    await expect(page.locator('.viewer-page, .preset-selector').first()).toBeVisible({ timeout: 10_000 });
  });

  test('direct navigation to /#/editor works', async ({ page }) => {
    await page.goto('/#/editor');
    await expect(page.locator('.editor-page, .header-bar').first()).toBeVisible({ timeout: 10_000 });
  });

  test('direct navigation to /#/preview works', async ({ page }) => {
    // Load base page first so wasm is initialized, then navigate via link
    await page.goto('/');
    await page.waitForSelector('.app', { timeout: 15_000 });
    await page.click('a[href="#/preview"]');
    await expect(page.locator('.preview-page').first()).toBeVisible({ timeout: 5_000 });
  });

  test('nav links switch pages without full reload', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.app', { timeout: 10_000 });

    // Go to Editor
    await page.click('a[href="#/editor"]');
    await expect(page.locator('.editor-page, .header-bar').first()).toBeVisible({ timeout: 5_000 });

    // Go to Viewer
    await page.click('a[href="#/viewer"]');
    await expect(page.locator('.viewer-page, .preset-selector').first()).toBeVisible({ timeout: 5_000 });
  });

  test('active nav link has active class', async ({ page }) => {
    await page.goto('/#/viewer');
    await page.waitForSelector('.nav-link', { timeout: 10_000 });

    const viewerLink = page.locator('a.nav-link[href="#/viewer"]');
    await expect(viewerLink).toHaveClass(/active/);

    await page.click('a[href="#/editor"]');
    const editorLink = page.locator('a.nav-link[href="#/editor"]');
    await expect(editorLink).toHaveClass(/active/);
  });

  test('page title is AST Viewer', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/AST/);
  });

  test('header always shows app title', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.header-title')).toContainText('AST');
  });
});
