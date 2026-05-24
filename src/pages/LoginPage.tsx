import { useState } from "react";
import { useVaultStore, useUIStore } from "@/store";

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
    <div className="min-h-screen bg-vault-bg flex items-center justify-center">
      <div className="absolute top-4 right-4">
        <button
          onClick={toggleTheme}
          className="p-2 rounded-lg bg-vault-surface hover:bg-vault-border text-vault-muted"
        >
          {theme === "dark" ? "🌙" : "☀️"}
        </button>
      </div>

      <div className="bg-vault-surface rounded-2xl p-8 w-full max-w-md mx-4 shadow-xl">
        <div className="text-center mb-8">
          <div className="text-5xl mb-4">🔐</div>
          <h1 className="text-2xl font-bold text-vault-text">API Key Vault</h1>
          <p className="text-vault-muted mt-2">
            {isInitMode ? "初始化您的密钥保险库" : "解锁您的密钥保险库"}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-vault-muted mb-2">
              主密码
            </label>
            <div className="relative">
              <input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-4 py-3 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
                placeholder="输入主密码"
                autoFocus
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-vault-muted hover:text-vault-text"
              >
                {showPassword ? "🙈" : "👁️"}
              </button>
            </div>
          </div>

          {isInitMode && (
            <div>
              <label className="block text-sm font-medium text-vault-muted mb-2">
                确认密码
              </label>
              <input
                type={showPassword ? "text" : "password"}
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="w-full px-4 py-3 rounded-lg bg-vault-bg border border-vault-border text-vault-text focus:outline-none focus:border-vault-primary"
                placeholder="再次输入密码"
              />
            </div>
          )}

          {(localError || error) && (
            <div className="text-vault-error text-sm text-center">
              {localError || error}
            </div>
          )}

          <button
            type="submit"
            disabled={isLoading}
            className="w-full py-3 rounded-lg bg-vault-primary hover:bg-vault-primary/80 text-white font-medium transition-colors disabled:opacity-50"
          >
            {isLoading
              ? "处理中..."
              : isInitMode
              ? "初始化"
              : "解锁"}
          </button>
        </form>
      </div>
    </div>
  );
}
