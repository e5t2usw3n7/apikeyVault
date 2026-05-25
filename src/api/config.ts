import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "@/types";

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function updateConfig(params: {
  autoLockMinutes?: number;
  clipboardClearSeconds?: number;
  theme?: string;
  defaultEnvironment?: string;
  auditLogEnabled?: boolean;
}): Promise<void> {
  return invoke("update_config", params);
}

export async function backupVault(filePath: string): Promise<void> {
  return invoke("backup_vault", { filePath });
}

export async function restoreVault(filePath: string): Promise<void> {
  return invoke("restore_vault", { filePath });
}

export async function resetVault(): Promise<void> {
  return invoke("reset_vault");
}
