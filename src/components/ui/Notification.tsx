import { useEffect } from "react";
import { useUIStore } from "@/store";
import { CheckCircle2, XCircle, X } from "lucide-react";

export function Notification() {
  const { notifications, removeNotification } = useUIStore();

  useEffect(() => {
    const timer = setInterval(() => {
      const now = Date.now();
      notifications.forEach((n) => {
        if (now - n.createdAt > 4000) {
          removeNotification(n.id);
        }
      });
    }, 1000);
    return () => clearInterval(timer);
  }, [notifications, removeNotification]);

  if (notifications.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2" style={{ minWidth: 320 }}>
      {notifications.map((n) => (
        <div
          key={n.id}
          className="animate-slide-up flex items-center gap-3 px-4 py-3 rounded-xl"
          style={{
            background: n.type === "success" ? "var(--status-success-bg)" : "var(--status-error-bg)",
            border: `1px solid ${n.type === "success" ? "rgba(16,185,129,0.2)" : "rgba(239,68,68,0.2)"}`,
            backdropFilter: "blur(12px)",
          }}
        >
          {n.type === "success" ? (
            <CheckCircle2 size={18} style={{ color: "var(--status-success)", flexShrink: 0 }} />
          ) : (
            <XCircle size={18} style={{ color: "var(--status-error)", flexShrink: 0 }} />
          )}
          <span className="text-sm flex-1" style={{ color: "var(--text-primary)" }}>
            {n.message}
          </span>
          <button
            onClick={() => removeNotification(n.id)}
            className="p-1 rounded-md transition-smooth"
            style={{ color: "var(--text-muted)" }}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}
