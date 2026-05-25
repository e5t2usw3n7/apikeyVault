import { invoke } from "@tauri-apps/api/core";
import type { Group } from "@/types";

export async function listGroups(): Promise<Group[]> {
  return invoke<Group[]>("list_groups");
}

export async function createGroup(name: string): Promise<Group> {
  return invoke<Group>("create_group", { name });
}

export async function updateGroup(
  id: string,
  name?: string,
  description?: string
): Promise<void> {
  return invoke("update_group", { id, name, description });
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke("delete_group", { id });
}
