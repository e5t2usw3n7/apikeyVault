import { useVaultStore } from "@/store";
import { Shield, ShieldCheck, ShieldAlert } from "lucide-react";

export function StatusBar() {
  const vaultState = useVaultStore((s) => s.state);

  const stateConfig = {
    unlocked: { icon: ShieldCheck, label: "已解锁", color: "var(--status-success)" },
    locked: { icon: ShieldAlert, label: "已锁定", color: "var(--status-warning)" },
    uninitialized: { icon: Shield, label: "未初始化", color: "var(--text-muted)" },
  };

  const config = stateConfig[vaultState];
  const Icon = config.icon;

  return (
    <footer
      className="h-8 flex items-center justify-between px-4 text-xs shrink-0"
      style={{
        background: "var(--bg-secondary)",
        borderTop: "1px solid var(--border-default)",
        color: "var(--text-muted)",
      }}
    >
      <div className="flex items-center gap-2">
        <Icon size={12} style={{ color: config.color }} />
        <span>
          状态: <span style={{ color: config.color }}>{config.label}</span>
        </span>
      </div>
      <span>API Key Vault v1.0.0</span>
    </footer>
  );
}
