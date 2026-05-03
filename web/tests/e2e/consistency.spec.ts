import { test, expect } from '@playwright/test';
import { EditorPage } from './fixtures/editor-page';

test.describe('Web editor consistency', () => {
  let editor: EditorPage;

  test.beforeEach(async ({ page }) => {
    editor = new EditorPage(page);
    await editor.goto();
  });

  test('app title and header use the Random Test Creator name', async () => {
    await expect(editor.page).toHaveTitle('Random Test Creator');
    await expect(editor.page.getByRole('heading', { level: 1 })).toHaveText('Random Test Creator');
  });

  test('draft constraints suppress sample until required bounds are filled', async () => {
    await editor.addScalar('N');

    await expect(editor.getDraftConstraints()).toHaveCount(1);
    await expect(editor.getSampleOutput()).toBeEmpty();

    await editor.openDraft(0);
    await editor.fillBoundLiteral('lower', '2');
    await editor.fillBoundLiteral('upper', '2');
    await editor.confirmConstraint();

    await expect(editor.getSampleOutput()).not.toBeEmpty();
  });

  test('length and count variable pickers only show Int scalar variables', async () => {
    await editor.addScalar('N');
    await editor.openDraft(0);
    await editor.fillBoundLiteral('lower', '1');
    await editor.fillBoundLiteral('upper', '3');
    await editor.confirmConstraint();

    await editor.addScalar('C', 'char');
    await editor.page.getByTestId('constraint-item-1').click();
    await editor.page.getByTestId('charset-option-lowercase').click();
    await editor.confirmConstraint();

    await editor.clickHotspot('below');
    await editor.selectPopupOption('array');
    await expect(editor.page.getByTestId('length-var-option-N')).toBeVisible();
    await expect(editor.page.getByTestId('length-var-option-C')).toHaveCount(0);
    await editor.closePopupByEscape();

    await editor.clickHotspot('below');
    await editor.selectPopupOption('repeat');
    await expect(editor.page.getByTestId('count-var-option-N')).toBeVisible();
    await expect(editor.page.getByTestId('count-var-option-C')).toHaveCount(0);
  });

  test('completed constraints can be edited and deleted back to draft slots', async () => {
    await editor.addScalar('N');
    await editor.openDraft(0);
    await editor.fillBoundLiteral('lower', '1');
    await editor.fillBoundLiteral('upper', '3');
    await editor.confirmConstraint();

    await expect(editor.page.getByTestId('constraint-item-0')).toHaveAttribute('data-constraint-status', 'completed');

    await editor.page.getByTestId('completed-constraint-0').click();
    await editor.page.getByTestId('constraint-lower-expression').click();
    await editor.page.getByTestId('function-op-add').click();
    await editor.page.getByTestId('function-operand-input').fill('1');
    await editor.page.getByTestId('function-operand-input').press('Enter');
    await editor.confirmConstraint();

    await expect(editor.page.getByTestId('constraint-item-0')).toContainText('2');

    await editor.page.getByTestId('delete-constraint-0').click();
    await expect(editor.page.getByTestId('constraint-item-0')).toHaveAttribute('data-constraint-status', 'draft');
    await expect(editor.getSampleOutput()).toBeEmpty();
  });

  test('function expressions in constraints still allow sample generation', async () => {
    await editor.addScalar('N');
    await editor.openDraft(0);
    await editor.fillBoundLiteral('lower', '1');
    await editor.fillBoundLiteral('upper', '3');
    await editor.confirmConstraint();

    await editor.addScalarRight('M');
    await editor.openDraft(0);
    await editor.fillBoundLiteral('lower', '1');
    await editor.fillBoundVar('upper', 'N');
    await editor.applyBoundFunction('upper', 'add', '2');
    await editor.confirmConstraint();

    await expect(editor.getSampleOutput()).not.toBeEmpty();
  });

  test('header reset clears the current document', async () => {
    await editor.addScalar('N');
    await expect(editor.structurePane).toContainText('N');

    await editor.page.getByTestId('reset-document-button').click();

    await expect(editor.structurePane).not.toContainText('N');
    await expect(editor.insertionHotspots.first()).toBeVisible();
  });
});
