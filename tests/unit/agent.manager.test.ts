import { beforeEach, describe, expect, it, vi } from "vitest";

// The global tests/setup.ts mocks better-sqlite3 so most managers can run with
// stubbed prepare/run. AgentManager exercises real SQL semantics, so we restore
// the real module for this file.
vi.doUnmock("better-sqlite3");

const mockState = vi.hoisted(() => ({ events: [] as unknown[][] }));

vi.mock("../../src/agent/providers", () => {
  return {
    getProviderClient: () => ({
      async *stream() {
        const turn = mockState.events.shift() ?? [];
        for (const ev of turn) yield ev;
      },
    }),
  };
});

const { default: Database } = await import("better-sqlite3");
const { AgentManager } = await import("../../src/agent/AgentManager");
const { initializeDatabase } = await import("../../src/database/schema");
type AgentEvent = import("../../src/agent/types").AgentEvent;

function makeDb() {
  // Use an in-memory DB so we exercise the real schema.
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE users (id TEXT PRIMARY KEY, username TEXT, password_hash TEXT, role TEXT, created_at INTEGER);
    CREATE TABLE agent_settings (
      user_id TEXT PRIMARY KEY,
      provider TEXT NOT NULL,
      model TEXT NOT NULL,
      api_key TEXT NOT NULL,
      base_url TEXT,
      enabled INTEGER NOT NULL DEFAULT 1,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );
    CREATE TABLE agent_conversations (
      id TEXT PRIMARY KEY,
      user_id TEXT NOT NULL,
      title TEXT,
      provider TEXT NOT NULL,
      model TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );
    CREATE TABLE agent_messages (
      id TEXT PRIMARY KEY,
      conversation_id TEXT NOT NULL,
      role TEXT NOT NULL,
      content TEXT NOT NULL,
      created_at INTEGER NOT NULL
    );
    INSERT INTO users (id, username, password_hash, role, created_at) VALUES ('u1', 'alice', '', 'admin', 0);
  `);
  return db;
}

function makeDeps() {
  return {
    nodeManager: {
      getAllNodes: () => [
        {
          id: "n1",
          name: "alpha",
          type: "neo-go",
          network: "testnet",
          version: "v0.118.0",
          ports: { rpc: 10332, p2p: 10333 },
          settings: {},
          process: { status: "stopped" },
          metrics: { blockHeight: 1, headerHeight: 1, connectedPeers: 0, unconnectedPeers: 0, syncProgress: 0, memoryUsage: 0, cpuUsage: 0, lastUpdate: 0 },
        },
      ],
      getNode: () => null,
      getNodeLogs: () => [],
      startNode: vi.fn(),
      stopNode: vi.fn(),
      restartNode: vi.fn(),
      setPluginEnabled: vi.fn(),
    },
    remoteServerManager: { listServersWithStatus: async () => [] },
    integrationManager: { listAll: () => [] },
    metricsCollector: { collectSystemMetrics: async () => ({ cpu: { usage: 0, cores: 1 }, memory: { total: 1, used: 0, free: 1, percentage: 0 }, disk: { total: 1, used: 0, free: 1, percentage: 0 }, network: { rx: 0, tx: 0 } }) },
    networkHeightTracker: { getHeight: () => 9999 },
    auditLogger: { log: vi.fn() },
  } as never;
}

describe("AgentManager", () => {
  beforeEach(() => {
    mockState.events = [];
    delete process.env.NEONEXUS_ALLOW_INSECURE_AGENT_PROVIDER_URLS;
    delete process.env.NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS;
  });

  it("refuses to send when feature flag is off", async () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: false });
    mgr.saveSettings("u1", { provider: "anthropic", model: "claude-x", apiKey: "k" });
    const conv = mgr.createConversation("u1", mgr.getSettings("u1")!);
    await expect(mgr.send({
      user: { id: "u1", username: "alice", role: "admin" },
      conversationId: conv.id,
      text: "hi",
      onEvent: () => {},
      signal: new AbortController().signal,
    })).rejects.toThrow(/disabled/i);
  });

  it("requires baseUrl for openai-compatible provider", () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });
    expect(() =>
      mgr.saveSettings("u1", { provider: "openai-compatible", model: "x", apiKey: "k" }),
    ).toThrow(/baseUrl/i);
  });

  it("normalizes public HTTPS provider base URLs before persisting", () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });

    const settings = mgr.saveSettings("u1", {
      provider: "openai-compatible",
      model: "x",
      apiKey: "k",
      baseUrl: "https://api.example.com/",
    });

    expect(settings.baseUrl).toBe("https://api.example.com");
  });

  it("rejects insecure agent provider base URLs by default", () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });

    expect(() =>
      mgr.saveSettings("u1", {
        provider: "openai-compatible",
        model: "x",
        apiKey: "k",
        baseUrl: "http://api.example.com",
      }),
    ).toThrow(/HTTPS/i);
  });

  it("rejects private literal agent provider base URLs by default", () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });

    expect(() =>
      mgr.saveSettings("u1", {
        provider: "openai-compatible",
        model: "x",
        apiKey: "k",
        baseUrl: "https://127.0.0.1:11434",
      }),
    ).toThrow(/private|local/i);
  });

  it("redacts settings round-trip and persists conversations + messages", async () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });
    mgr.saveSettings("u1", { provider: "anthropic", model: "claude-x", apiKey: "secret-key-1234" });
    const settings = mgr.getSettings("u1")!;
    expect(settings.provider).toBe("anthropic");
    expect(settings.apiKey).toBe("secret-key-1234");
    const conv = mgr.createConversation("u1", settings, "First chat");
    expect(mgr.listConversations("u1")).toHaveLength(1);
    expect(mgr.listMessages(conv.id)).toEqual([]);
  });

  it("rejects oversized agent messages before persistence or provider calls", async () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });
    mgr.saveSettings("u1", { provider: "anthropic", model: "claude-x", apiKey: "k" });
    const conv = mgr.createConversation("u1", mgr.getSettings("u1")!);

    await expect(mgr.send({
      user: { id: "u1", username: "alice", role: "admin" },
      conversationId: conv.id,
      text: "x".repeat(16_001),
      onEvent: () => {},
      signal: new AbortController().signal,
    })).rejects.toThrow(/16000 characters or fewer/);
    expect(mgr.listMessages(conv.id)).toEqual([]);
  });

  it("runs a turn end-to-end: streams text, persists user + assistant messages, completes", async () => {
    const db = makeDb();
    const mgr = new AgentManager(db, { deps: makeDeps(), enabled: true });
    mgr.saveSettings("u1", { provider: "anthropic", model: "claude-x", apiKey: "k" });
    const conv = mgr.createConversation("u1", mgr.getSettings("u1")!);

    mockState.events = [[
      { type: "text_delta", text: "Hello " },
      { type: "text_delta", text: "world" },
      { type: "stop", reason: "end_turn" },
    ]];

    const events: AgentEvent[] = [];
    await mgr.send({
      user: { id: "u1", username: "alice", role: "admin" },
      conversationId: conv.id,
      text: "hi",
      onEvent: (e) => events.push(e),
      signal: new AbortController().signal,
    });

    const types = events.map((e) => e.type);
    expect(types).toEqual(expect.arrayContaining(["message_start", "delta", "message_end", "complete"]));
    const messages = mgr.listMessages(conv.id);
    expect(messages).toHaveLength(2);
    expect(messages[0].role).toBe("user");
    expect(messages[1].role).toBe("assistant");
    const assistantText = (messages[1].content[0] as { type: string; text: string }).text;
    expect(assistantText).toBe("Hello world");
  });

  it("loops on tool_use, executes the tool, persists tool message, then completes", async () => {
    const db = makeDb();
    const deps = makeDeps();
    const mgr = new AgentManager(db, { deps, enabled: true });
    mgr.saveSettings("u1", { provider: "anthropic", model: "claude-x", apiKey: "k" });
    const conv = mgr.createConversation("u1", mgr.getSettings("u1")!);

    mockState.events = [
      // First turn — model asks to call list_nodes
      [
        { type: "text_delta", text: "checking..." },
        { type: "tool_use", id: "call_1", name: "list_nodes", input: {} },
        { type: "stop", reason: "tool_use" },
      ],
      // Second turn — model responds with the result
      [
        { type: "text_delta", text: "you have 1 node" },
        { type: "stop", reason: "end_turn" },
      ],
    ];

    const events: AgentEvent[] = [];
    await mgr.send({
      user: { id: "u1", username: "alice", role: "admin" },
      conversationId: conv.id,
      text: "what nodes do I have?",
      onEvent: (e) => events.push(e),
      signal: new AbortController().signal,
    });

    const messages = mgr.listMessages(conv.id);
    expect(messages.map((m) => m.role)).toEqual(["user", "assistant", "tool", "assistant"]);
    // Tool result content is a stringified JSON of the node summary.
    const toolMsg = messages[2];
    expect(toolMsg.content[0]).toMatchObject({ type: "tool_result", tool_use_id: "call_1", isError: false });
    expect(events.some((e) => e.type === "tool_use")).toBe(true);
    expect(events.some((e) => e.type === "tool_result")).toBe(true);
    expect(events.at(-1)).toMatchObject({ type: "complete" });
  });

  it("captures tool errors and continues the conversation", async () => {
    const db = makeDb();
    const deps = makeDeps() as unknown as { nodeManager: { getNode: () => unknown } };
    deps.nodeManager.getNode = () => null; // get_node will throw "Node not found"
    const mgr = new AgentManager(db, { deps: deps as never, enabled: true });
    mgr.saveSettings("u1", { provider: "anthropic", model: "claude-x", apiKey: "k" });
    const conv = mgr.createConversation("u1", mgr.getSettings("u1")!);

    mockState.events = [
      [
        { type: "tool_use", id: "call_x", name: "get_node", input: { node_id: "missing" } },
        { type: "stop", reason: "tool_use" },
      ],
      [
        { type: "text_delta", text: "I couldn't find that node." },
        { type: "stop", reason: "end_turn" },
      ],
    ];

    const events: AgentEvent[] = [];
    await mgr.send({
      user: { id: "u1", username: "alice", role: "admin" },
      conversationId: conv.id,
      text: "tell me about node missing",
      onEvent: (e) => events.push(e),
      signal: new AbortController().signal,
    });

    const messages = mgr.listMessages(conv.id);
    const toolMsg = messages.find((m) => m.role === "tool");
    expect(toolMsg).toBeDefined();
    expect(toolMsg!.content[0]).toMatchObject({ type: "tool_result", isError: true });
  });
});

describe("schema initializeDatabase", () => {
  it("creates agent tables", async () => {
    const db = await initializeDatabase();
    const tables = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table'").all().map((r: unknown) => (r as { name: string }).name);
    expect(tables).toEqual(expect.arrayContaining(["agent_settings", "agent_conversations", "agent_messages"]));
    db.close();
  });
});
