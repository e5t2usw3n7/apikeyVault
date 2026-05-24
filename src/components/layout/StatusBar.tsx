import { useVaultStore } from "@/store";

export function StatusBar() {
  const vaultState = useVaultStore((s) => s.state);

  return (
    <footer className="bg-vault-surface border-t border-vault-border px-4 py-2 flex items-center justify-between text-sm text-vault-muted">
      <div className="flex items-center space-x-4">
        <span>
          状态:{" "}
          <span
            className={
              vaultState === "unlocked"
                ? "text-vault-success"
                : "text-vault-warning"
            }
          >
            {vaultState === "unlocked"
              ? "已解锁"
              : vaultState === "locked"
              ? "已锁定"
              : "未初始化"}
          </span>
        </span>
      </div>
      <div>API Key Vault v1.0.0</div>
    </footer>
  );
}
