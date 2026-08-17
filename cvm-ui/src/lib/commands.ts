import { invoke } from "@tauri-apps/api/core";

export interface EnvironmentSummary {
  name: string;
  path: string;
  active: boolean;
}

export function listEnvironments(): Promise<EnvironmentSummary[]> {
  return invoke("list_environments");
}
