import { randomUUID } from "node:crypto";
import type Database from "better-sqlite3";
import type {
  AgentContentBlock,
  AgentConversation,
  AgentEvent,
  AgentManagerDeps,
  AgentMessage,
  AgentProvider,
  AgentSettings,
  AgentUser,
} from "./types";
import { executeTool, toolsForUser } from "./tools";
import { getProviderClient } from "./providers";
import { Errors } from "../api/errors";
import { assertLiteralPublicTarget } from "../utils/outboundTargets";

const DEFAULT_MAX_TOKENS = 4096;
const MAX_TURNS_PER_RUN = 8; // Hard cap on tool-call → response loops to avoid runaway.
const MAX_AGENT_MESSAGE_CHARS = 16_000;
const VALID_PROVIDERS: AgentProvider[] = ["anthropic", "openai", "openai-compatible"];

export interface AgentManagerOptions {
  deps: AgentManagerDeps;
  enabled?: boolean;
}

export interface SendOptions {
  user: AgentUser;
  conversationId: string;
  text: string;
  onEvent: (event: AgentEvent) => void;
  signal: AbortSignal;
}

export class AgentManager {
  private readonly active = new Map<string, AbortController>();

  constructor(private readonly db: Database.Database, private readonly opts: AgentManagerOptions) {}

  isEnabled(): boolean {
    return this.opts.enabled ?? false;
  }

  // ---------- Settings ----------

  getSettings(userId: string): AgentSettings | null {
    const row = this.db
      .prepare("SELECT * FROM agent_settings WHERE user_id = ?")
      .get(userId) as
      | {
          user_id: string;
          provider: AgentProvider;
          model: string;
          api_key: string;
          base_url: string | null;
          enabled: number;
          created_at: number;
          updated_at: number;
        }
      | undefined;
    return row ? mapSettingsRow(row) : null;
  }

