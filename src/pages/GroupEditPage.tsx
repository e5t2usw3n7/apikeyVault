import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listGroups, createGroup, updateGroup } from "@/api";
import { useUIStore } from "@/store";
import { ArrowLeft, Save } from "lucide-react";

export function GroupEditPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const isEditMode = !!id && id !== "new";

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  const { data: groups = [] } = useQuery({ queryKey: ["groups"], queryFn: listGroups });

  useEffect(() => {
    if (isEditMode) {
      const group = groups.find((g) => g.id === id);
      if (group) {
        setName(group.name);
        setDescription(group.description || "");
      }
    }
  }, [isEditMode, id, groups]);

  const mutation = useMutation({
    mutationFn: async () => {
      if (isEditMode) return updateGroup(id!, name, description || undefined);
      return createGroup(name);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["groups"] });
      addNotification("success", isEditMode ? "分组已更新" : "分组已创建");
      navigate("/groups");
    },
    onError: (err) => addNotification("error", `操作失败: ${err}`),
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) { addNotification("error", "请输入分组名称"); return; }
    mutation.mutate();
  };

  const inputStyle = { background: "var(--bg-surface)", border: "1px solid var(--border-default)", color: "var(--text-primary)", outline: "none" };

  return (
    <div className="max-w-lg mx-auto space-y-5 animate-fade-in">
      <div className="flex items-center gap-3">
        <button onClick={() => navigate("/groups")} className="p-2 rounded-lg transition-smooth" style={{ color: "var(--text-muted)" }}>
          <ArrowLeft size={18} />
        </button>
        <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>{isEditMode ? "编辑分组" : "创建分组"}</h2>
      </div>

      <form onSubmit={handleSubmit} className="p-6 rounded-xl space-y-5" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
        <div>
          <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>名称 <span style={{ color: "var(--status-error)" }}>*</span></label>
          <input type="text" value={name} onChange={(e) => setName(e.target.value)} className="w-full px-3 py-2.5 rounded-lg text-sm transition-smooth" style={inputStyle} placeholder="分组名称" autoFocus />
        </div>
        <div>
          <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>描述</label>
          <textarea value={description} onChange={(e) => setDescription(e.target.value)} rows={3} className="w-full px-3 py-2.5 rounded-lg text-sm transition-smooth" style={inputStyle} placeholder="可选描述" />
        </div>
        <div className="flex justify-end gap-3">
          <button type="button" onClick={() => navigate("/groups")} className="px-5 py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--bg-surface-hover)", color: "var(--text-secondary)", border: "1px solid var(--border-default)" }}>取消</button>
          <button type="submit" disabled={mutation.isPending} className="flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--brand-primary)", color: "white" }}>
            <Save size={14} /> {mutation.isPending ? "保存中..." : isEditMode ? "更新" : "创建"}
          </button>
        </div>
      </form>
    </div>
  );
}
