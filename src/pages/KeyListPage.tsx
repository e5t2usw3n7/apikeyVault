import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listKeys, deleteKey, testConnectivity } from "@/api";
import { useUIStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import type { KeyEntry, Environment } from "@/types";

export function KeyListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);

  const [search, setSearch] = useState("");
  const [envFilter, setEnvFilter] = useState<Environment | "">("");
  const [deleteTarget, setDeleteTarget] = useState<KeyEntry | null>(null);

  const { data: keys = [], isLoading } = useQuery({
    queryKey: ["keys"],
    queryFn: listKeys,
  });

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
    onSuccess: (result) => {
      addNotification(
        result.success ? "success" : "error",
        result.message
      );
    },
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

  const environments: Environment[] = [
    "Production",
    "Staging",
    "Development",
    "Testing",
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-vault-text">密钥管理</h2>
        <button
          onClick={() => navigate("/keys/new")}
          className="px-4 py-2 bg-vault-primary hover:bg-vault-primary/80 text-white rounded-lg"
        >
          添加密钥
        </button>
      </div>

      {/* 搜索和筛选 */}
      <div className="flex gap-4">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="搜索密钥名称或提供商..."
          className="flex-1 px-4 py-2 rounded-lg bg-vault-surface border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
        />
        <select
          value={envFilter}
          onChange={(e) => setEnvFilter(e.target.value as Environment | "")}
          className="px-4 py-2 rounded-lg bg-vault-surface border border-vault-border text-vault-text"
        >
          <option value="">所有环境</option>
          {environments.map((env) => (
            <option key={env} value={env}>
              {env}
            </option>
          ))}
        </select>
      </div>

      {/* 密钥列表 */}
      {isLoading ? (
        <div className="text-center py-8 text-vault-muted">加载中...</div>
      ) : filteredKeys.length === 0 ? (
        <div className="text-center py-8 text-vault-muted">暂无密钥</div>
      ) : (
        <div className="bg-vault-surface rounded-lg border border-vault-border overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-vault-border">
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  名称
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  提供商
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  类型
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  环境
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  更新时间
                </th>
                <th className="px-4 py-3 text-right text-vault-muted font-medium">
                  操作
                </th>
              </tr>
            </thead>
            <tbody>
              {filteredKeys.map((key) => (
                <tr
                  key={`${key.name}-${key.environment}`}
                  className="border-b border-vault-border last:border-0 hover:bg-vault-bg/50"
                >
                  <td className="px-4 py-3">
                    <button
                      onClick={() =>
                        navigate(`/keys/${key.name}/${key.environment}`)
                      }
                      className="text-vault-primary hover:underline"
                    >
                      {key.name}
                    </button>
                  </td>
                  <td className="px-4 py-3 text-vault-muted">{key.provider}</td>
                  <td className="px-4 py-3 text-vault-muted">{key.key_type}</td>
                  <td className="px-4 py-3">
                    <span className="px-2 py-1 rounded text-xs bg-vault-primary/20 text-vault-primary">
                      {key.environment}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-vault-muted text-sm">
                    {new Date(key.updated_at).toLocaleDateString("zh-CN")}
                  </td>
                  <td className="px-4 py-3 text-right space-x-2">
                    <button
                      onClick={() => testMutation.mutate(key)}
                      className="text-vault-muted hover:text-vault-success text-sm"
                      title="测试连通性"
                    >
                      🔗
                    </button>
                    <button
                      onClick={() =>
                        navigate(`/keys/${key.name}/${key.environment}/edit`)
                      }
                      className="text-vault-muted hover:text-vault-primary text-sm"
                      title="编辑"
                    >
                      ✏️
                    </button>
                    <button
                      onClick={() => setDeleteTarget(key)}
                      className="text-vault-muted hover:text-vault-error text-sm"
                      title="删除"
                    >
                      🗑️
                    </button>
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
