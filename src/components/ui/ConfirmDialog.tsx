import { AlertTriangle, X } from "lucide-react";

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "warning" | "default";
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmLabel = "确认",
  cancelLabel = "取消",
  variant = "default",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  if (!isOpen) return null;

  const confirmBg =
    variant === "danger"
      ? "var(--status-error)"
      : variant === "warning"
      ? "var(--status-warning)"
      : "var(--brand-primary)";

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center animate-fade-in"
      style={{ background: "rgba(0,0,0,0.6)", backdropFilter: "blur(4px)" }}
      onClick={onCancel}
    >
      <div
        className="glass-card p-6 max-w-md w-full mx-4 animate-scale-in"
        style={{ background: "var(--bg-elevated)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start gap-4">
          {variant !== "default" && (
            <div
              className="w-10 h-10 rounded-full flex items-center justify-center shrink-0"
              style={{ background: variant === "danger" ? "var(--status-error-bg)" : "var(--status-warning-bg)" }}
            >
              <AlertTriangle size={20} style={{ color: confirmBg }} />
            </div>
          )}
          <div className="flex-1">
            <h3 className="text-base font-semibold" style={{ color: "var(--text-primary)" }}>
              {title}
            </h3>
            <p className="mt-2 text-sm" style={{ color: "var(--text-secondary)" }}>
              {message}
            </p>
          </div>
          <button
            onClick={onCancel}
            className="p-1 rounded-md transition-smooth"
            style={{ color: "var(--text-muted)" }}
          >
            <X size={18} />
          </button>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
            style={{
              background: "var(--bg-surface)",
              color: "var(--text-secondary)",
              border: "1px solid var(--border-default)",
            }}
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
            style={{ background: confirmBg, color: "white" }}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
