import { Activity } from "lucide-react";

interface WorkspaceOperationNoticeProps {
  message: string;
  pending: boolean;
}

export function WorkspaceOperationNotice(props: WorkspaceOperationNoticeProps) {
  if (!props.message) {
    return <div className="workspace-operation-notice workspace-operation-notice--empty" aria-hidden="true" />;
  }

  return (
    <div
      className="workspace-operation-notice"
      data-operation-state={props.pending ? "pending" : "settled"}
      role="status"
      aria-live="polite"
      aria-atomic="true"
    >
      <Activity aria-hidden="true" size={14} strokeWidth={1.9} />
      <span>{props.message}</span>
    </div>
  );
}
