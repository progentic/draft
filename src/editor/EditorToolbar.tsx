import type { Editor } from "@tiptap/react";
import { useEditorState } from "@tiptap/react";
import type { LucideIcon } from "lucide-react";
import {
  Bold,
  Heading1,
  Heading2,
  Italic,
  List,
  ListOrdered,
  Quote,
  Redo2,
  Strikethrough,
  Undo2,
} from "lucide-react";

interface EditorToolbarProps {
  editor: Editor | null;
}

interface ToolbarButtonProps {
  active?: boolean;
  disabled?: boolean;
  icon: LucideIcon;
  label: string;
  onPress: () => void;
}

type ToolbarState = typeof EMPTY_TOOLBAR_STATE;

const EMPTY_TOOLBAR_STATE = {
  blockquote: false,
  bold: false,
  bulletList: false,
  canRedo: false,
  canUndo: false,
  headingOne: false,
  headingTwo: false,
  italic: false,
  orderedList: false,
  strike: false,
};

export function EditorToolbar(props: EditorToolbarProps) {
  const state = useToolbarState(props.editor);

  return (
    <div className="editor-toolbar" role="toolbar" aria-label="Text formatting">
      <HistoryTools editor={props.editor} state={state} />
      <span className="toolbar-separator" aria-hidden="true" />
      <InlineTools editor={props.editor} state={state} />
      <span className="toolbar-separator" aria-hidden="true" />
      <StructureTools editor={props.editor} state={state} />
    </div>
  );
}

function HistoryTools(props: { editor: Editor | null; state: ToolbarState }) {
  return (
    <div className="toolbar-group" aria-label="History">
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
    <div className="toolbar-group" aria-label="Inline formatting">
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
    <div className="toolbar-group" aria-label="Structure">
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
      aria-label={props.label}
      aria-pressed={props.active ?? false}
      disabled={props.disabled}
      title={props.label}
      onClick={props.onPress}
    >
      <Icon aria-hidden="true" size={17} strokeWidth={1.9} />
    </button>
  );
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
    headingOne: editor.isActive("heading", { level: 1 }),
    headingTwo: editor.isActive("heading", { level: 2 }),
    italic: editor.isActive("italic"),
    orderedList: editor.isActive("orderedList"),
    strike: editor.isActive("strike"),
  };
}
