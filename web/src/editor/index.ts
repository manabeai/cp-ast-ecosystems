export type {
  FullProjection,
  OutlineNode,
  Diagnostic,
  CompletenessInfo,
  NodeDetailProjection,
  SlotInfo,
  SlotKind,
  SlotId,
  ConstraintSummary,
  HoleCandidate,
  ExprCandidateMenu,
  ReferenceCandidate,
  ConstraintTargetMenu,
  EditorAction,
  FillContent,
  DraftConstraint,
} from './types';

export {
  editorDocumentJson,
  selectedNodeId,
  selectedConstraintId,
  draftConstraint,
  lastError,
  projection,
  isComplete,
  holeCount,
  initEditor,
  initEditorFromJson,
} from './state';

export { dispatchAction } from './actions';