import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getConfig, updateConfig, changePassword, resetVault } from "@/api";
import { useUIStore, useVaultStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const lock = useVaultStore((s) => s.lock);

  const [showResetDialog, setShowResetDialog] = useState(false);
  const [showPasswordForm, setShowPasswordForm] = useState(false);
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  const { data: config } = useQuery({
    queryKey: ["config"],
    queryFn: getConfig,
  });

  const updateMutation = useMutation({
    mutationFn: updateConfig,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["config"] });
      addNotification("success", "设置已保存");
    },
    onError: (err) => addNotification("error", `保存失败: ${err}`),
  });

  const passwordMutation = useMutation({
    mutationFn: () => changePassword(oldPassword, newPassword),
    onSuccess: () => {
      addNotification("success", "密码已更改");
      setShowPasswordForm(false);
      setOldPassword("");
      setNewPassword("");
      setConfirmPassword("");
    },
    onError: (err) => addNotification("error", `更改密码失败: ${err}`),
  });

  const resetMutation = useMutation({
    mutationFn: resetVault,
    onSuccess: () => {
      addNotification("success", "Vault 已重置");
      lock();
    },
    onError: (err) => addNotification("error", `重置失败: ${err}`),
  });

  const handlePasswordChange = (e: React.FormEvent) => {
    e.preventDefault();
    if (newPassword !== confirmPassword) {
      addNotification("error", "两次密码不一致");
      return;
    }
    if (newPassword.length < 8) {
      addNotification("error", "密码长度至少 8 位");
      return;
    }
    passwordMutation.mutate();
  };

  if (!config) {
    return <div className="text-center py-8 text-vault-muted">加载中...</div>;
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <h2 className="text-2xl font-bold text-vault-text">设置</h2>

      {/* 通用设置 */}
      <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
        <h3 className="text-lg font-semibold text-vault-text mb-4">通用设置</h3>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-vault-muted mb-2">
              自动锁定时间（分钟）
            </label>
            <input
              type="number"
              value={config.auto_lock_minutes}
              onChange={(e) =>
                updateMutation.mutate({
                  auto_lock_minutes: parseInt(e.target.value) || 5,
                })
              }
              min="1"
              max="60"
              className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-vault-muted mb-2">
              剪贴板自动清除时间（秒）
            </label>
            <input
              type="number"
              value={config.clipboard_clear_seconds}
              onChange={(e) =>
                updateMutation.mutate({
                  clipboard_clear_seconds: parseInt(e.target.value) || 30,
                })
              }
              min="5"
              max="120"
              className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-vault-muted mb-2">
              默认环境
            </label>
            <select
              value={config.default_environment}
              onChange={(e) =>
                updateMutation.mutate({ default_environment: e.target.value })
              }
              className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
            >
              <option value="Production">Production</option>
              <option value="Staging">Staging</option>
              <option value="Development">Development</option>
              <option value="Testing">Testing</option>
            </select>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-vault-muted">启用审计日志</span>
            <button
              onClick={() =>
                updateMutation.mutate({
                  audit_log_enabled: !config.audit_log_enabled,
                })
              }
              className={`w-12 h-6 rounded-full transition-colors ${
                config.audit_log_enabled ? "bg-vault-success" : "bg-vault-border"
              }`}
            >
              <div
                className={`w-5 h-5 rounded-full bg-white transform transition-transform ${
                  config.audit_log_enabled ? "translate-x-6" : "translate-x-0.5"
                }`}
              />
            </button>
          </div>
        </div>
      </div>

      {/* 安全设置 */}
      <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
        <h3 className="text-lg font-semibold text-vault-text mb-4">安全设置</h3>
        <div className="space-y-4">
          <button
            onClick={() => setShowPasswordForm(!showPasswordForm)}
            className="w-full text-left px-4 py-3 rounded-lg bg-vault-bg border border-vault-border text-vault-text hover:border-vault-primary"
          >
            更改主密码
          </button>

          {showPasswordForm && (
            <form onSubmit={handlePasswordChange} className="space-y-4">
              <input
                type="password"
                value={oldPassword}
                onChange={(e) => setOldPassword(e.target.value)}
                placeholder="当前密码"
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              />
              <input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="新密码"
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              />
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="确认新密码"
                className="w-full px-4 py-2 rounded-lg bg-vault-bg border border-vault-border text-vault-text"
              />
              <button
                type="submit"
                disabled={passwordMutation.isPending}
                className="w-full py-2 bg-vault-primary hover:bg-vault-primary/80 text-white rounded-lg disabled:opacity-50"
              >
                {passwordMutation.isPending ? "更改中..." : "更改密码"}
              </button>
            </form>
          )}
        </div>
      </div>

      {/* 危险区域 */}
      <div className="bg-vault-surface rounded-lg p-6 border border-vault-danger">
        <h3 className="text-lg font-semibold text-vault-error mb-4">危险区域</h3>
        <p className="text-vault-muted mb-4">
          重置 Vault 将删除所有密钥、分组和审计日志。此操作不可恢复。
        </p>
        <button
          onClick={() => setShowResetDialog(true)}
          className="px-4 py-2 bg-vault-danger hover:bg-vault-danger/80 text-white rounded-lg"
        >
          重置 Vault
        </button>
      </div>

      <ConfirmDialog
        isOpen={showResetDialog}
        title="重置 Vault"
        message="确定要重置 Vault 吗？所有数据将被永久删除，此操作不可恢复。"
        confirmLabel="重置"
        variant="danger"
        onConfirm={() => resetMutation.mutate()}
        onCancel={() => setShowResetDialog(false)}
      />
    </div>
  );
}
