import { type Page, type Locator } from '@playwright/test';

/**
 * Page Object Model for the AST Editor.
 *
 * Encapsulates all user interactions with the editor UI.
 * data-testid selectors here define the contract between E2E tests and UI implementation.
 */
export class EditorPage {
  readonly page: Page;

  // --- Panes ---
  readonly structurePane: Locator;
  readonly constraintPane: Locator;
  readonly previewPane: Locator;

  // --- Structure interactions ---
  readonly insertionHotspots: Locator;
  readonly nodePopup: Locator;

  constructor(page: Page) {
    this.page = page;
    this.structurePane = page.getByTestId('structure-pane');
    this.constraintPane = page.getByTestId('constraint-pane');
    this.previewPane = page.getByTestId('preview-pane');
    this.insertionHotspots = page.getByTestId(/^insertion-hotspot/);
    this.nodePopup = page.getByTestId('node-popup');
  }

  async goto(): Promise<void> {
    await this.page.goto('/');
  }

  // ── Structure operations ──────────────────────────────────────────

  async clickHotspot(
    direction: 'below' | 'right' | 'inside' = 'below',
  ): Promise<void> {
    await this.page
      .getByTestId(`insertion-hotspot-${direction}`)
      .first()
      .click();
  }

  async selectPopupOption(option: string): Promise<void> {
    await this.nodePopup.waitFor({ state: 'visible' });
    await this.page.getByTestId(`popup-option-${option}`).click();
  }

  async inputName(name: string): Promise<void> {
    await this.page.getByTestId('name-input').fill(name);
  }

  async selectType(type: string): Promise<void> {
    await this.page.getByTestId('type-select').selectOption(type);
  }

  async selectLength(varName: string): Promise<void> {
    await this.page.getByTestId('length-select').selectOption(varName);
  }

  /**
   * Build an expression by selecting a variable and applying an operation.
   *
   * Flow: click count field → select variable → click variable in expression
   *       → select operation → enter operand → press Enter
   *
   * Example: buildCountExpression('N', 'subtract', '1') → N - 1
   */
  async buildCountExpression(
    baseVar: string,
    op: string,
    operand: string,
  ): Promise<void> {
    // 1. Click the count field to open variable list
    await this.page.getByTestId('count-field').click();
    // 2. Select the base variable
    await this.page.getByTestId(`count-var-option-${baseVar}`).click();
    // 3. Click the variable element in the expression to open function popup
    await this.page.getByTestId(`expression-element-${baseVar}`).click();
    // 4. Select the operation (e.g., subtract, add, multiply, divide, min, max)
    await this.page.getByTestId(`function-op-${op}`).click();
    // 5. Enter the operand value and confirm
    await this.page.getByTestId('function-operand-input').fill(operand);
    await this.page.getByTestId('function-operand-input').press('Enter');
  }

  async confirm(): Promise<void> {
    await this.page.getByTestId('confirm-button').click();
  }

  /**
   * High-level helper: add a scalar variable to the structure.
   */
  async addScalar(name: string, type: string = 'number'): Promise<void> {
    await this.clickHotspot('below');
    await this.selectPopupOption('scalar');
    await this.selectType(type);
    await this.inputName(name);
    await this.confirm();
  }

  /**
   * High-level helper: add a scalar to the right of the current node (same line).
   */
  async addScalarRight(name: string, type: string = 'number'): Promise<void> {
    await this.clickHotspot('right');
    await this.selectPopupOption('scalar');
    await this.selectType(type);
    await this.inputName(name);
    await this.confirm();
  }

  /**
   * High-level helper: add a horizontal array to the structure.
   */
  async addArray(
    name: string,
    lengthVar: string,
    type: string = 'number',
  ): Promise<void> {
    await this.clickHotspot('below');
    await this.selectPopupOption('array');
    await this.selectType(type);
    await this.inputName(name);
    await this.selectLength(lengthVar);
    await this.confirm();
  }

  // ── Constraint operations ─────────────────────────────────────────

  getDraftConstraints(): Locator {
    return this.constraintPane.getByTestId(/^draft-constraint/);
  }

  getCompletedConstraints(): Locator {
    return this.constraintPane.getByTestId(/^completed-constraint/);
  }

  async fillDraftRange(
    index: number,
    lower: string,
    upper: string,
  ): Promise<void> {
    const draft = this.page.getByTestId(`draft-constraint-${index}`);
    await draft.click();
    await this.page.getByTestId('constraint-lower-input').fill(lower);
    await this.page.getByTestId('constraint-upper-input').fill(upper);
    await this.page.getByTestId('constraint-confirm').click();
  }

  async addProperty(propertyName: string): Promise<void> {
    await this.page.getByTestId('property-shortcut').click();
    await this.page.getByTestId(`property-option-${propertyName}`).click();
  }

  async addSumBound(varName: string, upper: string): Promise<void> {
    await this.page.getByTestId('sumbound-shortcut').click();
    await this.page.getByTestId('sumbound-var-select').selectOption(varName);
    await this.page.getByTestId('sumbound-upper-input').fill(upper);
    await this.page.getByTestId('constraint-confirm').click();
  }

  // ── Right pane (Preview) assertions ───────────────────────────────

  getTexInputFormat(): Locator {
    return this.page.getByTestId('tex-input-format');
  }

  getTexConstraints(): Locator {
    return this.page.getByTestId('tex-constraints');
  }

  getSampleOutput(): Locator {
    return this.page.getByTestId('sample-output');
  }

  // ── Math editing ──────────────────────────────────────────────────

  async clickMathElement(id: string): Promise<void> {
    await this.page.getByTestId(`math-editable-${id}`).click();
  }

  async editMathValue(value: string): Promise<void> {
    await this.page.getByTestId('math-editor-input').fill(value);
    await this.page.getByTestId('math-editor-confirm').click();
  }
}
