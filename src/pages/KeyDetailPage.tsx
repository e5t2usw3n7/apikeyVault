import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listKeys, getKeyValue, deleteKey, testConnectivity } from "@/api";
import { useUIStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";

export function KeyDetailPage() {
  const { name, env } = useParams<{ name: string; env: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);

  const [showValue, setShowValue] = useState(false);
  const [keyValue, setKeyValue] = useState<string | null>(null);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const { data: keys = [] } = useQuery({
    queryKey: ["keys"],
    queryFn: listKeys,
  });

  const key = keys.find((k) => k.name === name && k.environment === env);

  const fetchValue = async () => {
    if (!name || !env) return;
    try {
      const value = await getKeyValue(name, env);
      setKeyValue(value);
      setShowValue(true);
    } catch (err) {
      addNotification("error", `获取密钥值失败: ${err}`);
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
    onSuccess: (result) => {
      addNotification(
        result.success ? "success" : "error",
        `${result.message}${result.latency_ms ? ` (${result.latency_ms}ms)` : ""}`
      );
    },
    onError: (err) => addNotification("error", `测试失败: ${err}`),
  });

  if (!key) {
    return (
      <div className="text-center py-8">
        <p className="text-vault-muted">密钥未找到</p>
        <button
          onClick={() => navigate("/keys")}
          className="mt-4 text-vault-primary hover:underline"
        >
          返回列表
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <button
            onClick={() => navigate("/keys")}
            className="text-vault-muted hover:text-vault-text"
          >
            ← 返回
          </button>
          <h2 className="text-2xl font-bold text-vault-text">{key.name}</h2>
        </div>
        <div className="flex space-x-3">
          <button
            onClick={() => testMutation.mutate()}
            className="px-4 py-2 bg-vault-surface border border-vault-border hover:bg-vault-border rounded-lg text-vault-text"
          >
            测试连通性
          </button>
          <button
            onClick={() => navigate(`/keys/${name}/${env}/edit`)}
            className="px-4 py-2 bg-vault-primary hover:bg-vault-primary/80 text-white rounded-lg"
          >
            编辑
          </button>
          <button
            onClick={() => setShowDeleteDialog(true)}
            className="px-4 py-2 bg-vault-danger hover:bg-vault-danger/80 text-white rounded-lg"
          >
            删除
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 基本信息 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <h3 className="text-lg font-semibold text-vault-text mb-4">基本信息</h3>
          <dl className="space-y-3">
            <div className="flex justify-between">
              <dt className="text-vault-muted">提供商</dt>
              <dd className="text-vault-text">{key.provider}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-vault-muted">类型</dt>
              <dd className="text-vault-text">{key.key_type}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-vault-muted">环境</dt>
              <dd>
                <span className="px-2 py-1 rounded text-xs bg-vault-primary/20 text-vault-primary">
                  {key.environment}
                </span>
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-vault-muted">版本</dt>
              <dd className="text-vault-text">{key.version}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-vault-muted">使用次数</dt>
              <dd className="text-vault-text">{key.usage_count}</dd>
            </div>
          </dl>
        </div>

        {/* 时间信息 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <h3 className="text-lg font-semibold text-vault-text mb-4">时间信息</h3>
          <dl className="space-y-3">
            <div className="flex justify-between">
              <dt className="text-vault-muted">创建时间</dt>
              <dd className="text-vault-text">
                {new Date(key.created_at).toLocaleString("zh-CN")}
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-vault-muted">更新时间</dt>
              <dd className="text-vault-text">
                {new Date(key.updated_at).toLocaleString("zh-CN")}
              </dd>
            </div>
            {key.expires_at && (
              <div className="flex justify-between">
                <dt className="text-vault-muted">过期时间</dt>
                <dd className="text-vault-text">
                  {new Date(key.expires_at).toLocaleString("zh-CN")}
                </dd>
              </div>
            )}
            {key.last_used_at && (
              <div className="flex justify-between">
                <dt className="text-vault-muted">最后使用</dt>
                <dd className="text-vault-text">
                  {new Date(key.last_used_at).toLocaleString("zh-CN")}
                </dd>
              </div>
            )}
          </dl>
        </div>

        {/* 描述和标签 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border lg:col-span-2">
          <h3 className="text-lg font-semibold text-vault-text mb-4">描述和标签</h3>
          {key.description && (
            <p className="text-vault-muted mb-4">{key.description}</p>
          )}
          <div className="flex flex-wrap gap-2">
            {key.tags.map((tag) => (
              <span
                key={tag}
                className="px-3 py-1 rounded-full bg-vault-border text-vault-text text-sm"
              >
                {tag}
              </span>
            ))}
            {key.tags.length === 0 && (
              <span className="text-vault-muted">无标签</span>
            )}
          </div>
        </div>

        {/* 密钥值 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border lg:col-span-2">
          <h3 className="text-lg font-semibold text-vault-text mb-4">密钥值</h3>
          {showValue && keyValue ? (
            <div className="bg-vault-bg rounded-lg p-4 font-mono text-sm break-all">
              {keyValue}
            </div>
          ) : (
            <button
              onClick={fetchValue}
              className="px-4 py-2 bg-vault-border hover:bg-vault-border/80 rounded-lg text-vault-text"
            >
              显示密钥值
            </button>
          )}
        </div>
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
