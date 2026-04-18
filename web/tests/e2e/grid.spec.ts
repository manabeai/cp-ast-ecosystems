/**
 * E2E Tests: グリッド (H W / S_1...S_H)
 *
 * 対象: ABC390-C 相当
 * ユーザーフロー: doc/view/problem-user-flows.md §2
 *
 * テスト観点:
 * - 同一行 tuple (H, W)
 * - 文字グリッドテンプレート
 * - |S_i| = W 自動生成
 * - charset draft
 * - TeX グリッド表示 + sample grid 生成
 */
import { test, expect } from '@playwright/test';
import { EditorPage } from './fixtures/editor-page';
import {
  expectStructureContains,
  expectSampleLines,
  expectRightPanePopulated,
} from './fixtures/helpers';

test.describe('グリッド: H W / S_1...S_H', () => {
  let editor: EditorPage;

  test.beforeEach(async ({ page }) => {
    editor = new EditorPage(page);
    await editor.goto();
  });

  test('tuple [H, W] を作成する', async () => {
    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');

    // H を追加
    await editor.selectType('number');
    await editor.inputName('H');
    await editor.confirm();

    // W を追加
    await editor.inputName('W');
    await editor.confirm();

    // Structure ペインに H W が同一行に表示
    await expectStructureContains(editor, 'H');
    await expectStructureContains(editor, 'W');

    // draft: H と W の range
    const drafts = editor.getDraftConstraints();
    await expect(drafts).toHaveCount(2);
  });

  test('文字グリッドテンプレートを追加する', async () => {
    // tuple 作成
    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');
    await editor.selectType('number');
    await editor.inputName('H');
    await editor.confirm();
    await editor.inputName('W');
    await editor.confirm();

    // グリッドテンプレート
    await editor.clickHotspot('below');
    await editor.selectPopupOption('grid-template');
    await editor.selectLength('H'); // rows
    await editor.selectLength('W'); // cols
    await editor.confirm();

    // Structure ペインにグリッド表示 (S_1 ... S_H)
    await expectStructureContains(editor, 'S');

    // draft: H range, W range, charset の3つ以上
    const drafts = editor.getDraftConstraints();
    const count = await drafts.count();
    expect(count).toBeGreaterThanOrEqual(3);
  });

  test('charset を英小文字に設定する', async () => {
    // setup: tuple + grid template
    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');
    await editor.selectType('number');
    await editor.inputName('H');
    await editor.confirm();
    await editor.inputName('W');
    await editor.confirm();

    await editor.clickHotspot('below');
    await editor.selectPopupOption('grid-template');
    await editor.selectLength('H');
    await editor.selectLength('W');
    await editor.confirm();

    // charset draft をクリックして英小文字を選択
    // (charset の draft index は H range, W range の後)
    const charsetDraft = editor.page.getByTestId(/draft-constraint/).last();
    await charsetDraft.click();

    // charset プリセットから英小文字を選択する想定
    // 具体的な UI は実装時に確定するため、ここでは概要を記述
    await editor.page.getByTestId('charset-option-lowercase').click();
    await editor.page.getByTestId('constraint-confirm').click();

    // completed に英小文字制約が表示
    const completed = editor.getCompletedConstraints();
    await expect(completed.first()).toBeVisible();
  });

  test('完成状態: グリッド + 制約 + 右ペイン検証', async () => {
    // Structure 構築
    await editor.clickHotspot('below');
    await editor.selectPopupOption('tuple');
    await editor.selectType('number');
    await editor.inputName('H');
    await editor.confirm();
    await editor.inputName('W');
    await editor.confirm();

    await editor.clickHotspot('below');
    await editor.selectPopupOption('grid-template');
    await editor.selectLength('H');
    await editor.selectLength('W');
    await editor.confirm();

    // 制約を全て埋める
    await editor.fillDraftRange(0, '1', '500'); // H
    await editor.fillDraftRange(0, '1', '500'); // W
    // charset は既に設定済みと仮定

    // 右ペイン TeX 入力形式にグリッド要素
    await expect(editor.getTexInputFormat()).toContainText('H');
    await expect(editor.getTexInputFormat()).toContainText('S');

    // sample: 1行(H W) + H行(グリッド) = H+1 行以上
    await expectSampleLines(editor, 2);
  });
});
