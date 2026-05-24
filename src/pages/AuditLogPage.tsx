import { useQuery } from "@tanstack/react-query";
import { getAuditLogs } from "@/api";

export function AuditLogPage() {
  const { data: logs = [], isLoading } = useQuery({
    queryKey: ["audit-logs"],
    queryFn: () => getAuditLogs(100),
  });

  const getActionColor = (action: string) => {
    if (action.includes("Added") || action.includes("Created") || action.includes("Initialized")) {
      return "text-vault-success";
    }
    if (action.includes("Deleted") || action.includes("Reset")) {
      return "text-vault-error";
    }
    if (action.includes("Locked")) {
      return "text-vault-warning";
    }
    return "text-vault-muted";
  };

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-vault-text">审计日志</h2>

      {isLoading ? (
        <div className="text-center py-8 text-vault-muted">加载中...</div>
      ) : logs.length === 0 ? (
        <div className="text-center py-8 text-vault-muted">暂无审计日志</div>
      ) : (
        <div className="bg-vault-surface rounded-lg border border-vault-border overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-vault-border">
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  时间
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  操作
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  资源类型
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  资源 ID
                </th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => (
                <tr
                  key={log.id}
                  className="border-b border-vault-border last:border-0 hover:bg-vault-bg/50"
                >
                  <td className="px-4 py-3 text-vault-muted text-sm">
                    {new Date(log.timestamp).toLocaleString("zh-CN")}
                  </td>
                  <td className={`px-4 py-3 font-medium ${getActionColor(log.action)}`}>
                    {log.action}
                  </td>
                  <td className="px-4 py-3 text-vault-muted">
                    {log.resource_type}
                  </td>
                  <td className="px-4 py-3 text-vault-muted text-sm font-mono">
                    {log.resource_id}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
