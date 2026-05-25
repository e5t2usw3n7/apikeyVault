import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listKeys, deleteKey, testConnectivity } from "@/api";
import { useUIStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import type { KeyEntry, Environment } from "@/types";
import { Search, Plus, Link2, Pencil, Trash2, Filter } from "lucide-react";

export function KeyListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const [search, setSearch] = useState("");
  const [envFilter, setEnvFilter] = useState<Environment | "">("");
  const [deleteTarget, setDeleteTarget] = useState<KeyEntry | null>(null);

  const { data: keys = [], isLoading } = useQuery({ queryKey: ["keys"], queryFn: listKeys });

  const deleteMutation = useMutation({
    mutationFn: (key: KeyEntry) => deleteKey(key.name, key.environment),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["keys"] });
      addNotification("success", "密钥已删除");
      setDeleteTarget(null);
    },
    onError: (err) => addNotification("error", `删除失败: ${err}`),
  });

  const testMutation = useMutation({
    mutationFn: (key: KeyEntry) => testConnectivity(key.name, key.environment),
    onSuccess: (result) => addNotification(result.success ? "success" : "error", result.message),
    onError: (err) => addNotification("error", `测试失败: ${err}`),
  });

  const filteredKeys = keys.filter((key) => {
    const matchesSearch =
      !search ||
      key.name.toLowerCase().includes(search.toLowerCase()) ||
      key.provider.toLowerCase().includes(search.toLowerCase());
    const matchesEnv = !envFilter || key.environment === envFilter;
    return matchesSearch && matchesEnv;
  });

  const environments: Environment[] = ["Production", "Staging", "Development", "Testing"];

  return (
    <div className="space-y-5 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>
          密钥管理
        </h2>
        <button
          onClick={() => navigate("/keys/new")}
          className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
          style={{ background: "var(--brand-primary)", color: "white" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--brand-primary-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "var(--brand-primary)")}
        >
          <Plus size={16} />
          添加密钥
        </button>
      </div>

      {/* Search and filter */}
      <div className="flex gap-3">
        <div className="flex-1 relative">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: "var(--text-muted)" }} />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="搜索密钥名称或提供商..."
            className="form-input w-full pl-10 pr-4 py-2.5 rounded-xl text-sm"
          />
        </div>
        <div className="relative">
          <Filter size={16} className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: "var(--text-muted)" }} />
          <select
            value={envFilter}
            onChange={(e) => setEnvFilter(e.target.value as Environment | "")}
            className="form-input pl-10 pr-4 py-2.5 rounded-xl text-sm appearance-none"
            style={{ minWidth: 150 }}
          >
            <option value="">所有环境</option>
            {environments.map((env) => (
              <option key={env} value={env}>{env}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Table */}
      {isLoading ? (
        <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>加载中...</div>
      ) : filteredKeys.length === 0 ? (
        <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>暂无密钥</div>
      ) : (
        <div
          className="rounded-xl overflow-hidden"
          style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}
        >
          <table className="w-full">
            <thead>
              <tr style={{ borderBottom: "1px solid var(--border-default)" }}>
                {["名称", "提供商", "类型", "环境", "更新时间", "操作"].map((h) => (
                  <th
                    key={h}
                    className="px-4 py-3 text-left text-xs font-medium"
                    style={{ color: "var(--text-muted)" }}
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filteredKeys.map((key, i) => (
                <tr
                  key={`${key.name}-${key.environment}`}
                  className="transition-smooth animate-fade-in"
                  style={{
                    borderBottom: i < filteredKeys.length - 1 ? "1px solid var(--border-default)" : "none",
                    animationDelay: `${i * 30}ms`,
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
                  onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                >
                  <td className="px-4 py-3">
                    <button
                      onClick={() => navigate(`/keys/${key.name}/${key.environment}`)}
                      className="text-sm font-medium transition-smooth"
                      style={{ color: "var(--brand-primary)" }}
                    >
                      {key.name}
                    </button>
                  </td>
                  <td className="px-4 py-3 text-sm" style={{ color: "var(--text-secondary)" }}>
                    {key.provider}
                  </td>
                  <td className="px-4 py-3 text-sm" style={{ color: "var(--text-secondary)" }}>
                    {key.key_type}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className="px-2 py-1 rounded-md text-xs font-medium"
                      style={{
                        background: "rgba(124,58,237,0.1)",
                        color: "var(--brand-primary)",
                      }}
                    >
                      {key.environment}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--text-muted)" }}>
                    {new Date(key.updated_at).toLocaleDateString("zh-CN")}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => testMutation.mutate(key)}
                        className="p-1.5 rounded-md transition-smooth"
                        style={{ color: "var(--text-muted)" }}
                        title="测试连通性"
                        onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
                        onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                      >
                        <Link2 size={14} />
                      </button>
                      <button
                        onClick={() => navigate(`/keys/${key.name}/${key.environment}/edit`)}
                        className="p-1.5 rounded-md transition-smooth"
                        style={{ color: "var(--text-muted)" }}
                        title="编辑"
                        onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
                        onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                      >
                        <Pencil size={14} />
                      </button>
                      <button
                        onClick={() => setDeleteTarget(key)}
                        className="p-1.5 rounded-md transition-smooth"
                        style={{ color: "var(--status-error)" }}
                        title="删除"
                        onMouseEnter={(e) => (e.currentTarget.style.background = "var(--status-error-bg)")}
                        onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <ConfirmDialog
        isOpen={!!deleteTarget}
        title="删除密钥"
        message={`确定要删除密钥 "${deleteTarget?.name}" 吗？此操作不可恢复。`}
        confirmLabel="删除"
        variant="danger"
        onConfirm={() => deleteTarget && deleteMutation.mutate(deleteTarget)}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  );
}
