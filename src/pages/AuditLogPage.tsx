import { useQuery } from "@tanstack/react-query";
import { getAuditLogs } from "@/api";
import { ScrollText, ShieldCheck, ShieldAlert, Key, FolderOpen, ArrowDownUp } from "lucide-react";

const actionIcons: Record<string, typeof ScrollText> = {
  LOGIN: ShieldCheck, LOGOUT: ShieldAlert,
  KEY_CREATED: Key, KEY_UPDATED: Key, KEY_DELETED: Key, KEY_VIEWED: Key, KEY_COPIED: Key, KEY_ROTATED: Key,
  GROUP_CREATED: FolderOpen, GROUP_UPDATED: FolderOpen, GROUP_DELETED: FolderOpen,
  KEY_IMPORTED: ArrowDownUp, KEY_EXPORTED: ArrowDownUp,
};

const actionColors: Record<string, string> = {
  KEY_CREATED: "var(--status-success)", GROUP_CREATED: "var(--status-success)",
  KEY_DELETED: "var(--status-error)", GROUP_DELETED: "var(--status-error)",
  VAULT_LOCKED: "var(--status-warning)",
};

export function AuditLogPage() {
  const { data: logs = [], isLoading } = useQuery({ queryKey: ["audit-logs"], queryFn: () => getAuditLogs(100) });

  return (
    <div className="space-y-5 animate-fade-in">
      <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>审计日志</h2>

      {isLoading ? (
        <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>加载中...</div>
      ) : logs.length === 0 ? (
        <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>暂无审计日志</div>
      ) : (
        <div className="rounded-xl overflow-hidden" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
          <div className="divide-y" style={{ borderColor: "var(--border-default)" }}>
            {logs.map((log, i) => {
              const Icon = actionIcons[log.action] || ScrollText;
              const color = actionColors[log.action] || "var(--text-muted)";
              return (
                <div
                  key={log.id}
                  className="flex items-center gap-4 px-4 py-3 transition-smooth animate-fade-in"
                  style={{ animationDelay: `${i * 20}ms` }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
                  onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                >
                  <div className="w-8 h-8 rounded-lg flex items-center justify-center" style={{ background: `${color}15` }}>
                    <Icon size={14} style={{ color }} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <span className="text-sm font-medium" style={{ color }}>{log.action}</span>
                    <span className="text-xs ml-2" style={{ color: "var(--text-muted)" }}>{log.resource_type}</span>
                  </div>
                  <span className="text-xs shrink-0" style={{ color: "var(--text-muted)" }}>
                    {new Date(log.timestamp).toLocaleString("zh-CN")}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
