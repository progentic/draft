import type { LucideIcon } from "lucide-react";
import type { Dispatch, KeyboardEvent, RefObject, SetStateAction } from "react";
import { useEffect, useRef, useState } from "react";
import {
  BookOpen,
  Download,
  FilePlus2,
  FolderOpen,
  MoreHorizontal,
  Save,
  SaveAll,
  ScanText,
  X,
} from "lucide-react";

import type {
  WorkspaceAction,
  WorkspaceActions,
} from "../features/workspace-actions/useWorkspaceActions";

interface WorkspaceCommandBarProps {
  activePanel: "references" | "text-review" | null;
  actions: WorkspaceActions;
  exportLabel: string;
}

export function WorkspaceCommandBar(props: WorkspaceCommandBarProps) {
  return (
    <nav className="workspace-command-bar" aria-label="Document actions">
      <PrimaryDocumentActions actions={props.actions} />
      <OverflowActions {...props} />
    </nav>
  );
}

function PrimaryDocumentActions(props: { actions: WorkspaceActions }) {
  return (
    <div
      className="workspace-command-group"
      role="group"
      aria-label="Primary document actions"
    >
      <CommandButton
        accessibleLabel="New Document"
        action="new_document"
        actions={props.actions}
        icon={FilePlus2}
        label="New"
        showLabel
      />
      <CommandButton
        action="open_document"
        actions={props.actions}
        icon={FolderOpen}
        label="Open…"
      />
      <CommandButton action="save_document" actions={props.actions} icon={Save} label="Save" />
      <CommandButton action="close_document" actions={props.actions} icon={X} label="Close" />
    </div>
  );
}

function OverflowActions(props: WorkspaceCommandBarProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  useOverflowDismissal(open, rootRef, setOpen);
  useOverflowInitialFocus(open, rootRef);

  const close = (restoreFocus: boolean) => {
    setOpen(false);
    if (restoreFocus) {
      triggerRef.current?.focus();
    }
  };
  return (
    <div className="workspace-overflow" ref={rootRef}>
      <button
        ref={triggerRef}
        className="workspace-command workspace-command--icon"
        type="button"
        aria-controls="workspace-overflow-menu"
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label="More document actions"
        title="More document actions"
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => openOverflowFromKeyboard(event, setOpen)}
      >
        <MoreHorizontal aria-hidden="true" size={17} strokeWidth={1.9} />
      </button>
      <OverflowMenu {...props} open={open} onClose={close} />
    </div>
  );
}

function OverflowMenu(
  props: WorkspaceCommandBarProps & {
    open: boolean;
    onClose: (restoreFocus: boolean) => void;
  },
) {
  return (
    <div
      id="workspace-overflow-menu"
      className="workspace-overflow__menu"
      role="menu"
      aria-label="More document actions"
      hidden={!props.open}
      onKeyDown={(event) => navigateOverflowMenu(event, props.onClose)}
    >
      <OverflowItem
        action="save_document_as"
        actions={props.actions}
        icon={SaveAll}
        label="Save As…"
        onClose={props.onClose}
      />
      <OverflowItem
        action="export_docx"
        actions={props.actions}
        icon={Download}
        label={props.exportLabel}
        onClose={props.onClose}
      />
      <span className="workspace-overflow__separator" role="separator" />
      <OverflowItem
        action="open_references"
        actions={props.actions}
        checked={props.activePanel === "references"}
        icon={BookOpen}
        label="References"
        onClose={props.onClose}
      />
      <OverflowItem
        action="run_text_checks"
        actions={props.actions}
        checked={props.activePanel === "text-review"}
        icon={ScanText}
        label="Text checks"
        onClose={props.onClose}
      />
    </div>
  );
}

function CommandButton(props: {
  accessibleLabel?: string;
  action: WorkspaceAction;
  actions: WorkspaceActions;
  icon: LucideIcon;
  label: string;
  showLabel?: boolean;
}) {
  const Icon = props.icon;
  const accessibleLabel = props.accessibleLabel ?? props.label;
  return (
    <button
      className={
        props.showLabel
          ? "workspace-command"
          : "workspace-command workspace-command--icon"
      }
      type="button"
      aria-label={accessibleLabel}
      disabled={!props.actions.enabled[props.action]}
      title={accessibleLabel}
      onClick={() => props.actions.dispatch(props.action)}
    >
      <Icon aria-hidden="true" size={16} strokeWidth={1.9} />
      {props.showLabel ? <span>{props.label}</span> : null}
    </button>
  );
}

function OverflowItem(props: {
  action: WorkspaceAction;
  actions: WorkspaceActions;
  checked?: boolean;
  icon: LucideIcon;
  label: string;
  onClose: (restoreFocus: boolean) => void;
}) {
  const Icon = props.icon;
  const role = props.checked === undefined ? "menuitem" : "menuitemcheckbox";
  return (
    <button
      className="workspace-overflow__item"
      type="button"
      role={role}
      aria-checked={props.checked}
      tabIndex={-1}
      disabled={!props.actions.enabled[props.action]}
      onClick={() => runOverflowAction(props)}
    >
      <Icon aria-hidden="true" size={16} strokeWidth={1.9} />
      <span>{props.label}</span>
    </button>
  );
}

function runOverflowAction(props: {
  action: WorkspaceAction;
  actions: WorkspaceActions;
  onClose: (restoreFocus: boolean) => void;
}) {
  props.actions.dispatch(props.action);
  props.onClose(true);
}

function useOverflowDismissal(
  open: boolean,
  rootRef: RefObject<HTMLDivElement | null>,
  setOpen: Dispatch<SetStateAction<boolean>>,
) {
  useEffect(() => {
    if (!open) return;
    const dismiss = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", dismiss);
    return () => document.removeEventListener("pointerdown", dismiss);
  }, [open, rootRef, setOpen]);
}

function useOverflowInitialFocus(open: boolean, rootRef: RefObject<HTMLDivElement | null>) {
  useEffect(() => {
    if (open) queueMicrotask(() => enabledOverflowItems(rootRef.current)[0]?.focus());
  }, [open, rootRef]);
}

function openOverflowFromKeyboard(
  event: KeyboardEvent<HTMLButtonElement>,
  setOpen: (open: boolean) => void,
) {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    setOpen(true);
  }
}

function navigateOverflowMenu(
  event: KeyboardEvent<HTMLDivElement>,
  close: (restoreFocus: boolean) => void,
) {
  if (event.key === "Escape") {
    event.preventDefault();
    close(true);
    return;
  }
  const items = enabledOverflowItems(event.currentTarget);
  const index = items.indexOf(document.activeElement as HTMLButtonElement);
  focusOverflowItem(event, items, index);
}

function focusOverflowItem(
  event: KeyboardEvent<HTMLDivElement>,
  items: HTMLButtonElement[],
  index: number,
) {
  const target = overflowTargetIndex(event.key, index, items.length);
  if (target === undefined) return;
  event.preventDefault();
  items[target]?.focus();
}

function overflowTargetIndex(key: string, index: number, length: number) {
  if (key === "Home") return 0;
  if (key === "End") return length - 1;
  if (key === "ArrowDown") return (index + 1) % length;
  if (key === "ArrowUp") return (index - 1 + length) % length;
  return undefined;
}

function enabledOverflowItems(root: HTMLElement | null) {
  return Array.from(
    root?.querySelectorAll<HTMLButtonElement>('[role^="menuitem"]:not(:disabled)') ?? [],
  );
}
