import { invoke } from "@tauri-apps/api/core";
import type { KeyEntry, ConnectivityResult } from "@/types";

export interface AddKeyParams {
  name: string;
  provider: string;
  key_type: string;
  value: string;
  environment: string;
  description?: string;
  group_id?: string;
  tags: string[];
}

export interface UpdateKeyParams {
  name: string;
  environment: string;
  value?: string;
  description?: string;
  tags?: string[];
}

export async function listKeys(): Promise<KeyEntry[]> {
  return invoke<KeyEntry[]>("list_keys");
}

export async function searchKeys(query: string): Promise<KeyEntry[]> {
  return invoke<KeyEntry[]>("search_keys", { query });
}

export async function getKeyValue(
  name: string,
  environment: string
): Promise<string> {
  return invoke<string>("get_key_value", { name, environment });
}

export async function addKey(params: AddKeyParams): Promise<KeyEntry> {
  return invoke<KeyEntry>("add_key", params);
}

export async function updateKey(params: UpdateKeyParams): Promise<KeyEntry> {
  return invoke<KeyEntry>("update_key", params);
}

export async function deleteKey(
  name: string,
  environment: string
): Promise<void> {
  return invoke("delete_key", { name, environment });
}

export async function renameKey(
  oldName: string,
  environment: string,
  newName: string
): Promise<KeyEntry> {
  return invoke<KeyEntry>("rename_key", { oldName, environment, newName });
}

export async function rotateKey(
  name: string,
  environment: string,
  newValue: string
): Promise<KeyEntry> {
  return invoke<KeyEntry>("rotate_key", { name, environment, newValue });
}

export async function testConnectivity(
  name: string,
  environment: string
): Promise<ConnectivityResult> {
  return invoke<ConnectivityResult>("test_connectivity", { name, environment });
}
