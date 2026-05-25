export type KeyType =
  | "ApiKey"
  | "BearerToken"
  | "BasicAuth"
  | "OAuth2"
  | "Jwt"
  | "SshKey"
  | "Other";

export type Environment =
  | "Production"
  | "Staging"
  | "Development"
  | "Testing"
  | "Other";

export interface KeyEntry {
  id: string;
  name: string;
  provider: string;
  key_type: KeyType;
  description: string | null;
  tags: string[];
  environment: Environment;
  group_id: string | null;
  created_at: string;
  updated_at: string;
  expires_at: string | null;
  last_used_at: string | null;
  usage_count: number;
  version: number;
}

export interface ConnectivityResult {
  success: boolean;
  message: string;
  latency_ms: number | null;
}
