import { useNavigate, useLocation } from "react-router-dom";
import { useVaultStore, useUIStore } from "@/store";

const navItems = [
  { path: "/", label: "仪表盘", icon: "📊" },
  { path: "/keys", label: "密钥管理", icon: "🔑" },
  { path: "/groups", label: "分组管理", icon: "📁" },
  { path: "/audit", label: "审计日志", icon: "📋" },
  { path: "/import-export", label: "导入导出", icon: "📤" },
  { path: "/settings", label: "设置", icon: "⚙️" },
];

export function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const lock = useVaultStore((s) => s.lock);
  const { sidebarCollapsed, toggleSidebar, toggleTheme, theme } = useUIStore();

  return (
    <aside
      className={`bg-vault-surface border-r border-vault-border flex flex-col transition-all duration-300 ${
        sidebarCollapsed ? "w-16" : "w-56"
      }`}
    >
      <div className="p-4 border-b border-vault-border flex items-center justify-between">
        {!sidebarCollapsed && (
          <h1 className="text-lg font-bold text-vault-primary truncate">
            API Key Vault
          </h1>
        )}
        <button
          onClick={toggleSidebar}
          className="p-1 hover:bg-vault-border rounded"
        >
          {sidebarCollapsed ? "→" : "←"}
        </button>
      </div>

      <nav className="flex-1 py-4">
        {navItems.map((item) => (
          <button
            key={item.path}
            onClick={() => navigate(item.path)}
            className={`w-full flex items-center px-4 py-3 hover:bg-vault-border transition-colors ${
              location.pathname === item.path
                ? "bg-vault-primary/20 text-vault-primary border-r-2 border-vault-primary"
                : "text-vault-muted"
            }`}
          >
            <span className="text-xl">{item.icon}</span>
            {!sidebarCollapsed && (
              <span className="ml-3 truncate">{item.label}</span>
            )}
          </button>
        ))}
      </nav>

      <div className="p-4 border-t border-vault-border space-y-2">
        <button
          onClick={toggleTheme}
          className="w-full flex items-center px-3 py-2 hover:bg-vault-border rounded text-vault-muted"
        >
          <span>{theme === "dark" ? "🌙" : "☀️"}</span>
          {!sidebarCollapsed && (
            <span className="ml-3">{theme === "dark" ? "深色" : "浅色"}模式</span>
          )}
        </button>
        <button
          onClick={lock}
          className="w-full flex items-center px-3 py-2 hover:bg-vault-border rounded text-vault-error"
        >
          <span>🔒</span>
          {!sidebarCollapsed && <span className="ml-3">锁定</span>}
        </button>
      </div>
    </aside>
  );
}
