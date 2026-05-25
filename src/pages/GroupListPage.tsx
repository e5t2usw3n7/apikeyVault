import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listGroups, deleteGroup } from "@/api";
import { useUIStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import type { Group } from "@/types";
import { Plus, Pencil, Trash2, FolderOpen } from "lucide-react";

export function GroupListPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const [deleteTarget, setDeleteTarget] = useState<Group | null>(null);

  const { data: groups = [], isLoading } = useQuery({ queryKey: ["groups"], queryFn: listGroups });

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
    <div className="space-y-5 animate-fade-in">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>分组管理</h2>
        <button
          onClick={() => navigate("/groups/new")}
          className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
          style={{ background: "var(--brand-primary)", color: "white" }}
        >
          <Plus size={16} /> 创建分组
        </button>
      </div>

      {isLoading ? (
        <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>加载中...</div>
      ) : groups.length === 0 ? (
        <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>暂无分组</div>
      ) : (
        <div className="rounded-xl overflow-hidden" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
          <table className="w-full">
            <thead>
              <tr style={{ borderBottom: "1px solid var(--border-default)" }}>
                {["名称", "描述", "创建时间", "操作"].map((h) => (
                  <th key={h} className="px-4 py-3 text-left text-xs font-medium" style={{ color: "var(--text-muted)" }}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {groups.map((group, i) => (
                <tr
                  key={group.id}
                  className="transition-smooth animate-fade-in"
                  style={{ borderBottom: i < groups.length - 1 ? "1px solid var(--border-default)" : "none", animationDelay: `${i * 30}ms` }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
                  onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                >
                  <td className="px-4 py-3 flex items-center gap-2">
                    <FolderOpen size={14} style={{ color: "var(--brand-primary)" }} />
                    <span className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>{group.name}</span>
                  </td>
                  <td className="px-4 py-3 text-sm" style={{ color: "var(--text-secondary)" }}>{group.description || "-"}</td>
                  <td className="px-4 py-3 text-xs" style={{ color: "var(--text-muted)" }}>{new Date(group.created_at).toLocaleDateString("zh-CN")}</td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-1">
                      <button onClick={() => navigate(`/groups/${group.id}/edit`)} className="p-1.5 rounded-md transition-smooth" style={{ color: "var(--text-muted)" }} onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")} onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
                        <Pencil size={14} />
                      </button>
                      <button onClick={() => setDeleteTarget(group)} className="p-1.5 rounded-md transition-smooth" style={{ color: "var(--status-error)" }} onMouseEnter={(e) => (e.currentTarget.style.background = "var(--status-error-bg)")} onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
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

      <ConfirmDialog isOpen={!!deleteTarget} title="删除分组" message={`确定要删除分组 "${deleteTarget?.name}" 吗？`} confirmLabel="删除" variant="danger" onConfirm={() => deleteTarget && deleteMutation.mutate(deleteTarget.id)} onCancel={() => setDeleteTarget(null)} />
    </div>
  );
}
