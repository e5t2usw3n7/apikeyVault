import { useState } from "react";
import { useVaultStore, useUIStore } from "@/store";
import { Shield, Eye, EyeOff, Lock, Sun, Moon } from "lucide-react";

export function LoginPage() {
  const { state, init, unlock, isLoading, error, clearError } = useVaultStore();
  const { theme, toggleTheme } = useUIStore();
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [localError, setLocalError] = useState("");

  const isInitMode = state === "uninitialized";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError("");
    clearError();

    if (!password) {
      setLocalError("请输入密码");
      return;
    }

    if (isInitMode) {
      if (password !== confirmPassword) {
        setLocalError("两次密码不一致");
        return;
      }
      if (password.length < 8) {
        setLocalError("密码长度至少 8 位");
        return;
      }
      await init(password);
    } else {
      await unlock(password);
    }
  };

  return (
    <div
      className="min-h-screen flex items-center justify-center relative"
      style={{
        background: "linear-gradient(135deg, var(--bg-primary) 0%, #0d0d2b 50%, #1a0a2e 100%)",
      }}
    >
      {/* Background decoration */}
      <div
        className="absolute inset-0 opacity-30"
        style={{
          backgroundImage:
            "radial-gradient(circle at 20% 50%, rgba(124,58,237,0.15) 0%, transparent 50%), radial-gradient(circle at 80% 20%, rgba(6,182,212,0.1) 0%, transparent 50%)",
        }}
      />

      {/* Theme toggle */}
      <button
        onClick={toggleTheme}
        className="absolute top-6 right-6 p-2.5 rounded-xl transition-smooth z-10"
        style={{
          background: "var(--bg-surface)",
          color: "var(--text-muted)",
          border: "1px solid var(--border-default)",
        }}
      >
        {theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
      </button>

      {/* Login card */}
      <div
        className="relative z-10 w-full max-w-md mx-4 animate-scale-in"
        style={{
          background: "var(--glass-bg)",
          backdropFilter: "blur(24px)",
          border: "1px solid var(--glass-border)",
          borderRadius: "var(--radius-xl)",
          boxShadow: "var(--shadow-lg)",
        }}
      >
        <div className="p-8">
          {/* Header */}
          <div className="text-center mb-8">
            <div
              className="w-16 h-16 mx-auto mb-4 rounded-2xl flex items-center justify-center"
              style={{
                background: "linear-gradient(135deg, var(--brand-primary), #a855f7)",
                boxShadow: "0 0 30px var(--brand-primary-glow)",
              }}
            >
              <Shield size={32} color="white" />
            </div>
            <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>
              API Key Vault
            </h1>
            <p className="mt-2 text-sm" style={{ color: "var(--text-muted)" }}>
              {isInitMode ? "创建您的密钥保险库" : "解锁您的密钥保险库"}
            </p>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>
                主密码
              </label>
              <div className="relative">
                <div className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: "var(--text-muted)" }}>
                  <Lock size={16} />
                </div>
                <input
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full pl-10 pr-10 py-3 rounded-xl text-sm transition-smooth"
                  style={{
                    background: "var(--bg-surface)",
                    border: "1px solid var(--border-default)",
                    color: "var(--text-primary)",
                    outline: "none",
                  }}
                  placeholder="输入主密码"
                  autoFocus
                  onFocus={(e) => (e.target.style.borderColor = "var(--brand-primary)")}
                  onBlur={(e) => (e.target.style.borderColor = "var(--border-default)")}
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 p-1 rounded-md transition-smooth"
                  style={{ color: "var(--text-muted)" }}
                >
                  {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>

            {isInitMode && (
              <div className="animate-fade-in">
                <label className="block text-xs font-medium mb-2" style={{ color: "var(--text-secondary)" }}>
                  确认密码
                </label>
                <div className="relative">
                  <div className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: "var(--text-muted)" }}>
                    <Lock size={16} />
                  </div>
                  <input
                    type={showPassword ? "text" : "password"}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    className="w-full pl-10 pr-4 py-3 rounded-xl text-sm transition-smooth"
                    style={{
                      background: "var(--bg-surface)",
                      border: "1px solid var(--border-default)",
                      color: "var(--text-primary)",
                      outline: "none",
                    }}
                    placeholder="再次输入密码"
                    onFocus={(e) => (e.target.style.borderColor = "var(--brand-primary)")}
                    onBlur={(e) => (e.target.style.borderColor = "var(--border-default)")}
                  />
                </div>
              </div>
            )}

            {(localError || error) && (
              <div
                className="flex items-center gap-2 px-3 py-2.5 rounded-lg text-sm animate-fade-in"
                style={{ background: "var(--status-error-bg)", color: "var(--status-error)" }}
              >
                <span>{localError || error}</span>
              </div>
            )}

            <button
              type="submit"
              disabled={isLoading}
              className="w-full py-3 rounded-xl text-sm font-semibold transition-smooth"
              style={{
                background: "var(--brand-primary)",
                color: "white",
                boxShadow: "0 0 20px var(--brand-primary-glow)",
                opacity: isLoading ? 0.7 : 1,
              }}
              onMouseEnter={(e) => !isLoading && (e.currentTarget.style.background = "var(--brand-primary-hover)")}
              onMouseLeave={(e) => !isLoading && (e.currentTarget.style.background = "var(--brand-primary)")}
            >
              {isLoading ? "处理中..." : isInitMode ? "创建保险库" : "解锁"}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
