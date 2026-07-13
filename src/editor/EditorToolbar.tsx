import type { Editor } from "@tiptap/react";
import { useEditorState } from "@tiptap/react";
import type { LucideIcon } from "lucide-react";
import type { FocusEvent, KeyboardEvent } from "react";
import { useLayoutEffect, useRef } from "react";
import {
  Bold,
  Heading1,
  Heading2,
  Italic,
  List,
  ListChecks,
  ListOrdered,
  Quote,
  Redo2,
  Strikethrough,
  Undo2,
} from "lucide-react";

import {
  FONT_FAMILIES,
  MAX_FONT_SIZE_POINTS,
  MIN_FONT_SIZE_POINTS,
  isFontFamilyId,
  isFontSizePoints,
} from "../documents/textFormatting";
import {
  DOCUMENT_FONT_FAMILY,
  DOCUMENT_FONT_SIZE_POINTS,
  MIXED_FONT_VALUE,
  RESET_FONT_VALUE,
  effectiveFontControlState,
} from "./fontControlState";

interface EditorToolbarProps {
  editor: Editor | null;
  formattingReviewOpen: boolean;
  onToggleFormattingReview: () => void;
}

interface ToolbarButtonProps {
  active?: boolean;
  controls?: string;
  disabled?: boolean;
  expanded?: boolean;
  icon: LucideIcon;
  label: string;
  onPress: () => void;
}

interface ToolbarState {
  blockquote: boolean;
  bold: boolean;
  bulletList: boolean;
  canRedo: boolean;
  canUndo: boolean;
  fontFamily: string;
  fontSize: string;
  headingOne: boolean;
  headingTwo: boolean;
  italic: boolean;
  orderedList: boolean;
  strike: boolean;
}
type ToolbarNavigationKey = "ArrowLeft" | "ArrowRight" | "End" | "Home";

const EMPTY_TOOLBAR_STATE = {
  blockquote: false,
  bold: false,
  bulletList: false,
  canRedo: false,
  canUndo: false,
  fontFamily: DOCUMENT_FONT_FAMILY,
  fontSize: String(DOCUMENT_FONT_SIZE_POINTS),
  headingOne: false,
  headingTwo: false,
  italic: false,
  orderedList: false,
  strike: false,
};

export function EditorToolbar(props: EditorToolbarProps) {
  const state = useToolbarState(props.editor);
  const toolbarRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => normalizeToolbarTabStop(toolbarRef.current), [
    state.canRedo,
    state.canUndo,
  ]);

  return (
    <div className="editor-formatting-row">
      <div
        ref={toolbarRef}
        className="editor-toolbar"
        role="toolbar"
        aria-label="Text formatting"
        aria-orientation="horizontal"
        onKeyDown={navigateToolbar}
      >
        <HistoryTools editor={props.editor} state={state} />
        <span className="toolbar-separator" aria-hidden="true" />
        <InlineTools editor={props.editor} state={state} />
        <span className="toolbar-separator" aria-hidden="true" />
        <StructureTools editor={props.editor} state={state} />
        <span className="toolbar-separator" aria-hidden="true" />
        <ReviewTools
          isOpen={props.formattingReviewOpen}
          onToggle={props.onToggleFormattingReview}
        />
      </div>
      <FontTools editor={props.editor} state={state} />
    </div>
  );
}

function FontTools(props: { editor: Editor | null; state: ToolbarState }) {
  return (
    <div className="font-tools" role="group" aria-label="Font formatting">
      <select
        className="font-tools__family"
        aria-label="Font family"
        value={props.state.fontFamily}
        disabled={!props.editor}
        onChange={(event) => setFontFamily(props.editor, event.currentTarget.value)}
      >
        {props.state.fontFamily === MIXED_FONT_VALUE && (
          <option value={MIXED_FONT_VALUE} disabled>Mixed fonts</option>
        )}
        {FONT_FAMILIES.map((family) => (
          <option key={family.id} value={family.id}>{family.label}</option>
        ))}
        <option value={RESET_FONT_VALUE}>Use document font</option>
      </select>
      <select
        className="font-tools__size"
        aria-label="Font size in points"
        value={props.state.fontSize}
        disabled={!props.editor}
        onChange={(event) => setFontSize(props.editor, event.currentTarget.value)}
      >
        {props.state.fontSize === MIXED_FONT_VALUE && (
          <option value={MIXED_FONT_VALUE} disabled>Mixed sizes</option>
        )}
        {fontSizeOptions().map((points) => (
          <option key={points} value={points}>{points} pt</option>
        ))}
        <option value={RESET_FONT_VALUE}>Use document size</option>
      </select>
    </div>
  );
}

