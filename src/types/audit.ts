export type AuditAction =
  | "VaultInitialized"
  | "VaultUnlocked"
  | "VaultLocked"
  | "KeyAdded"
  | "KeyUpdated"
  | "KeyDeleted"
  | "KeyRotated"
  | "KeyRenamed"
  | "KeyViewed"
  | "KeyCopied"
  | "KeyTested"
  | "GroupCreated"
  | "GroupUpdated"
  | "GroupDeleted"
  | "ImportCompleted"
  | "ExportCompleted"
  | "BackupCreated"
  | "VaultReset";

export interface AuditEntry {
  id: number;
  timestamp: string;
  action: AuditAction;
  resource_type: string;
  resource_id: string;
  details: Record<string, unknown>;
}
