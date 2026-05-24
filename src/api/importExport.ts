import { invoke } from "@tauri-apps/api/core";

export interface ImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

export async function importKeys(
  format: string,
  content: string,
  environment: string
): Promise<ImportResult> {
  return invoke<ImportResult>("import_keys", { format, content, environment });
}

export async function exportKeys(
  format: string,
  environment?: string
): Promise<string> {
  return invoke<string>("export_keys", { format, environment });
}
