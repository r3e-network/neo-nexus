import type { AgentProvider, ProviderClient } from "../types";
import { anthropicClient } from "./anthropic";
import { openaiClient } from "./openai";

export function getProviderClient(provider: AgentProvider): ProviderClient {
  if (provider === "anthropic") return anthropicClient;
  // OpenAI and OpenAI-compatible share the same wire protocol; the only
  // difference is the base URL the caller supplies.
  if (provider === "openai" || provider === "openai-compatible") return openaiClient;
  throw new Error(`Unknown agent provider: ${String(provider)}`);
}