function setFontFamily(editor: Editor | null, value: string) {
  if (!editor) {
    return;
  }
  const chain = editor.chain().focus(undefined, { scrollIntoView: false });
  if (value === RESET_FONT_VALUE) {
    chain.unsetMark("fontFamily").run();
  } else if (isFontFamilyId(value)) {
    chain.setMark("fontFamily", { family: value }).run();
  }
  restoreEditorFocus(editor);
}

function setFontSize(editor: Editor | null, value: string) {
  if (!editor) {
    return;
  }
  const points = Number(value);
  const chain = editor.chain().focus(undefined, { scrollIntoView: false });
  if (value === RESET_FONT_VALUE) {
    chain.unsetMark("fontSize").run();
  } else if (isFontSizePoints(points)) {
    chain.setMark("fontSize", { points }).run();
  }
  restoreEditorFocus(editor);
}

function restoreEditorFocus(editor: Editor) {
  queueMicrotask(() => editor.commands.focus(undefined, { scrollIntoView: false }));
}

function fontSizeOptions() {
  return Array.from(
    { length: MAX_FONT_SIZE_POINTS - MIN_FONT_SIZE_POINTS + 1 },
    (_, index) => MIN_FONT_SIZE_POINTS + index,
  );
}

function ReviewTools(props: { isOpen: boolean; onToggle: () => void }) {
  return (
    <div className="toolbar-group" role="group" aria-label="Review">
      <ToolbarButton
        controls="formatting-review-panel"
        expanded={props.isOpen}
        icon={ListChecks}
        label="Formatting review"
        onPress={props.onToggle}
      />
    </div>
  );
}

function HistoryTools(props: { editor: Editor | null; state: ToolbarState }) {
  return (
    <div className="toolbar-group" role="group" aria-label="History">
      <ToolbarButton
        disabled={!props.state.canUndo}
        icon={Undo2}
        label="Undo"
        onPress={() => props.editor?.chain().focus().undo().run()}
      />
      <ToolbarButton
        disabled={!props.state.canRedo}
        icon={Redo2}
        label="Redo"
        onPress={() => props.editor?.chain().focus().redo().run()}
      />
    </div>
  );
}

function InlineTools(props: { editor: Editor | null; state: ToolbarState }) {
  return (
    <div className="toolbar-group" role="group" aria-label="Inline formatting">
      <ToolbarButton
        active={props.state.bold}
        icon={Bold}
        label="Bold"
        onPress={() => props.editor?.chain().focus().toggleBold().run()}
      />
      <ToolbarButton
        active={props.state.italic}
        icon={Italic}
        label="Italic"
        onPress={() => props.editor?.chain().focus().toggleItalic().run()}
      />
      <ToolbarButton
        active={props.state.strike}
        icon={Strikethrough}
        label="Strikethrough"
        onPress={() => props.editor?.chain().focus().toggleStrike().run()}
      />
    </div>
  );
}

function StructureTools(props: { editor: Editor | null; state: ToolbarState }) {
  return (
    <div className="toolbar-group" role="group" aria-label="Structure">
      <ToolbarButton
        active={props.state.headingOne}
        icon={Heading1}
        label="Heading 1"
        onPress={() => props.editor?.chain().focus().toggleHeading({ level: 1 }).run()}
      />
      <ToolbarButton
        active={props.state.headingTwo}
        icon={Heading2}
        label="Heading 2"
        onPress={() => props.editor?.chain().focus().toggleHeading({ level: 2 }).run()}
      />
      <ToolbarButton
        active={props.state.bulletList}
        icon={List}
        label="Bulleted list"
        onPress={() => props.editor?.chain().focus().toggleBulletList().run()}
      />
      <ToolbarButton
        active={props.state.orderedList}
        icon={ListOrdered}
        label="Numbered list"
        onPress={() => props.editor?.chain().focus().toggleOrderedList().run()}
      />
      <ToolbarButton
        active={props.state.blockquote}
        icon={Quote}
        label="Block quote"
        onPress={() => props.editor?.chain().focus().toggleBlockquote().run()}
      />
    </div>
  );
}

