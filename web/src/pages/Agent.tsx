import { useEffect, useMemo, useRef, useState } from 'react';
import { Sparkles, Send, StopCircle, Plus, Trash2, Wrench, AlertCircle } from 'lucide-react';
import { useWebSocket } from '../hooks/useWebSocket';
import { useAuth } from '../hooks/useAuth';

type Provider = 'anthropic' | 'openai' | 'openai-compatible';

interface AgentSettingsResponse {
  configured: boolean;
  provider?: Provider;
  model?: string;
  baseUrl?: string | null;
  enabled?: boolean;
  apiKey?: string;
}

interface AgentConversation {
  id: string;
  title?: string;
  provider: Provider;
  model: string;
  createdAt: number;
  updatedAt: number;
}

type ContentBlock =
  | { type: 'text'; text: string }
  | { type: 'tool_use'; id: string; name: string; input: Record<string, unknown> }
  | { type: 'tool_result'; tool_use_id: string; output: string; isError?: boolean };

interface AgentMessage {
  id: string;
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: ContentBlock[];
  createdAt: number;
}

const HEADERS = (token: string) => ({
  'Content-Type': 'application/json',
  Authorization: `Bearer ${token}`,
});

export default function Agent() {
  const { token } = useAuth();
  const { lastMessage, connected, sendMessage } = useWebSocket();

  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [settings, setSettings] = useState<AgentSettingsResponse | null>(null);
  const [conversations, setConversations] = useState<AgentConversation[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [messages, setMessages] = useState<AgentMessage[]>([]);
  const [input, setInput] = useState('');
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Initial load.
  useEffect(() => {
    if (!token) return;
    void (async () => {
      try {
        const [healthRes, settingsRes, convRes] = await Promise.all([
          fetch('/api/agent/health', { headers: HEADERS(token) }),
          fetch('/api/agent/settings', { headers: HEADERS(token) }),
          fetch('/api/agent/conversations', { headers: HEADERS(token) }),
        ]);
        const health = (await healthRes.json()) as { enabled: boolean };
        setEnabled(health.enabled);
        const settingsBody = (await settingsRes.json()) as AgentSettingsResponse;
        setSettings(settingsBody);
        const convBody = (await convRes.json()) as { conversations: AgentConversation[] };
        setConversations(convBody.conversations);
        if (convBody.conversations.length > 0) {
          setActiveId(convBody.conversations[0].id);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load agent');
      }
    })();
  }, [token]);

  // Load messages when active conversation changes.
  useEffect(() => {
    if (!token || !activeId) {
      setMessages([]);
      return;
    }
    void (async () => {
      const res = await fetch(`/api/agent/conversations/${activeId}`, { headers: HEADERS(token) });
      if (!res.ok) return;
      const body = (await res.json()) as { messages: AgentMessage[] };
      setMessages(body.messages);
    })();
  }, [token, activeId]);

  // Stream agent events into the message list.
  useEffect(() => {
    if (!lastMessage || typeof lastMessage === 'string') return;
    const ev = lastMessage as { type: string; conversationId?: string; messageId?: string; role?: AgentMessage['role']; text?: string; toolUseId?: string; name?: string; input?: Record<string, unknown>; output?: string; isError?: boolean; error?: string };
    if (!ev.type?.startsWith('agent.')) return;
    if (ev.conversationId && activeId && ev.conversationId !== activeId) return;
    const subtype = ev.type.slice('agent.'.length);

    setMessages((prev) => {
      const next = [...prev];
      if (subtype === 'message_start' && ev.messageId && ev.role) {
        next.push({ id: ev.messageId, role: ev.role, content: [], createdAt: Date.now() });
      } else if (subtype === 'delta' && ev.messageId && ev.text) {
        const idx = next.findIndex((m) => m.id === ev.messageId);
        if (idx >= 0) {
          const msg = next[idx];
          const last = msg.content.at(-1);
          if (last?.type === 'text') last.text += ev.text;
          else msg.content.push({ type: 'text', text: ev.text });
          next[idx] = { ...msg, content: [...msg.content] };
        }
      } else if (subtype === 'tool_use' && ev.messageId && ev.toolUseId && ev.name) {
        const idx = next.findIndex((m) => m.id === ev.messageId);
        if (idx >= 0) {
          next[idx] = {
            ...next[idx],
            content: [...next[idx].content, { type: 'tool_use', id: ev.toolUseId, name: ev.name, input: ev.input ?? {} }],
          };
        }
      } else if (subtype === 'tool_result' && ev.messageId && ev.toolUseId) {
        const idx = next.findIndex((m) => m.id === ev.messageId);
        if (idx >= 0) {
          next[idx] = {
            ...next[idx],
            content: [
              ...next[idx].content,
              { type: 'tool_result', tool_use_id: ev.toolUseId, output: ev.output ?? '', isError: ev.isError },
            ],
          };
        }
      }
      return next;
    });

    if (subtype === 'complete') setRunning(false);
    if (subtype === 'error') {
      setRunning(false);
      setError(ev.error ?? 'Agent error');
    }
  }, [lastMessage, activeId]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const newConversation = async () => {
    if (!token || !settings?.configured) return;
    const res = await fetch('/api/agent/conversations', {
      method: 'POST',
      headers: HEADERS(token),
      body: JSON.stringify({ title: 'New chat' }),
    });
    if (!res.ok) return;
    const body = (await res.json()) as { conversation: AgentConversation };
    setConversations((c) => [body.conversation, ...c]);
    setActiveId(body.conversation.id);
    setMessages([]);
  };

  const deleteConversation = async (id: string) => {
    if (!token) return;
    await fetch(`/api/agent/conversations/${id}`, { method: 'DELETE', headers: HEADERS(token) });
    setConversations((c) => c.filter((conv) => conv.id !== id));
    if (activeId === id) setActiveId(null);
  };

  const send = () => {
    const text = input.trim();
    if (!text || !activeId || running) return;
    if (!sendMessage({ type: 'agent.send', conversationId: activeId, text })) {
      setError('WebSocket is not connected');
      return;
    }
    setInput('');
    setRunning(true);
    setError(null);
  };

  const cancel = () => {
    if (!activeId) return;
    sendMessage({ type: 'agent.cancel', conversationId: activeId });
    setRunning(false);
  };

  const activeConversation = useMemo(
    () => conversations.find((c) => c.id === activeId) ?? null,
    [conversations, activeId],
  );

  if (enabled === false) {
    return (
      <div className="max-w-3xl mx-auto p-8">
        <div className="rounded-2xl bg-amber-50 border border-amber-200 p-6">
          <div className="flex items-start gap-3">
            <Sparkles className="w-5 h-5 text-amber-600 mt-0.5" />
            <div>
              <h2 className="text-lg font-semibold text-amber-900">Hermes is disabled</h2>
              <p className="mt-1 text-sm text-amber-800">
                Set the environment variable <code className="px-1.5 py-0.5 bg-amber-100 rounded">NEONEXUS_ENABLE_HERMES_AGENT=true</code> on the server and restart NeoNexus to enable the in-app AI agent.
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (settings && !settings.configured) {
    return <SettingsPanel token={token!} onSaved={(s) => setSettings(s)} />;
  }

  return (
    <div className="flex flex-col md:flex-row h-[calc(100vh-4rem)] gap-4 p-4">
      {/* Conversation list — collapses to a horizontally-scrolling pill bar on mobile */}
      <aside className="md:w-64 md:flex-shrink-0 flex md:flex-col bg-white rounded-xl border border-slate-200 overflow-hidden">
        <div className="p-3 border-b-0 md:border-b md:border-slate-200 md:w-full">
          <button
            onClick={newConversation}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg text-sm font-medium transition whitespace-nowrap"
          >
            <Plus className="w-4 h-4" />
            New chat
          </button>
        </div>
        <div className="flex-1 overflow-x-auto md:overflow-x-visible md:overflow-y-auto flex md:block">
          {conversations.length === 0 && (
            <div className="p-4 text-sm text-slate-500">No conversations yet.</div>
          )}
          {conversations.map((c) => (
            <button
              key={c.id}
              onClick={() => setActiveId(c.id)}
              className={`shrink-0 md:shrink md:w-full px-3 py-2 text-left flex items-start gap-2 group hover:bg-slate-50 ${activeId === c.id ? 'bg-slate-100' : ''}`}
            >
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium text-slate-900 truncate">{c.title || 'Untitled'}</div>
                <div className="text-xs text-slate-500">{new Date(c.updatedAt).toLocaleString()}</div>
              </div>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  void deleteConversation(c.id);
                }}
                className="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-red-500"
                aria-label="Delete conversation"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </button>
          ))}
        </div>
        <div className="hidden md:block p-3 border-t border-slate-200 text-xs text-slate-500">
          <div className="flex items-center gap-1.5">
            <span className={`inline-block w-1.5 h-1.5 rounded-full ${connected ? 'bg-emerald-500' : 'bg-slate-300'}`} />
            {connected ? 'Streaming live' : 'Disconnected'}
          </div>
          {settings?.provider && (
            <div className="mt-1">
              {settings.provider} · <span className="font-mono">{settings.model}</span>
            </div>
          )}
        </div>
      </aside>

      {/* Conversation pane */}
      <main className="flex-1 flex flex-col bg-white rounded-xl border border-slate-200 overflow-hidden">
        <div className="px-5 py-3 border-b border-slate-200 flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-emerald-600" />
          <h1 className="text-lg font-semibold text-slate-900">{activeConversation?.title || 'Hermes Agent'}</h1>
        </div>

        <div className="flex-1 overflow-y-auto px-5 py-4 space-y-4">
          {messages.length === 0 && (
            <div className="text-center py-12 text-slate-500">
              <Sparkles className="w-8 h-8 text-slate-300 mx-auto mb-2" />
              <p className="text-sm">Ask about node status, logs, or metrics.</p>
              <p className="text-xs mt-1">Examples: "what nodes do I have?", "show recent errors on NeoCLI-Testnet", "stop NeoGo-Testnet"</p>
            </div>
          )}
          {messages.filter(isVisibleMessage).map((m) => (
            <MessageRow key={m.id} message={m} />
          ))}
          {error && (
            <div className="flex items-start gap-2 text-sm text-red-700 bg-red-50 border border-red-200 rounded-lg p-3">
              <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
              <div>{error}</div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>

        <div className="px-4 py-3 border-t border-slate-200 bg-slate-50/50">
          {!activeId ? (
            <div className="text-sm text-slate-500 text-center">Pick a conversation or start a new chat.</div>
          ) : (
            <div className="flex items-end gap-2">
              <textarea
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    send();
                  }
                }}
                placeholder="Ask the agent..."
                disabled={running}
                rows={2}
                className="flex-1 px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-500 text-sm resize-none disabled:bg-slate-100"
              />
              {running ? (
                <button
                  onClick={cancel}
                  className="flex items-center gap-1.5 px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg text-sm font-medium transition"
                >
                  <StopCircle className="w-4 h-4" />
                  Stop
                </button>
              ) : (
                <button
                  onClick={send}
                  disabled={!input.trim()}
                  className="flex items-center gap-1.5 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-300 text-white rounded-lg text-sm font-medium transition"
                >
                  <Send className="w-4 h-4" />
                  Send
                </button>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

function isVisibleMessage(m: AgentMessage): boolean {
  // Drop assistant messages that ended up empty (e.g. provider error before any
  // text or tool call streamed). Otherwise the chat shows a blank bubble next
  // to the error banner, which reads as a layout glitch.
  if (m.role === 'assistant' && m.content.length === 0) return false;
  if (m.role === 'tool' && m.content.length === 0) return false;
  return true;
}

function MessageRow({ message }: { message: AgentMessage }) {
  if (message.role === 'tool') {
    return (
      <div className="space-y-2">
        {message.content.map((block, i) =>
          block.type === 'tool_result' ? (
            <div
              key={i}
              className={`rounded-lg px-3 py-2 text-xs font-mono ${block.isError ? 'bg-red-50 border border-red-200 text-red-900' : 'bg-slate-50 border border-slate-200 text-slate-700'}`}
            >
              <div className="text-[10px] uppercase tracking-wide text-slate-500 mb-1">
                tool result {block.isError ? '(error)' : ''}
              </div>
              <pre className="whitespace-pre-wrap break-words">{block.output}</pre>
            </div>
          ) : null,
        )}
      </div>
    );
  }
  const isUser = message.role === 'user';
  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div className={`max-w-[80%] ${isUser ? 'bg-emerald-600 text-white' : 'bg-slate-100 text-slate-900'} rounded-2xl px-4 py-2.5`}>
        {message.content.map((block, i) => {
          if (block.type === 'text') {
            return <p key={i} className="whitespace-pre-wrap text-sm">{block.text}</p>;
          }
          if (block.type === 'tool_use') {
            return (
              <div key={i} className="mt-2 flex items-center gap-1.5 text-xs opacity-90">
                <Wrench className="w-3 h-3" />
                Calling <span className="font-mono">{block.name}</span>
                {Object.keys(block.input).length > 0 && (
                  <span className="font-mono opacity-70">({Object.keys(block.input).join(', ')})</span>
                )}
              </div>
            );
          }
          return null;
        })}
      </div>
    </div>
  );
}

function SettingsPanel({ token, onSaved }: { token: string; onSaved: (s: AgentSettingsResponse) => void }) {
  const [provider, setProvider] = useState<Provider>('anthropic');
  const [model, setModel] = useState('claude-sonnet-4-6');
  const [apiKey, setApiKey] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setErr(null);
    try {
      const res = await fetch('/api/agent/settings', {
        method: 'PUT',
        headers: HEADERS(token),
        body: JSON.stringify({ provider, model, apiKey, baseUrl: baseUrl || undefined, enabled: true }),
      });
      if (!res.ok) {
        const body = (await res.json().catch(() => ({}))) as { error?: string };
        throw new Error(body.error ?? `HTTP ${res.status}`);
      }
      const reload = await fetch('/api/agent/settings', { headers: HEADERS(token) });
      onSaved((await reload.json()) as AgentSettingsResponse);
    } catch (error) {
      setErr(error instanceof Error ? error.message : 'Failed to save');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto p-8">
      <div className="bg-white rounded-2xl border border-slate-200 p-6">
        <div className="flex items-center gap-2 mb-4">
          <Sparkles className="w-6 h-6 text-emerald-600" />
          <h1 className="text-xl font-semibold text-slate-900">Set up Hermes Agent</h1>
        </div>
        <p className="text-sm text-slate-600 mb-6">
          Bring your own API key. Conversations are stored per-user; tools inherit your role (admin or viewer).
        </p>
        <form onSubmit={submit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Provider</label>
            <select
              value={provider}
              onChange={(e) => setProvider(e.target.value as Provider)}
              className="w-full px-3 py-2 border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-500"
            >
              <option value="anthropic">Anthropic (Claude)</option>
              <option value="openai">OpenAI (GPT)</option>
              <option value="openai-compatible">OpenAI-compatible (Mistral, Groq, vLLM, OpenRouter, ...)</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Model</label>
            <input
              type="text"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder={provider === 'anthropic' ? 'claude-sonnet-4-6' : provider === 'openai' ? 'gpt-4o-mini' : 'mistral-large-latest'}
              className="w-full px-3 py-2 border border-slate-200 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">API key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={provider === 'anthropic' ? 'sk-ant-...' : 'sk-...'}
              className="w-full px-3 py-2 border border-slate-200 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-500"
            />
            <p className="text-xs text-slate-500 mt-1">Stored on the server. Only the last 4 characters are returned to the UI.</p>
          </div>
          {provider === 'openai-compatible' && (
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">Base URL</label>
              <input
                type="url"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder="https://api.mistral.ai"
                className="w-full px-3 py-2 border border-slate-200 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-emerald-500/30 focus:border-emerald-500"
              />
              <p className="text-xs text-slate-500 mt-1">Anything that exposes <code>/v1/chat/completions</code> with the OpenAI schema.</p>
            </div>
          )}
          {err && (
            <div className="text-sm text-red-700 bg-red-50 border border-red-200 rounded-lg p-3">{err}</div>
          )}
          <button
            type="submit"
            disabled={saving || !apiKey || !model}
            className="w-full px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-300 text-white rounded-lg text-sm font-medium transition"
          >
            {saving ? 'Saving...' : 'Save & start'}
          </button>
        </form>
      </div>
    </div>
  );
}
