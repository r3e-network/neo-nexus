import type {
  AgentContentBlock,
  AgentMessage,
  ProviderClient,
  ProviderRequest,
  ProviderStreamEvent,
} from "../types";
import { pinnedStreamingRequest, sseEvents } from "./pinnedStream";

const DEFAULT_BASE_URL = "https://api.anthropic.com";
const ANTHROPIC_VERSION = "2023-06-01";

interface AnthropicTextDelta {
  type: "content_block_delta";
  index: number;
  delta: { type: "text_delta"; text: string } | { type: "input_json_delta"; partial_json: string };
}

interface AnthropicContentBlockStart {
  type: "content_block_start";
  index: number;
  content_block:
    | { type: "text"; text: string }
    | { type: "tool_use"; id: string; name: string; input: Record<string, unknown> };
}

interface AnthropicMessageDelta {
  type: "message_delta";
  delta: { stop_reason?: string };
}

interface AnthropicError {
  type: "error";
  error: { type: string; message: string };
}

type AnthropicEvent =
  | AnthropicTextDelta
  | AnthropicContentBlockStart
  | AnthropicMessageDelta
  | { type: "content_block_stop"; index: number }
  | { type: "message_start"; message: unknown }
  | { type: "message_stop" }
  | { type: "ping" }
  | AnthropicError;

export const anthropicClient: ProviderClient = {
  async *stream(req: ProviderRequest, signal: AbortSignal): AsyncGenerator<ProviderStreamEvent, void, void> {
    const baseUrl = (req.baseUrl?.replace(/\/$/, "") || DEFAULT_BASE_URL);
    const url = `${baseUrl}/v1/messages`;

    const body = {
      model: req.model,
      max_tokens: req.maxTokens,
      system: req.systemPrompt,
      messages: req.messages.map(toAnthropicMessage).filter((m) => m.content.length > 0),
      tools: req.tools.map((t) => ({
        name: t.name,
        description: t.description,
        input_schema: t.inputSchema,
      })),
      stream: true,
    };

    const res = await pinnedStreamingRequest(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-api-key": req.apiKey,
        "anthropic-version": ANTHROPIC_VERSION,
        Accept: "text/event-stream",
      },
      body: JSON.stringify(body),
      signal,
    });

    if ((res.statusCode ?? 0) >= 400) {
      const errBody = await readToString(res);
      throw new Error(`Anthropic ${res.statusCode}: ${errBody.slice(0, 500)}`);
    }

    // Track partial tool_use blocks: index -> { id, name, jsonBuffer }
    const toolBuilders = new Map<number, { id: string; name: string; jsonBuffer: string }>();

    for await (const data of sseEvents(res, signal)) {
      if (data === "[DONE]") return;
      let event: AnthropicEvent;
      try {
        event = JSON.parse(data) as AnthropicEvent;
      } catch {
        continue;
      }

      if (event.type === "content_block_start") {
        const block = event.content_block;
        if (block.type === "tool_use") {
          toolBuilders.set(event.index, { id: block.id, name: block.name, jsonBuffer: "" });
        }
      } else if (event.type === "content_block_delta") {
        if (event.delta.type === "text_delta") {
          yield { type: "text_delta", text: event.delta.text };
        } else if (event.delta.type === "input_json_delta") {
          const builder = toolBuilders.get(event.index);
          if (builder) builder.jsonBuffer += event.delta.partial_json;
        }
      } else if (event.type === "content_block_stop") {
        const builder = toolBuilders.get(event.index);
        if (builder) {
          let parsedInput: Record<string, unknown> = {};
          try {
            parsedInput = builder.jsonBuffer ? (JSON.parse(builder.jsonBuffer) as Record<string, unknown>) : {};
          } catch {
            // partial json may be empty for tools with no inputs.
          }
          yield { type: "tool_use", id: builder.id, name: builder.name, input: parsedInput };
          toolBuilders.delete(event.index);
        }
      } else if (event.type === "message_delta") {
        if (event.delta.stop_reason) {
          yield { type: "stop", reason: event.delta.stop_reason };
        }
      } else if (event.type === "error") {
        throw new Error(`Anthropic stream error: ${event.error.message}`);
      } else if (event.type === "message_stop") {
        yield { type: "stop" };
      }
    }
  },
};

function toAnthropicMessage(msg: AgentMessage): { role: "user" | "assistant"; content: unknown[] } {
  // Anthropic only accepts user|assistant for role; "tool" results are user messages
  // with tool_result content blocks.
  if (msg.role === "system") {
    return { role: "user", content: textBlocks(msg.content) };
  }
  if (msg.role === "tool") {
    return {
      role: "user",
      content: msg.content
        .filter((c): c is Extract<AgentContentBlock, { type: "tool_result" }> => c.type === "tool_result")
        .map((c) => ({
          type: "tool_result",
          tool_use_id: c.tool_use_id,
          content: c.output,
          is_error: c.isError ?? false,
        })),
    };
  }

  return {
    role: msg.role,
    content: msg.content
      .map((c) => {
        if (c.type === "text") return { type: "text", text: c.text };
        if (c.type === "tool_use") return { type: "tool_use", id: c.id, name: c.name, input: c.input };
        return null;
      })
      .filter((c): c is NonNullable<typeof c> => c !== null),
  };
}

function textBlocks(content: AgentContentBlock[]): { type: "text"; text: string }[] {
  return content.flatMap((c) => (c.type === "text" ? [{ type: "text" as const, text: c.text }] : []));
}

async function readToString(stream: NodeJS.ReadableStream): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of stream) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk as string));
  }
  return Buffer.concat(chunks).toString("utf8");
}
