import type { NodeManager } from "../core/NodeManager";
import type { RemoteServerManager } from "../core/RemoteServerManager";
import type { IntegrationManager } from "../integrations/IntegrationManager";
import type { MetricsCollector } from "../monitoring/MetricsCollector";
import type { NetworkHeightTracker } from "../core/NetworkHeightTracker";
import type { AuditLogger } from "../core/AuditLogger";

export type AgentProvider = "anthropic" | "openai" | "openai-compatible";

export interface AgentSettings {
  userId: string;
  provider: AgentProvider;
  model: string;
  apiKey: string;
  baseUrl?: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface AgentConversation {
  id: string;
  userId: string;
  title?: string;
  provider: AgentProvider;
  model: string;
  createdAt: number;
  updatedAt: number;
}

export type AgentRole = "system" | "user" | "assistant" | "tool";

export type AgentContentBlock =
  | { type: "text"; text: string }
  | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
  | { type: "tool_result"; tool_use_id: string; output: string; isError?: boolean };

export interface AgentMessage {
  id: string;
  conversationId: string;
  role: AgentRole;
  content: AgentContentBlock[];
  createdAt: number;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: "object";
    properties: Record<string, unknown>;
    required?: string[];
  };
  requiresAdmin: boolean;
  execute: (input: Record<string, unknown>, ctx: ToolContext) => Promise<unknown>;
}

export interface AgentUser {
  id: string;
  username: string;
  role: "admin" | "viewer";
}

export interface AgentManagerDeps {
  nodeManager: NodeManager;
  remoteServerManager: RemoteServerManager;
  integrationManager: IntegrationManager;
  metricsCollector: MetricsCollector;
  networkHeightTracker: NetworkHeightTracker;
  auditLogger: AuditLogger;
}

export interface ToolContext {
  user: AgentUser;
  deps: AgentManagerDeps;
}

export type ProviderStreamEvent =
  | { type: "text_delta"; text: string }
  | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
  | { type: "stop"; reason?: string };

export interface ProviderRequest {
  apiKey: string;
  baseUrl?: string;
  model: string;
  systemPrompt: string;
  messages: AgentMessage[];
  tools: ToolDefinition[];
  maxTokens: number;
}

export interface ProviderClient {
  stream(req: ProviderRequest, signal: AbortSignal): AsyncGenerator<ProviderStreamEvent, void, void>;
}

export type AgentEvent =
  | { type: "message_start"; messageId: string; role: AgentRole }
  | { type: "delta"; messageId: string; text: string }
  | { type: "tool_use"; messageId: string; toolUseId: string; name: string; input: Record<string, unknown> }
  | { type: "tool_result"; messageId: string; toolUseId: string; output: string; isError?: boolean }
  | { type: "message_end"; messageId: string }
  | { type: "complete"; conversationId: string }
  | { type: "error"; conversationId: string; error: string };
