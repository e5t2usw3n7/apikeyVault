import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listGroups, deleteGroup } from "@/api";
import { useUIStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import type { Group } from "@/types";

export function GroupListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const [deleteTarget, setDeleteTarget] = useState<Group | null>(null);

  const { data: groups = [], isLoading } = useQuery({
    queryKey: ["groups"],
    queryFn: listGroups,
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteGroup(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["groups"] });
      addNotification("success", "分组已删除");
      setDeleteTarget(null);
    },
    onError: (err) => addNotification("error", `删除失败: ${err}`),
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-vault-text">分组管理</h2>
        <button
          onClick={() => navigate("/groups/new")}
          className="px-4 py-2 bg-vault-primary hover:bg-vault-primary/80 text-white rounded-lg"
        >
          创建分组
        </button>
      </div>

      {isLoading ? (
        <div className="text-center py-8 text-vault-muted">加载中...</div>
      ) : groups.length === 0 ? (
        <div className="text-center py-8 text-vault-muted">暂无分组</div>
      ) : (
        <div className="bg-vault-surface rounded-lg border border-vault-border overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-vault-border">
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  名称
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  描述
                </th>
                <th className="px-4 py-3 text-left text-vault-muted font-medium">
                  创建时间
                </th>
                <th className="px-4 py-3 text-right text-vault-muted font-medium">
                  操作
                </th>
              </tr>
            </thead>
            <tbody>
              {groups.map((group) => (
                <tr
                  key={group.id}
                  className="border-b border-vault-border last:border-0 hover:bg-vault-bg/50"
                >
                  <td className="px-4 py-3 text-vault-text font-medium">
                    {group.name}
                  </td>
                  <td className="px-4 py-3 text-vault-muted">
                    {group.description || "-"}
                  </td>
                  <td className="px-4 py-3 text-vault-muted text-sm">
                    {new Date(group.created_at).toLocaleDateString("zh-CN")}
                  </td>
                  <td className="px-4 py-3 text-right space-x-2">
                    <button
                      onClick={() => navigate(`/groups/${group.id}/edit`)}
                      className="text-vault-muted hover:text-vault-primary text-sm"
                    >
                      ✏️
                    </button>
                    <button
                      onClick={() => setDeleteTarget(group)}
                      className="text-vault-muted hover:text-vault-error text-sm"
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
        title="删除分组"
        message={`确定要删除分组 "${deleteTarget?.name}" 吗？分组内的密钥不会被删除。`}
        confirmLabel="删除"
        variant="danger"
        onConfirm={() =>
          deleteTarget && deleteMutation.mutate(deleteTarget.id)
        }
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  );
}
