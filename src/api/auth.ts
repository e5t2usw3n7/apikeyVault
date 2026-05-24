import { invoke } from "@tauri-apps/api/core";

export interface VaultStatus {
  state: string;
  is_initialized: boolean;
}

export async function getVaultStatus(): Promise<VaultStatus> {
  return invoke<VaultStatus>("vault_status");
}

export async function initVault(password: string): Promise<void> {
  return invoke("vault_init", { password });
}

export async function unlockVault(password: string): Promise<void> {
  return invoke("vault_unlock", { password });
}

export async function lockVault(): Promise<void> {
  return invoke("vault_lock");
}

export async function tryRestoreSession(): Promise<boolean> {
  return invoke<boolean>("vault_try_restore_session");
}

export async function changePassword(
  oldPassword: string,
  newPassword: string
): Promise<void> {
  return invoke("vault_change_password", { oldPassword, newPassword });
}
