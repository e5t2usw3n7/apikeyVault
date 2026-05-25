import { useNavigate, useLocation } from "react-router-dom";
import { useVaultStore, useUIStore } from "@/store";
import {
  LayoutDashboard,
  KeyRound,
  FolderOpen,
  ScrollText,
  ArrowDownUp,
  Settings,
  Lock,
  Sun,
  Moon,
  ChevronLeft,
  ChevronRight,
  Shield,
} from "lucide-react";

const navItems = [
  { path: "/", label: "仪表盘", icon: LayoutDashboard },
  { path: "/keys", label: "密钥管理", icon: KeyRound },
  { path: "/groups", label: "分组管理", icon: FolderOpen },
  { path: "/audit", label: "审计日志", icon: ScrollText },
  { path: "/import-export", label: "导入导出", icon: ArrowDownUp },
  { path: "/settings", label: "设置", icon: Settings },
];

export function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const lock = useVaultStore((s) => s.lock);
  const { sidebarCollapsed, toggleSidebar, toggleTheme, theme } = useUIStore();

  return (
    <aside
      className="flex flex-col transition-all duration-300 ease-out"
      style={{
        width: sidebarCollapsed ? 72 : 240,
        background: "var(--bg-secondary)",
        borderRight: "1px solid var(--border-default)",
      }}
    >
      {/* Logo */}
      <div
        className="flex items-center gap-3 px-5 h-16 shrink-0"
        style={{ borderBottom: "1px solid var(--border-default)" }}
      >
        <div
          className="w-8 h-8 rounded-lg flex items-center justify-center"
          style={{ background: "var(--brand-primary)" }}
        >
          <Shield size={18} color="white" />
        </div>
        {!sidebarCollapsed && (
          <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
            API Key Vault
          </span>
        )}
        <button
          onClick={toggleSidebar}
          className="ml-auto p-1.5 rounded-md transition-smooth"
          style={{ color: "var(--text-muted)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
        >
          {sidebarCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-3 px-3 space-y-1">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          const Icon = item.icon;
          return (
            <button
              key={item.path}
              onClick={() => navigate(item.path)}
              className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-smooth text-sm"
              style={{
                background: isActive ? "rgba(124, 58, 237, 0.12)" : "transparent",
                color: isActive ? "var(--brand-primary)" : "var(--text-secondary)",
                fontWeight: isActive ? 500 : 400,
              }}
              onMouseEnter={(e) => {
                if (!isActive) e.currentTarget.style.background = "var(--bg-surface-hover)";
              }}
              onMouseLeave={(e) => {
                if (!isActive) e.currentTarget.style.background = "transparent";
              }}
            >
              <Icon size={18} strokeWidth={isActive ? 2 : 1.5} />
              {!sidebarCollapsed && <span>{item.label}</span>}
            </button>
          );
        })}
      </nav>

      {/* Bottom actions */}
      <div className="p-3 space-y-1" style={{ borderTop: "1px solid var(--border-default)" }}>
        <button
          onClick={toggleTheme}
          className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-smooth text-sm"
          style={{ color: "var(--text-secondary)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--bg-surface-hover)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
        >
          {theme === "dark" ? <Moon size={18} /> : <Sun size={18} />}
          {!sidebarCollapsed && <span>{theme === "dark" ? "深色模式" : "浅色模式"}</span>}
        </button>
        <button
          onClick={lock}
          className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-smooth text-sm"
          style={{ color: "var(--status-error)" }}
          onMouseEnter={(e) => (e.currentTarget.style.background = "var(--status-error-bg)")}
          onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
        >
          <Lock size={18} strokeWidth={1.5} />
          {!sidebarCollapsed && <span>锁定</span>}
        </button>
      </div>
    </aside>
  );
}
