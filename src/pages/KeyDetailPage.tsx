import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listKeys, getKeyValue, deleteKey, testConnectivity } from "@/api";
import { useUIStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import {
  ArrowLeft, Eye, Copy, Pencil, Trash2, Link2, Tag, Clock, Hash,
} from "lucide-react";

export function KeyDetailPage() {
  const { name, env } = useParams<{ name: string; env: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const [showValue, setShowValue] = useState(false);
  const [keyValue, setKeyValue] = useState<string | null>(null);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const { data: keys = [] } = useQuery({ queryKey: ["keys"], queryFn: listKeys });
  const key = keys.find((k) => k.name === name && k.environment === env);

  const fetchValue = async () => {
    if (!name || !env) return;
    try {
      const value = await getKeyValue(name, env);
      setKeyValue(value);
      setShowValue(true);
    } catch (err) {
      addNotification("error", `获取失败: ${err}`);
    }
  };

  const deleteMutation = useMutation({
    mutationFn: () => deleteKey(name!, env!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["keys"] });
      addNotification("success", "密钥已删除");
      navigate("/keys");
    },
    onError: (err) => addNotification("error", `删除失败: ${err}`),
  });

  const testMutation = useMutation({
    mutationFn: () => testConnectivity(name!, env!),
    onSuccess: (result) =>
      addNotification(result.success ? "success" : "error", `${result.message}${result.latency_ms ? ` (${result.latency_ms}ms)` : ""}`),
    onError: (err) => addNotification("error", `测试失败: ${err}`),
  });

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    addNotification("success", "已复制到剪贴板");
  };

  if (!key) {
    return (
      <div className="text-center py-12 animate-fade-in">
        <p style={{ color: "var(--text-muted)" }}>密钥未找到</p>
        <button
          onClick={() => navigate("/keys")}
          className="mt-4 text-sm"
          style={{ color: "var(--brand-primary)" }}
        >
          返回列表
        </button>
      </div>
    );
  }

  const InfoCard = ({ label, value }: { label: string; value: string | number }) => (
    <div className="flex items-center justify-between py-2">
      <span className="text-xs" style={{ color: "var(--text-muted)" }}>{label}</span>
      <span className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>{value}</span>
    </div>
  );

  return (
    <div className="space-y-5 animate-fade-in max-w-3xl">
      {/* Header */}
      <div className="flex items-center justify-between">
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
          <div>
            <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>{key.name}</h2>
            <span
              className="px-2 py-0.5 rounded-md text-xs font-medium"
              style={{ background: "rgba(124,58,237,0.1)", color: "var(--brand-primary)" }}
            >
              {key.environment}
            </span>
          </div>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => testMutation.mutate()}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-smooth"
            style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)", color: "var(--text-secondary)" }}
          >
            <Link2 size={14} /> 测试连通性
          </button>
          <button
            onClick={() => navigate(`/keys/${name}/${env}/edit`)}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-smooth"
            style={{ background: "var(--brand-primary)", color: "white" }}
          >
            <Pencil size={14} /> 编辑
          </button>
          <button
            onClick={() => setShowDeleteDialog(true)}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-smooth"
            style={{ background: "var(--status-error-bg)", color: "var(--status-error)", border: "1px solid rgba(239,68,68,0.2)" }}
          >
            <Trash2 size={14} /> 删除
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {/* Basic info */}
        <div className="p-5 rounded-xl" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
          <h3 className="text-sm font-semibold mb-3" style={{ color: "var(--text-primary)" }}>基本信息</h3>
          <InfoCard label="提供商" value={key.provider} />
          <InfoCard label="类型" value={key.key_type} />
          <InfoCard label="版本" value={key.version} />
          <InfoCard label="使用次数" value={key.usage_count} />
        </div>

        {/* Time info */}
        <div className="p-5 rounded-xl" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
          <h3 className="text-sm font-semibold mb-3 flex items-center gap-2" style={{ color: "var(--text-primary)" }}>
            <Clock size={14} /> 时间信息
          </h3>
          <InfoCard label="创建时间" value={new Date(key.created_at).toLocaleString("zh-CN")} />
          <InfoCard label="更新时间" value={new Date(key.updated_at).toLocaleString("zh-CN")} />
          {key.expires_at && <InfoCard label="过期时间" value={new Date(key.expires_at).toLocaleString("zh-CN")} />}
          {key.last_used_at && <InfoCard label="最后使用" value={new Date(key.last_used_at).toLocaleString("zh-CN")} />}
        </div>
      </div>

      {/* Tags */}
      <div className="p-5 rounded-xl" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
        <h3 className="text-sm font-semibold mb-3 flex items-center gap-2" style={{ color: "var(--text-primary)" }}>
          <Tag size={14} /> 标签
        </h3>
        <div className="flex flex-wrap gap-2">
          {key.tags.length > 0 ? key.tags.map((tag) => (
            <span
              key={tag}
              className="px-3 py-1 rounded-full text-xs"
              style={{ background: "var(--bg-surface-hover)", color: "var(--text-secondary)", border: "1px solid var(--border-default)" }}
            >
              {tag}
            </span>
          )) : (
            <span className="text-xs" style={{ color: "var(--text-muted)" }}>无标签</span>
          )}
        </div>
      </div>

      {/* Key value */}
      <div className="p-5 rounded-xl" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
        <h3 className="text-sm font-semibold mb-3 flex items-center gap-2" style={{ color: "var(--text-primary)" }}>
          <Hash size={14} /> 密钥值
        </h3>
        {showValue && keyValue ? (
          <div className="flex items-center gap-2">
            <code
              className="flex-1 px-4 py-3 rounded-lg text-xs break-all"
              style={{ background: "var(--bg-primary)", color: "var(--text-primary)", fontFamily: "monospace" }}
            >
              {keyValue}
            </code>
            <button
              onClick={() => copyToClipboard(keyValue)}
              className="p-2 rounded-lg transition-smooth"
              style={{ color: "var(--text-muted)" }}
              onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
            >
              <Copy size={16} />
            </button>
          </div>
        ) : (
          <button
            onClick={fetchValue}
            className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm transition-smooth"
            style={{ background: "var(--bg-surface-hover)", color: "var(--text-secondary)", border: "1px solid var(--border-default)" }}
          >
            <Eye size={14} /> 显示密钥值
          </button>
        )}
      </div>

      <ConfirmDialog
        isOpen={showDeleteDialog}
        title="删除密钥"
        message={`确定要删除密钥 "${key.name}" 吗？此操作不可恢复。`}
        confirmLabel="删除"
        variant="danger"
        onConfirm={() => deleteMutation.mutate()}
        onCancel={() => setShowDeleteDialog(false)}
      />
    </div>
  );
}
