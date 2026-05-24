import { invoke } from "@tauri-apps/api/core";
import type { AuditEntry } from "@/types";

export async function getAuditLogs(limit?: number): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>("get_audit_logs", { limit });
}
