import type {
  AgentContentBlock,
  AgentMessage,
  ProviderClient,
  ProviderRequest,
  ProviderStreamEvent,
} from "../types";
import { pinnedStreamingRequest, sseEvents } from "./pinnedStream";

const DEFAULT_BASE_URL = "https://api.openai.com";

interface OpenAIToolCallDelta {
  index: number;
  id?: string;
  type?: "function";
  function?: { name?: string; arguments?: string };
}

interface OpenAIChoiceDelta {
  delta?: {
    role?: string;
    content?: string | null;
    tool_calls?: OpenAIToolCallDelta[];
  };
  finish_reason?: string | null;
}

interface OpenAIStreamChunk {
  choices?: OpenAIChoiceDelta[];
  error?: { message: string };
}

export const openaiClient: ProviderClient = {
  async *stream(req: ProviderRequest, signal: AbortSignal): AsyncGenerator<ProviderStreamEvent, void, void> {
    const baseUrl = (req.baseUrl?.replace(/\/$/, "") || DEFAULT_BASE_URL);
    const url = `${baseUrl}/v1/chat/completions`;

    const messages = [
      { role: "system" as const, content: req.systemPrompt },
      ...req.messages.flatMap(toOpenAIMessages),
    ];

    const body = {
      model: req.model,
      messages,
      tools: req.tools.map((t) => ({
        type: "function" as const,
        function: {
          name: t.name,
          description: t.description,
          parameters: t.inputSchema,
        },
      })),
      stream: true,
      max_tokens: req.maxTokens,
    };

    const res = await pinnedStreamingRequest(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${req.apiKey}`,
        Accept: "text/event-stream",
      },
      body: JSON.stringify(body),
      signal,
    });

    if ((res.statusCode ?? 0) >= 400) {
      const errBody = await readToString(res);
      throw new Error(`OpenAI ${res.statusCode}: ${errBody.slice(0, 500)}`);
    }

    // Track partial tool calls per index.
    const toolBuilders = new Map<number, { id: string; name: string; jsonBuffer: string; emitted: boolean }>();

    for await (const data of sseEvents(res, signal)) {
      if (data === "[DONE]") {
        // Flush any tool_use that hasn't been emitted because no finish_reason marker.
        for (const [, b] of toolBuilders) {
          if (!b.emitted) {
            yield emitToolUse(b);
            b.emitted = true;
          }
        }
        return;
      }
      let chunk: OpenAIStreamChunk;
      try {
        chunk = JSON.parse(data) as OpenAIStreamChunk;
      } catch {
        continue;
      }

      if (chunk.error) {
        throw new Error(`OpenAI stream error: ${chunk.error.message}`);
      }

      const choice = chunk.choices?.[0];
      if (!choice) continue;

      if (choice.delta?.content) {
        yield { type: "text_delta", text: choice.delta.content };
      }

      if (choice.delta?.tool_calls) {
        for (const tc of choice.delta.tool_calls) {
          let builder = toolBuilders.get(tc.index);
          if (!builder) {
            builder = { id: tc.id ?? `call_${tc.index}`, name: tc.function?.name ?? "", jsonBuffer: "", emitted: false };
            toolBuilders.set(tc.index, builder);
          }
          if (tc.id) builder.id = tc.id;
          if (tc.function?.name) builder.name = tc.function.name;
          if (tc.function?.arguments) builder.jsonBuffer += tc.function.arguments;
        }
      }

      if (choice.finish_reason === "tool_calls") {
        for (const [, b] of toolBuilders) {
          if (!b.emitted) {
            yield emitToolUse(b);
            b.emitted = true;
          }
        }
      } else if (choice.finish_reason) {
        yield { type: "stop", reason: choice.finish_reason };
      }
    }
  },
};

function emitToolUse(b: { id: string; name: string; jsonBuffer: string }): ProviderStreamEvent {
  let input: Record<string, unknown> = {};
  if (b.jsonBuffer) {
    try {
      input = JSON.parse(b.jsonBuffer) as Record<string, unknown>;
    } catch {
      // Treat as empty input on malformed JSON; the model will see the tool error and retry.
    }
  }
  return { type: "tool_use", id: b.id, name: b.name, input };
}

interface OpenAIMessage {
  role: "system" | "user" | "assistant" | "tool";
  content: string | null;
  tool_calls?: { id: string; type: "function"; function: { name: string; arguments: string } }[];
  tool_call_id?: string;
}

function toOpenAIMessages(msg: AgentMessage): OpenAIMessage[] {
  if (msg.role === "system") {
    return [{ role: "system", content: collectText(msg.content) }];
  }
  if (msg.role === "user") {
    return [{ role: "user", content: collectText(msg.content) }];
  }
  if (msg.role === "assistant") {
    const toolCalls = msg.content.filter((c): c is Extract<AgentContentBlock, { type: "tool_use" }> => c.type === "tool_use");
    const text = collectText(msg.content);
    const out: OpenAIMessage = {
      role: "assistant",
      content: text || null,
    };
    if (toolCalls.length > 0) {
      out.tool_calls = toolCalls.map((tc) => ({
        id: tc.id,
        type: "function",
        function: { name: tc.name, arguments: JSON.stringify(tc.input) },
      }));
    }
    return [out];
  }
  // role === "tool" — flatten each tool_result into its own tool message.
  return msg.content
    .filter((c): c is Extract<AgentContentBlock, { type: "tool_result" }> => c.type === "tool_result")
    .map((c) => ({ role: "tool", content: c.output, tool_call_id: c.tool_use_id }));
}

function collectText(blocks: AgentContentBlock[]): string {
  return blocks
    .flatMap((b) => (b.type === "text" ? [b.text] : []))
    .join("\n");
}

async function readToString(stream: NodeJS.ReadableStream): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of stream) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk as string));
  }
  return Buffer.concat(chunks).toString("utf8");
}
