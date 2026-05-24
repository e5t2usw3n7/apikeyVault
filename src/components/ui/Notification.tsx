import { useEffect } from "react";
import { useUIStore } from "@/store";

export function Notification() {
  const { notifications, removeNotification } = useUIStore();

  useEffect(() => {
    const timer = setInterval(() => {
      const now = Date.now();
      notifications.forEach((n) => {
        if (now - n.createdAt > 3000) {
          removeNotification(n.id);
        }
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [notifications, removeNotification]);

  if (notifications.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2">
      {notifications.map((n) => (
        <div
          key={n.id}
          className={`px-4 py-3 rounded-lg shadow-lg text-white flex items-center justify-between min-w-[300px] ${
            n.type === "success" ? "bg-vault-success" : "bg-vault-error"
          }`}
        >
          <span>{n.message}</span>
          <button
            onClick={() => removeNotification(n.id)}
            className="ml-4 hover:opacity-80"
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
