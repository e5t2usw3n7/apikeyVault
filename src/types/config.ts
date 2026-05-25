export type Theme = "Dark" | "Light";

export interface AppConfig {
  vault_path: string;
  auto_lock_minutes: number;
  clipboard_clear_seconds: number;
  theme: Theme;
  default_environment: string;
  audit_log_enabled: boolean;
}
