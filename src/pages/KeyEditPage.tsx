import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listKeys, addKey, updateKey, listGroups } from "@/api";
import { useUIStore } from "@/store";
import type { KeyType, Environment } from "@/types";

const KEY_TYPES: KeyType[] = [
  "ApiKey",
  "BearerToken",
  "BasicAuth",
  "OAuth2",
  "Jwt",
  "SshKey",
  "Other",
];

const ENVIRONMENTS: Environment[] = [
  "Production",
  "Staging",
  "Development",
  "Testing",
];

export function KeyEditPage() {
  const { name: urlName, env: urlEnv } = useParams<{
    name: string;
    env: string;
  }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const isEditMode = !!urlName && !!urlEnv && urlName !== "new";

  const [form, setForm] = useState({
    name: "",
    provider: "",
    key_type: "ApiKey" as KeyType,
    value: "",
    environment: "Development" as Environment,
    description: "",
    group_id: "",
    tags: "",
  });

  const { data: keys = [] } = useQuery({
    queryKey: ["keys"],
    queryFn: listKeys,
  });

  const { data: groups = [] } = useQuery({
    queryKey: ["groups"],
    queryFn: listGroups,
  });

  useEffect(() => {
    if (isEditMode) {
      const key = keys.find(
        (k) => k.name === urlName && k.environment === urlEnv
      );
      if (key) {
        setForm({
          name: key.name,
          provider: key.provider,
          key_type: key.key_type,
          value: "",
          environment: key.environment,
          description: key.description || "",
          group_id: key.group_id || "",
          tags: key.tags.join(", "),
        });
      }
    }
  }, [isEditMode, urlName, urlEnv, keys]);

  const mutation = useMutation({
    mutationFn: async () => {
      const tags = form.tags
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);

      if (isEditMode) {
        return updateKey({
          name: form.name,
          environment: form.environment,
          value: form.value || undefined,
          description: form.description || undefined,
          tags,
        });
      } else {
        return addKey({
          name: form.name,
          provider: form.provider,
          key_type: form.key_type,
          value: form.value,
          environment: form.environment,
          description: form.description || undefined,
          group_id: form.group_id || undefined,
          tags,
        });
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["keys"] });
      addNotification("success", isEditMode ? "密钥已更新" : "密钥已创建");
      navigate("/keys");
    },
    onError: (err) => {
      addNotification("error", `操作失败: ${err}`);
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name || !form.provider || (!isEditMode && !form.value)) {
      addNotification("error", "请填写所有必填字段");
      return;
    }
    mutation.mutate();
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div className="flex items-center space-x-4">
        <button
          onClick={() => navigate("/keys")}
          className="text-vault-muted hover:text-vault-text"
        >
          ← 返回
        </button>
        <h2 className="text-2xl font-bold text-vault-text">
          {isEditMode ? "编辑密钥" : "添加密钥"}
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
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            disabled={isEditMode}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary disabled:opacity-50"
            placeholder="例如: OpenAI API Key"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            提供商 *
          </label>
          <input
            type="text"
            value={form.provider}
            onChange={(e) => setForm({ ...form, provider: e.target.value })}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
            placeholder="例如: OpenAI"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            密钥类型
          </label>
          <select
            value={form.key_type}
            onChange={(e) =>
              setForm({ ...form, key_type: e.target.value as KeyType })
            }
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
          >
            {KEY_TYPES.map((type) => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            密钥值 {!isEditMode && "*"}
          </label>
          <input
            type="password"
            value={form.value}
            onChange={(e) => setForm({ ...form, value: e.target.value })}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
            placeholder={isEditMode ? "留空则不更新" : "输入密钥值"}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            环境
          </label>
          <select
            value={form.environment}
            onChange={(e) =>
              setForm({ ...form, environment: e.target.value as Environment })
            }
            disabled={isEditMode}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text disabled:opacity-50"
          >
            {ENVIRONMENTS.map((env) => (
              <option key={env} value={env}>
                {env}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            分组
          </label>
          <select
            value={form.group_id}
            onChange={(e) => setForm({ ...form, group_id: e.target.value })}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
          >
            <option value="">无分组</option>
            {groups.map((group) => (
              <option key={group.id} value={group.id}>
                {group.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            描述
          </label>
          <textarea
            value={form.description}
            onChange={(e) =>
              setForm({ ...form, description: e.target.value })
            }
            rows={3}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
            placeholder="可选描述"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-vault-muted mb-2">
            标签（逗号分隔）
          </label>
          <input
            type="text"
            value={form.tags}
            onChange={(e) => setForm({ ...form, tags: e.target.value })}
            className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
            placeholder="例如: production, ai, openai"
          />
        </div>

        <div className="flex justify-end space-x-3">
          <button
            type="button"
            onClick={() => navigate("/keys")}
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
