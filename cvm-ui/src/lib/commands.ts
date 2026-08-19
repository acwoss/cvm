import { invoke } from "@tauri-apps/api/core";

export type EnvVarSource = "dotenv" | "settings";

export interface EnvVarSummary {
  key: string;
  source: EnvVarSource;
}

export interface ConfigSection {
  allowedTools: string[];
  deniedTools: string[];
  description: string | null;
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

export type ItemSource = { kind: "native" } | { kind: "plugin"; marketplace: string; plugin: string };

export interface McpServerInfo {
  name: string;
  command: string;
  args: string[];
  envKeys: string[];
  source: ItemSource;
}

export interface SkillOrAgentInfo {
  id: string;
  name: string;
  description: string;
  builtIn: boolean;
  source: ItemSource;
}

export type AuthMethod = "oauth" | "apiKey";

export interface AccountInfo {
  authMethod: AuthMethod;
  email: string | null;
  displayName: string | null;
  organizationName: string | null;
  seatTier: string | null;
}

export interface EnvironmentSummary {
  name: string;
  path: string;
  active: boolean;
  description: string | null;
  pluginCount: number;
  skillsAgentsCount: number;
  account: AccountInfo | null;
}

export function listEnvironments(): Promise<EnvironmentSummary[]> {
  return invoke("list_environments");
}

export interface AuthStatus {
  loggedIn: boolean;
  authMethod: string;
  apiProvider: string | null;
  email: string | null;
  orgId: string | null;
  orgName: string | null;
  subscriptionType: string | null;
  apiKeySource: string | null;
}

export interface EnvironmentDetail {
  name: string;
  path: string;
  active: boolean;
  config: ConfigSection;
  envVars: EnvVarSummary[];
  marketplaces: MarketplaceInfo[];
  mcpServers: McpServerInfo[];
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

export function openInClaude(name: string, directory?: string): Promise<void> {
  return invoke("open_in_claude", { name, directory: directory ?? null });
}

export function loginInClaude(name: string): Promise<void> {
  return invoke("login_in_claude", { name });
}

export function checkAuthStatus(name: string): Promise<AuthStatus> {
  return invoke("check_auth_status", { name });
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
  description: string | null,
): Promise<void> {
  return invoke("write_config_section", { name, allowedTools, deniedTools, description });
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

export function getClaudeMd(envName: string): Promise<string> {
  return invoke("get_claude_md", { envName });
}

export function writeClaudeMd(envName: string, content: string): Promise<void> {
  return invoke("write_claude_md", { envName, content });
}

export interface SkillContent {
  name: string;
  description: string;
  body: string;
}

export function getSkillContent(envName: string, id: string): Promise<SkillContent> {
  return invoke("get_skill_content", { envName, id });
}

export function writeSkillContent(envName: string, id: string, content: SkillContent): Promise<void> {
  return invoke("write_skill_content", { envName, id, content });
}

export function createSkill(
  envName: string,
  id: string,
  name: string,
  description: string,
): Promise<void> {
  return invoke("create_skill", { envName, id, name, description });
}

export function deleteSkill(envName: string, id: string): Promise<void> {
  return invoke("delete_skill", { envName, id });
}

export function getAgentContent(envName: string, id: string): Promise<SkillContent> {
  return invoke("get_agent_content", { envName, id });
}

export function writeAgentContent(envName: string, id: string, content: SkillContent): Promise<void> {
  return invoke("write_agent_content", { envName, id, content });
}

export function createAgent(
  envName: string,
  id: string,
  name: string,
  description: string,
): Promise<void> {
  return invoke("create_agent", { envName, id, name, description });
}

export function deleteAgent(envName: string, id: string): Promise<void> {
  return invoke("delete_agent", { envName, id });
}

export interface HookSummary {
  event: string;
  configured: boolean;
  enabled: boolean;
  preview: string | null;
}

export function listHooks(): Promise<HookSummary[]> {
  return invoke("list_hooks");
}

export function getHook(event: string): Promise<string | null> {
  return invoke("get_hook", { event });
}

export function writeHook(event: string, content: string): Promise<void> {
  return invoke("write_hook", { event, content });
}

export function setHookEnabled(event: string, enabled: boolean): Promise<void> {
  return invoke("set_hook_enabled", { event, enabled });
}

export function deleteHook(event: string): Promise<void> {
  return invoke("delete_hook", { event });
}

export interface UpdateInfo {
  current: string;
  latest: string;
}

export function checkUiUpdate(): Promise<UpdateInfo | null> {
  return invoke("check_ui_update");
}

export function applyUiUpdate(): Promise<void> {
  return invoke("apply_ui_update");
}
