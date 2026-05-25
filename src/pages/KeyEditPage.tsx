import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listKeys, addKey, updateKey, listGroups } from "@/api";
import { useUIStore } from "@/store";
import { ArrowLeft, Save } from "lucide-react";
import type { KeyType, Environment } from "@/types";

const KEY_TYPES: KeyType[] = ["ApiKey", "BearerToken", "BasicAuth", "OAuth2", "Jwt", "SshKey", "Other"];
const ENVIRONMENTS: Environment[] = ["Production", "Staging", "Development", "Testing"];

export function KeyEditPage() {
  const { name: urlName, env: urlEnv } = useParams<{ name: string; env: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const isEditMode = !!urlName && !!urlEnv && urlName !== "new";

  const [form, setForm] = useState({
    name: "", provider: "", key_type: "ApiKey" as KeyType,
    value: "", environment: "Development" as Environment,
    description: "", group_id: "", tags: "",
  });

  const { data: keys = [] } = useQuery({ queryKey: ["keys"], queryFn: listKeys });
  const { data: groups = [] } = useQuery({ queryKey: ["groups"], queryFn: listGroups });

  useEffect(() => {
    if (isEditMode) {
      const key = keys.find((k) => k.name === urlName && k.environment === urlEnv);
      if (key) {
        setForm({
          name: key.name, provider: key.provider, key_type: key.key_type,
          value: "", environment: key.environment,
          description: key.description || "", group_id: key.group_id || "",
          tags: key.tags.join(", "),
        });
      }
    }
  }, [isEditMode, urlName, urlEnv, keys]);

  const mutation = useMutation({
    mutationFn: async () => {
      const tags = form.tags.split(",").map((t) => t.trim()).filter(Boolean);
      if (isEditMode) {
        return updateKey({ name: form.name, environment: form.environment, value: form.value || undefined, description: form.description || undefined, tags });
      }
      return addKey({ name: form.name, provider: form.provider, key_type: form.key_type, value: form.value, environment: form.environment, description: form.description || undefined, group_id: form.group_id || undefined, tags });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["keys"] });
      addNotification("success", isEditMode ? "密钥已更新" : "密钥已创建");
      navigate("/keys");
    },
    onError: (err) => addNotification("error", `操作失败: ${err}`),
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name || !form.provider || (!isEditMode && !form.value)) {
      addNotification("error", "请填写所有必填字段");
      return;
    }
    mutation.mutate();
  };

  const inputStyle = {
    background: "var(--bg-surface)",
    border: "1px solid var(--border-default)",
    color: "var(--text-primary)",
    outline: "none",
  };

  const InputField = ({ label, required, ...props }: { label: string; required?: boolean } & React.InputHTMLAttributes<HTMLInputElement>) => (
    <div>
      <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>
        {label} {required && <span style={{ color: "var(--status-error)" }}>*</span>}
      </label>
      <input
        {...props}
        className="w-full px-3 py-2.5 rounded-lg text-sm transition-smooth"
        style={inputStyle}
        onFocus={(e) => (e.target.style.borderColor = "var(--brand-primary)")}
        onBlur={(e) => (e.target.style.borderColor = "var(--border-default)")}
      />
    </div>
  );

  const SelectField = ({ label, options, ...props }: { label: string; options: { value: string; label: string }[] } & React.SelectHTMLAttributes<HTMLSelectElement>) => (
    <div>
      <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>{label}</label>
      <select {...props} className="w-full px-3 py-2.5 rounded-lg text-sm appearance-none transition-smooth" style={inputStyle}>
        {options.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
      </select>
    </div>
  );

  return (
    <div className="max-w-xl mx-auto space-y-5 animate-fade-in">
      <div className="flex items-center gap-3">
        <button
          onClick={() => navigate("/keys")}
          className="p-2 rounded-lg transition-smooth"
          style={{ color: "var(--text-muted)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
        >
          <ArrowLeft size={18} />
        </button>
        <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>
          {isEditMode ? "编辑密钥" : "添加密钥"}
        </h2>
      </div>

      <form
        onSubmit={handleSubmit}
        className="p-6 rounded-xl space-y-5"
        style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}
      >
        <InputField label="名称" required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} disabled={isEditMode} placeholder="例如: OpenAI API Key" />
        <InputField label="提供商" required value={form.provider} onChange={(e) => setForm({ ...form, provider: e.target.value })} placeholder="例如: OpenAI" />
        <SelectField label="密钥类型" value={form.key_type} onChange={(e) => setForm({ ...form, key_type: e.target.value as KeyType })} options={KEY_TYPES.map((t) => ({ value: t, label: t }))} />
        <InputField label="密钥值" required={!isEditMode} type="password" value={form.value} onChange={(e) => setForm({ ...form, value: e.target.value })} placeholder={isEditMode ? "留空则不更新" : "输入密钥值"} />
        <SelectField label="环境" value={form.environment} onChange={(e) => setForm({ ...form, environment: e.target.value as Environment })} options={ENVIRONMENTS.map((e) => ({ value: e, label: e }))} />
        <SelectField label="分组" value={form.group_id} onChange={(e) => setForm({ ...form, group_id: e.target.value })} options={[{ value: "", label: "无分组" }, ...groups.map((g) => ({ value: g.id, label: g.name }))]} />
        <div>
          <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>描述</label>
          <textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} rows={3} className="w-full px-3 py-2.5 rounded-lg text-sm transition-smooth" style={inputStyle} placeholder="可选描述" />
        </div>
        <InputField label="标签（逗号分隔）" value={form.tags} onChange={(e) => setForm({ ...form, tags: e.target.value })} placeholder="例如: production, ai, openai" />

        <div className="flex justify-end gap-3 pt-2">
          <button type="button" onClick={() => navigate("/keys")} className="px-5 py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--bg-surface-hover)", color: "var(--text-secondary)", border: "1px solid var(--border-default)" }}>
            取消
          </button>
          <button type="submit" disabled={mutation.isPending} className="flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-medium transition-smooth" style={{ background: "var(--brand-primary)", color: "white", opacity: mutation.isPending ? 0.7 : 1 }}>
            <Save size={14} />
            {mutation.isPending ? "保存中..." : isEditMode ? "更新" : "创建"}
          </button>
        </div>
      </form>
    </div>
  );
}
