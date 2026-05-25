import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getConfig, updateConfig, resetVault } from "@/api";
import { useUIStore, useVaultStore } from "@/store";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { Settings, Clock, Clipboard, Globe, Shield, AlertTriangle } from "lucide-react";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const addNotification = useUIStore((s) => s.addNotification);
  const lock = useVaultStore((s) => s.lock);
  const [showResetDialog, setShowResetDialog] = useState(false);

  const { data: config, isLoading, error } = useQuery({ queryKey: ["config"], queryFn: getConfig });

  const updateMutation = useMutation({
    mutationFn: updateConfig,
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["config"] }); addNotification("success", "设置已保存"); },
    onError: (err) => addNotification("error", `保存失败: ${err}`),
  });

  const resetMutation = useMutation({
    mutationFn: resetVault,
    onSuccess: () => { addNotification("success", "Vault 已重置"); lock(); },
    onError: (err) => addNotification("error", `重置失败: ${err}`),
  });

  if (isLoading) return <div className="text-center py-12" style={{ color: "var(--text-muted)" }}>加载中...</div>;
  if (error) return <div className="text-center py-12" style={{ color: "var(--status-error)" }}>加载失败: {String(error)}</div>;
  if (!config) return null;

  const SettingRow = ({ icon: Icon, label, children }: { icon: typeof Settings; label: string; children: React.ReactNode }) => (
    <div className="flex items-center justify-between py-3">
      <div className="flex items-center gap-3">
        <Icon size={16} style={{ color: "var(--text-muted)" }} />
        <span className="text-sm" style={{ color: "var(--text-secondary)" }}>{label}</span>
      </div>
      {children}
    </div>
  );

  const inputStyle = { background: "var(--bg-surface)", border: "1px solid var(--border-default)", color: "var(--text-primary)", outline: "none" };

  return (
    <div className="max-w-2xl mx-auto space-y-5 animate-fade-in">
      <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>设置</h2>

      {/* General */}
      <div className="p-5 rounded-xl" style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}>
        <h3 className="text-sm font-semibold mb-3" style={{ color: "var(--text-primary)" }}>通用设置</h3>
        <div className="divide-y" style={{ borderColor: "var(--border-default)" }}>
          <SettingRow icon={Clock} label="自动锁定时间（分钟）">
            <input type="number" value={config.auto_lock_minutes} onChange={(e) => updateMutation.mutate({ autoLockMinutes: parseInt(e.target.value) || 5 })} min={1} max={60} className="w-20 px-2 py-1.5 rounded-lg text-sm text-right" style={inputStyle} />
          </SettingRow>
          <SettingRow icon={Clipboard} label="剪贴板自动清除（秒）">
            <input type="number" value={config.clipboard_clear_seconds} onChange={(e) => updateMutation.mutate({ clipboardClearSeconds: parseInt(e.target.value) || 30 })} min={5} max={120} className="w-20 px-2 py-1.5 rounded-lg text-sm text-right" style={inputStyle} />
          </SettingRow>
          <SettingRow icon={Globe} label="默认环境">
            <select value={config.default_environment} onChange={(e) => updateMutation.mutate({ defaultEnvironment: e.target.value })} className="px-3 py-1.5 rounded-lg text-sm appearance-none" style={inputStyle}>
              {["Production", "Staging", "Development", "Testing"].map((e) => <option key={e} value={e}>{e}</option>)}
            </select>
          </SettingRow>
          <SettingRow icon={Shield} label="启用审计日志">
            <button
              onClick={() => updateMutation.mutate({ auditLogEnabled: !config.audit_log_enabled })}
              className="w-10 h-5 rounded-full transition-smooth relative"
              style={{ background: config.audit_log_enabled ? "var(--status-success)" : "var(--border-hover)" }}
            >
              <div
                className="w-4 h-4 rounded-full bg-white absolute top-0.5 transition-smooth"
                style={{ left: config.audit_log_enabled ? 22 : 2 }}
              />
            </button>
          </SettingRow>
        </div>
      </div>

      {/* Danger zone */}
      <div className="p-5 rounded-xl" style={{ background: "var(--bg-surface)", border: "1px solid rgba(239,68,68,0.2)" }}>
        <h3 className="text-sm font-semibold mb-2 flex items-center gap-2" style={{ color: "var(--status-error)" }}>
          <AlertTriangle size={14} /> 危险区域
        </h3>
        <p className="text-xs mb-4" style={{ color: "var(--text-muted)" }}>
          重置 Vault 将删除所有密钥、分组和审计日志。此操作不可恢复。
        </p>
        <button
          onClick={() => setShowResetDialog(true)}
          className="px-4 py-2 rounded-lg text-sm font-medium transition-smooth"
          style={{ background: "var(--status-error-bg)", color: "var(--status-error)", border: "1px solid rgba(239,68,68,0.2)" }}
        >
          重置 Vault
        </button>
      </div>

      <ConfirmDialog isOpen={showResetDialog} title="重置 Vault" message="确定要重置 Vault 吗？所有数据将被永久删除。" confirmLabel="重置" variant="danger" onConfirm={() => resetMutation.mutate()} onCancel={() => setShowResetDialog(false)} />
    </div>
  );
}
