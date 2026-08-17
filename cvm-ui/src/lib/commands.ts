import { invoke } from "@tauri-apps/api/core";

export interface EnvironmentSummary {
  name: string;
  path: string;
  active: boolean;
}

export function listEnvironments(): Promise<EnvironmentSummary[]> {
  return invoke("list_environments");
}

export type EnvVarSource = "dotenv" | "settings";

export interface EnvVarSummary {
  key: string;
  source: EnvVarSource;
}

export interface ConfigSection {
  allowedTools: string[];
  deniedTools: string[];
  other: unknown;
}

export interface PluginInfo {
  name: string;
  description: string | null;
  enabled: boolean;
  installed: boolean;
  version: string | null;
}

export interface MarketplaceInfo {
  id: string;
  repo: string | null;
  plugins: PluginInfo[];
}

export interface SkillOrAgentInfo {
  id: string;
  name: string;
  description: string;
  builtIn: boolean;
}

export interface AccountInfo {
  email: string | null;
  displayName: string | null;
  organizationName: string | null;
  seatTier: string | null;
}

export interface EnvironmentDetail {
  name: string;
  path: string;
  active: boolean;
  config: ConfigSection;
  envVars: EnvVarSummary[];
  marketplaces: MarketplaceInfo[];
  skills: SkillOrAgentInfo[];
  agents: SkillOrAgentInfo[];
  account: AccountInfo | null;
  warnings: string[];
}

export function getEnvironmentDetail(name: string): Promise<EnvironmentDetail> {
  return invoke("get_environment_detail", { name });
}

export function revealEnvVar(name: string, source: EnvVarSource, key: string): Promise<string> {
  return invoke("reveal_env_var", { name, source, key });
}

export function openInClaude(name: string): Promise<void> {
  return invoke("open_in_claude", { name });
}

export function commandErrorMessage(err: unknown): string {
  if (
    err &&
    typeof err === "object" &&
    "message" in err &&
    typeof (err as { message: unknown }).message === "string"
  ) {
    return (err as { message: string }).message;
  }
  return String(err);
}

export function createEnvironment(
  name: string,
  anonymous: boolean,
  inherit: boolean,
  open: boolean,
): Promise<void> {
  return invoke("create_environment", { name, anonymous, inherit, open });
}

export function removeEnvironment(name: string): Promise<void> {
  return invoke("remove_environment", { name });
}

export function writeConfigSection(
  name: string,
  allowedTools: string[],
  deniedTools: string[],
): Promise<void> {
  return invoke("write_config_section", { name, allowedTools, deniedTools });
}

export function writeEnvVar(
  name: string,
  source: EnvVarSource,
  key: string,
  value: string,
): Promise<void> {
  return invoke("write_env_var", { name, source, key, value });
}

export function removeEnvVar(name: string, source: EnvVarSource, key: string): Promise<void> {
  return invoke("remove_env_var", { name, source, key });
}

export function addMarketplace(name: string, source: string): Promise<string> {
  return invoke("add_marketplace", { name, source });
}

export function removeMarketplace(name: string, marketplace: string): Promise<string> {
  return invoke("remove_marketplace", { name, marketplace });
}

export function installPlugin(name: string, plugin: string): Promise<string> {
  return invoke("install_plugin", { name, plugin });
}

export function uninstallPlugin(name: string, plugin: string): Promise<string> {
  return invoke("uninstall_plugin", { name, plugin });
}

export function enablePlugin(name: string, plugin: string): Promise<string> {
  return invoke("enable_plugin", { name, plugin });
}

export function disablePlugin(name: string, plugin: string): Promise<string> {
  return invoke("disable_plugin", { name, plugin });
}
