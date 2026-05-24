import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listGroups, createGroup, updateGroup } from "@/api";
import { useUIStore } from "@/store";

export function GroupEditPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const isEditMode = !!id && id !== "new";

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  const { data: groups = [] } = useQuery({
    queryKey: ["groups"],
    queryFn: listGroups,
  });

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
      if (isEditMode) {
        return updateGroup(id!, name, description || undefined);
      } else {
        return createGroup(name);
      }
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
    if (!name.trim()) {
      addNotification("error", "请输入分组名称");
      return;
    }
    mutation.mutate();
  };

  return (
    <div className="max-w-lg mx-auto space-y-6">
      <div className="flex items-center space-x-4">
        <button
          onClick={() => navigate("/groups")}
          className="text-vault-muted hover:text-vault-text"
        >
          ← 返回
        </button>
        <h2 className="text-2xl font-bold text-vault-text">
          {isEditMode ? "编辑分组" : "创建分组"}
        </h2>
      </div>

      <form
        onSubmit={handleSubmit}
        className="bg-vault-surface rounded-lg p-6 border border-vault-border space-y-6"
      >
        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            名称 *
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
            placeholder="分组名称"
            autoFocus
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            描述
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
            placeholder="可选描述"
          />
        </div>

        <div className="flex justify-end space-x-3">
          <button
            type="button"
            onClick={() => navigate("/groups")}
            className="px-6 py-2 rounded-lg bg-vault-border hover:bg-vault-border/80 text-vault-text"
          >
            取消
          </button>
          <button
            type="submit"
            disabled={mutation.isPending}
            className="px-6 py-2 rounded-lg bg-vault-primary hover:bg-vault-primary/80 text-white disabled:opacity-50"
          >
            {mutation.isPending
              ? "保存中..."
              : isEditMode
              ? "更新"
              : "创建"}
          </button>
        </div>
      </form>
    </div>
  );
}