  saveSettings(userId: string, input: {
    provider: AgentProvider;
    model: string;
    apiKey: string;
    baseUrl?: string;
    enabled?: boolean;
  }): AgentSettings {
    if (!VALID_PROVIDERS.includes(input.provider)) {
      throw new Error(`provider must be one of ${VALID_PROVIDERS.join(", ")}`);
    }
    if (!input.model.trim()) throw new Error("model is required");
    if (!input.apiKey.trim()) throw new Error("apiKey is required");
    if (input.provider === "openai-compatible" && !input.baseUrl?.trim()) {
      throw new Error("baseUrl is required for openai-compatible provider");
    }
    const baseUrl = normalizeAgentBaseUrl(input.baseUrl);
    const now = Date.now();
    const existing = this.getSettings(userId);
    const enabled = input.enabled ?? existing?.enabled ?? true;

    if (existing) {
      this.db
        .prepare(
          `UPDATE agent_settings SET provider = ?, model = ?, api_key = ?, base_url = ?, enabled = ?, updated_at = ? WHERE user_id = ?`,
        )
        .run(input.provider, input.model.trim(), input.apiKey, baseUrl ?? null, enabled ? 1 : 0, now, userId);
    } else {
      this.db
        .prepare(
          `INSERT INTO agent_settings (user_id, provider, model, api_key, base_url, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
        )
        .run(userId, input.provider, input.model.trim(), input.apiKey, baseUrl ?? null, enabled ? 1 : 0, now, now);
    }
    return this.getSettings(userId)!;
  }

  deleteSettings(userId: string): void {
    this.db.prepare("DELETE FROM agent_settings WHERE user_id = ?").run(userId);
  }

  // ---------- Conversations ----------

  listConversations(userId: string): AgentConversation[] {
    const rows = this.db
      .prepare("SELECT * FROM agent_conversations WHERE user_id = ? ORDER BY updated_at DESC")
      .all(userId) as ConversationRow[];
    return rows.map(mapConversationRow);
  }

  getConversation(userId: string, conversationId: string): AgentConversation | null {
    const row = this.db
      .prepare("SELECT * FROM agent_conversations WHERE id = ? AND user_id = ?")
      .get(conversationId, userId) as ConversationRow | undefined;
    return row ? mapConversationRow(row) : null;
  }

  createConversation(userId: string, settings: AgentSettings, title?: string): AgentConversation {
    const id = randomUUID();
    const now = Date.now();
    this.db
      .prepare(
        `INSERT INTO agent_conversations (id, user_id, title, provider, model, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)`,
      )
      .run(id, userId, title?.trim() || null, settings.provider, settings.model, now, now);
    return {
      id,
      userId,
      title: title?.trim() || undefined,
      provider: settings.provider,
      model: settings.model,
      createdAt: now,
      updatedAt: now,
    };
  }

  deleteConversation(userId: string, conversationId: string): void {
    this.db
      .prepare("DELETE FROM agent_conversations WHERE id = ? AND user_id = ?")
      .run(conversationId, userId);
  }

  listMessages(conversationId: string): AgentMessage[] {
    // Order by rowid so the sequence is the actual insertion order, not the
    // random-UUID ordering you'd otherwise get when two messages land in the
    // same millisecond (common during a single turn).
    const rows = this.db
      .prepare("SELECT * FROM agent_messages WHERE conversation_id = ? ORDER BY rowid ASC")
      .all(conversationId) as MessageRow[];
    return rows.map(mapMessageRow);
  }

  // ---------- Cancel ----------

  cancel(conversationId: string): boolean {
    const ctl = this.active.get(conversationId);
    if (!ctl) return false;
    ctl.abort();
    this.active.delete(conversationId);
    return true;
  }

  // ---------- Send ----------

  /**
   * Run a single user turn: append the user message, call the provider, execute
   * any requested tools, and continue the loop until the model stops or the
   * turn cap is hit. Streams events through `onEvent`.
   */
  async send(options: SendOptions): Promise<void> {
    if (!this.isEnabled()) {
      throw new Error("Hermes agent is disabled. Set NEONEXUS_ENABLE_HERMES_AGENT=true to enable.");
    }
    const conversation = this.getConversation(options.user.id, options.conversationId);
    if (!conversation) throw new Error("Conversation not found");

    const settings = this.getSettings(options.user.id);
    if (!settings || !settings.enabled) throw new Error("Agent settings not configured");
    const userText = normalizeUserText(options.text);

    const provider = getProviderClient(settings.provider);
    const tools = toolsForUser(options.user.role);

    // Persist user message.
    const userMsg: AgentMessage = {
      id: randomUUID(),
      conversationId: conversation.id,
      role: "user",
      content: [{ type: "text", text: userText }],
      createdAt: Date.now(),
    };
    this.persistMessage(userMsg);
    options.onEvent({ type: "message_start", messageId: userMsg.id, role: "user" });
    options.onEvent({ type: "delta", messageId: userMsg.id, text: userText });
    options.onEvent({ type: "message_end", messageId: userMsg.id });

    // Wire up cancellation.
    const ctl = new AbortController();
    options.signal.addEventListener("abort", () => ctl.abort(), { once: true });
    this.active.set(conversation.id, ctl);

    try {
      let history = this.listMessages(conversation.id);
      for (let turn = 0; turn < MAX_TURNS_PER_RUN; turn += 1) {
        const assistantMsgId = randomUUID();
        const assistantContent: AgentContentBlock[] = [];
        const pendingToolUses: { id: string; name: string; input: Record<string, unknown> }[] = [];

        options.onEvent({ type: "message_start", messageId: assistantMsgId, role: "assistant" });

        try {
          for await (const ev of provider.stream(
            {
              apiKey: settings.apiKey,
              baseUrl: settings.baseUrl,
              model: settings.model,
              systemPrompt: buildSystemPrompt(options.user, conversation),
              messages: history,
              tools,
              maxTokens: DEFAULT_MAX_TOKENS,
            },
            ctl.signal,
          )) {
            if (ev.type === "text_delta") {
              if (assistantContent.at(-1)?.type === "text") {
                (assistantContent.at(-1) as { type: "text"; text: string }).text += ev.text;
              } else {
                assistantContent.push({ type: "text", text: ev.text });
              }
              options.onEvent({ type: "delta", messageId: assistantMsgId, text: ev.text });
            } else if (ev.type === "tool_use") {
              assistantContent.push({ type: "tool_use", id: ev.id, name: ev.name, input: ev.input });
              pendingToolUses.push({ id: ev.id, name: ev.name, input: ev.input });
              options.onEvent({ type: "tool_use", messageId: assistantMsgId, toolUseId: ev.id, name: ev.name, input: ev.input });
            } else if (ev.type === "stop") {
              // Continue draining; no special action needed here.
            }
          }
        } catch (error) {
          options.onEvent({
            type: "error",
            conversationId: conversation.id,
            error: error instanceof Error ? error.message : String(error),
          });
          options.onEvent({ type: "message_end", messageId: assistantMsgId });
          return;
        }

        const assistantMsg: AgentMessage = {
          id: assistantMsgId,
          conversationId: conversation.id,
          role: "assistant",
          content: assistantContent,
          createdAt: Date.now(),
        };
        this.persistMessage(assistantMsg);
        options.onEvent({ type: "message_end", messageId: assistantMsgId });

        if (pendingToolUses.length === 0) {
          this.touchConversation(conversation.id);
          options.onEvent({ type: "complete", conversationId: conversation.id });
          return;
        }

        // Execute tools sequentially. Sequential keeps the audit-log order stable
        // and avoids interleaved start/stop calls hitting the same node.
        const toolMsgId = randomUUID();
        const toolResultBlocks: AgentContentBlock[] = [];
        options.onEvent({ type: "message_start", messageId: toolMsgId, role: "tool" });
        for (const call of pendingToolUses) {
          if (ctl.signal.aborted) break;
          const result = await this.runTool(call, options.user);
          toolResultBlocks.push(result.block);
          options.onEvent({
            type: "tool_result",
            messageId: toolMsgId,
            toolUseId: call.id,
            output: result.block.type === "tool_result" ? result.block.output : "",
            isError: result.block.type === "tool_result" ? result.block.isError : undefined,
          });
        }
        const toolMsg: AgentMessage = {
          id: toolMsgId,
          conversationId: conversation.id,
          role: "tool",
          content: toolResultBlocks,
          createdAt: Date.now(),
        };
        this.persistMessage(toolMsg);
        options.onEvent({ type: "message_end", messageId: toolMsgId });

        history = this.listMessages(conversation.id);
      }
      // Hit the turn cap without the model stopping.
      options.onEvent({
        type: "error",
        conversationId: conversation.id,
        error: `Conversation exceeded ${MAX_TURNS_PER_RUN} tool-call iterations and was halted.`,
      });
    } finally {
      this.active.delete(conversation.id);
      this.touchConversation(conversation.id);
    }
  }

  // ---------- Helpers ----------

  private persistMessage(msg: AgentMessage): void {
    this.db
      .prepare(
        "INSERT INTO agent_messages (id, conversation_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
      )
      .run(msg.id, msg.conversationId, msg.role, JSON.stringify(msg.content), msg.createdAt);
  }

  private touchConversation(conversationId: string): void {
    this.db
      .prepare("UPDATE agent_conversations SET updated_at = ? WHERE id = ?")
      .run(Date.now(), conversationId);
  }

  private async runTool(
    call: { id: string; name: string; input: Record<string, unknown> },
    user: AgentUser,
  ): Promise<{ block: AgentContentBlock }> {
    try {
      const output = await executeTool(call.name, call.input, { user, deps: this.opts.deps });
      const text = stringifyToolOutput(output);
      return { block: { type: "tool_result", tool_use_id: call.id, output: text, isError: false } };
    } catch (error) {
      const text = error instanceof Error ? error.message : String(error);
      return { block: { type: "tool_result", tool_use_id: call.id, output: text, isError: true } };
    }
  }
}

function buildSystemPrompt(user: AgentUser, conversation: AgentConversation): string {
  return [
    "You are the Hermes operations agent for NeoNexus, a self-hosted Neo N3 node management platform.",
    `You are talking to ${user.username} (role: ${user.role}). The conversation id is ${conversation.id}.`,
    "You have tools to inspect and operate the node fleet. Prefer concise, direct answers; show structured data only when it helps the user reason about the fleet.",
    user.role === "viewer"
      ? "This user has the viewer role — only read-only tools are available. Do not promise actions you cannot perform."
      : "Before you call any tool that changes state (start/stop/restart node, toggle plugin), briefly state what you're about to do in plain language so the user can interrupt.",
    "When a tool returns an error, surface the error verbatim and stop unless the user asks you to retry.",
    "Never invent node ids, plugin names, or metric values — always call the relevant tool first.",
  ].join("\n");
}

function stringifyToolOutput(output: unknown): string {
  if (typeof output === "string") return output;
  try {
    return JSON.stringify(output, null, 2);
  } catch {
    return String(output);
  }
}

function normalizeAgentBaseUrl(rawBaseUrl?: string): string | undefined {
  const value = rawBaseUrl?.trim();
  if (!value) {
    return undefined;
  }

  let parsed: URL;
  try {
    parsed = new URL(value);
  } catch {
    throw Errors.agentProviderUrlInvalid();
  }

  if (!["http:", "https:"].includes(parsed.protocol)) {
    throw Errors.agentProviderUrlInvalid("Base URL must use http or https");
  }
  if (parsed.search || parsed.hash) {
    throw Errors.agentProviderUrlInvalid("Base URL cannot include query or fragment parameters");
  }
  if (parsed.protocol !== "https:" && process.env.NEONEXUS_ALLOW_INSECURE_AGENT_PROVIDER_URLS !== "true") {
    throw Errors.agentProviderUrlInsecure();
  }

  assertLiteralPublicTarget(
    parsed.hostname,
    Errors.agentProviderUrlPrivateTarget,
    process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS === "true",
  );

  const path = parsed.pathname === "/" ? "" : parsed.pathname.replace(/\/+$/, "");
  return `${parsed.origin}${path}`;
}

function normalizeUserText(text: string): string {
  const value = text.trim();
  if (!value) {
    throw Errors.agentMessageInvalid("text is required");
  }
  if (value.length > MAX_AGENT_MESSAGE_CHARS) {
    throw Errors.agentMessageInvalid(`text must be ${MAX_AGENT_MESSAGE_CHARS} characters or fewer`);
  }
  return value;
}

interface ConversationRow {
  id: string;
  user_id: string;
  title: string | null;
  provider: AgentProvider;
  model: string;
  created_at: number;
  updated_at: number;
}

interface MessageRow {
  id: string;
  conversation_id: string;
  role: AgentMessage["role"];
  content: string;
  created_at: number;
}

function mapSettingsRow(row: {
  user_id: string;
  provider: AgentProvider;
  model: string;
  api_key: string;
  base_url: string | null;
  enabled: number;
  created_at: number;
  updated_at: number;
}): AgentSettings {
  return {
    userId: row.user_id,
    provider: row.provider,
    model: row.model,
    apiKey: row.api_key,
    baseUrl: row.base_url ?? undefined,
    enabled: row.enabled === 1,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapConversationRow(row: ConversationRow): AgentConversation {
  return {
    id: row.id,
    userId: row.user_id,
    title: row.title ?? undefined,
    provider: row.provider,
    model: row.model,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapMessageRow(row: MessageRow): AgentMessage {
  let content: AgentContentBlock[] = [];
  try {
    const parsed = JSON.parse(row.content) as unknown;
    if (Array.isArray(parsed)) content = parsed as AgentContentBlock[];
  } catch {
    content = [];
  }
  return {
    id: row.id,
    conversationId: row.conversation_id,
    role: row.role,
    content,
    createdAt: row.created_at,
  };
}