function ToolbarButton(props: ToolbarButtonProps) {
  const Icon = props.icon;

  return (
    <button
      className="icon-button icon-button--toolbar"
      type="button"
      aria-controls={props.controls}
      aria-expanded={props.expanded}
      aria-label={props.label}
      aria-pressed={props.active}
      data-toolbar-button=""
      disabled={props.disabled}
      title={props.label}
      onClick={props.onPress}
      onFocus={claimToolbarTabStop}
    >
      <Icon aria-hidden="true" size={17} strokeWidth={1.9} />
    </button>
  );
}

function navigateToolbar(event: KeyboardEvent<HTMLDivElement>) {
  if (
    !isToolbarNavigationKey(event.key) ||
    !(event.target instanceof HTMLButtonElement)
  ) {
    return;
  }

  const buttons = enabledToolbarButtons(event.currentTarget);
  const currentIndex = buttons.indexOf(event.target);
  if (currentIndex < 0) {
    return;
  }

  event.preventDefault();
  focusToolbarButton(buttons, nextToolbarIndex(event.key, currentIndex, buttons.length));
}

function claimToolbarTabStop(event: FocusEvent<HTMLButtonElement>) {
  const toolbar = event.currentTarget.closest<HTMLDivElement>('[role="toolbar"]');
  if (toolbar) {
    setToolbarTabStop(toolbarButtons(toolbar), event.currentTarget);
  }
}

function normalizeToolbarTabStop(toolbar: HTMLDivElement | null) {
  if (!toolbar) {
    return;
  }

  const buttons = toolbarButtons(toolbar);
  const current = buttons.find((button) => button.tabIndex === 0 && !button.disabled);
  const fallback = buttons.find((button) => !button.disabled);
  setToolbarTabStop(buttons, current ?? fallback);
}

function focusToolbarButton(buttons: HTMLButtonElement[], targetIndex: number) {
  const target = buttons[targetIndex];
  if (target) {
    setToolbarTabStop(buttons, target);
    target.focus();
  }
}

function setToolbarTabStop(
  buttons: HTMLButtonElement[],
  target: HTMLButtonElement | undefined,
) {
  buttons.forEach((button) => {
    button.tabIndex = button === target ? 0 : -1;
  });
}

function toolbarButtons(toolbar: HTMLDivElement) {
  return Array.from(toolbar.querySelectorAll<HTMLButtonElement>("[data-toolbar-button]"));
}

function enabledToolbarButtons(toolbar: HTMLDivElement) {
  return toolbarButtons(toolbar).filter((button) => !button.disabled);
}

function nextToolbarIndex(key: ToolbarNavigationKey, current: number, count: number) {
  if (key === "Home" || key === "End") {
    return key === "Home" ? 0 : count - 1;
  }

  const step = key === "ArrowRight" ? 1 : -1;
  return (current + step + count) % count;
}

function isToolbarNavigationKey(key: string): key is ToolbarNavigationKey {
  return key === "ArrowLeft" || key === "ArrowRight" || key === "Home" || key === "End";
}

function useToolbarState(editor: Editor | null) {
  return (
    useEditorState({
      editor,
      selector: ({ editor: currentEditor }) => getToolbarState(currentEditor),
    }) ?? EMPTY_TOOLBAR_STATE
  );
}

function getToolbarState(editor: Editor | null) {
  if (!editor) {
    return EMPTY_TOOLBAR_STATE;
  }

  return {
    blockquote: editor.isActive("blockquote"),
    bold: editor.isActive("bold"),
    bulletList: editor.isActive("bulletList"),
    canRedo: editor.can().chain().focus().redo().run(),
    canUndo: editor.can().chain().focus().undo().run(),
    fontFamily: selectedFontFamily(editor),
    fontSize: selectedFontSize(editor),
    headingOne: editor.isActive("heading", { level: 1 }),
    headingTwo: editor.isActive("heading", { level: 2 }),
    italic: editor.isActive("italic"),
    orderedList: editor.isActive("orderedList"),
    strike: editor.isActive("strike"),
  };
}

function selectedFontFamily(editor: Editor) {
  return effectiveFontControlState(editor).fontFamily;
}

function selectedFontSize(editor: Editor) {
  return effectiveFontControlState(editor).fontSize;
}
