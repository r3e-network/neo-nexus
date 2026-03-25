## Context

Neo-nexus currently exists as a basic infrastructure project. This design transforms it into a comprehensive web-based Neo node management platform. Users will install neo-nexus on their VMs and access a web interface to deploy, configure, and monitor Neo blockchain nodes (both neo-cli and neo-go) without manual CLI operations.

**Current State:** Minimal codebase with infrastructure configurations that need complete refactoring.

**Constraints:**

- Must support both neo-cli (.NET) and neo-go (Go) node types
- Must handle multiple node instances on single machine
- Must prevent port conflicts automatically
- Must work on Linux VMs (primary target)

## Goals / Non-Goals

**Goals:**

- Provide web-based UI for all node management operations
- Support neo-cli and neo-go deployment and configuration
- Enable multi-node management on single machine with port isolation
- Real-time monitoring of node status, health, and resource usage
- Plugin management for neo-cli nodes
- Storage management and backup capabilities
- REST API for programmatic access

**Non-Goals:**

- Blockchain consensus participation (nodes can run consensus, but we don't manage consensus logic)
- Wallet management UI (users manage wallets through node interfaces)
- Blockchain explorer functionality
- Multi-machine cluster management (single VM only)

## Decisions

### 1. Architecture: Monolithic Web Application

**Decision:** Build as single Node.js/TypeScript application with Express backend and React frontend.

**Rationale:**

- Simpler deployment (single process)
- Easier state management for node lifecycle
- Lower resource overhead on VM
- Faster development iteration

**Alternatives Considered:**

- Microservices: Rejected due to complexity and resource overhead for single-VM deployment
- Separate backend/frontend repos: Rejected to simplify deployment and versioning

### 2. Node Process Management: PM2

**Decision:** Use PM2 to manage neo-cli and neo-go node processes.

**Rationale:**

- Battle-tested process manager
- Auto-restart on crashes
- Log management built-in
- Programmatic API for start/stop/restart
- Resource monitoring capabilities

**Alternatives Considered:**

- systemd: Rejected due to complexity of dynamic service creation
- Custom process manager: Rejected due to reinventing wheel

### 3. Configuration Storage: File-based JSON + SQLite

**Decision:** Store node configurations in JSON files, metadata in SQLite.

**Rationale:**

- JSON files easy to backup and version control
- SQLite for queryable metadata (status, metrics history)
- No external database dependency
- Simple migration path

**Alternatives Considered:**

- PostgreSQL: Overkill for single-VM deployment
- Pure file-based: Difficult to query historical data

### 4. Frontend Framework: React + shadcn/ui

**Decision:** React with TypeScript, shadcn/ui components, TanStack Query for data fetching.

**Rationale:**

- Modern, maintainable stack
- shadcn/ui provides professional UI components
- TanStack Query handles real-time updates elegantly
- Strong TypeScript support

### 5. Real-time Updates: WebSocket + Polling Hybrid

**Decision:** WebSocket for live status updates, polling fallback for metrics.

**Rationale:**

- WebSocket for instant status changes
- Polling for resource metrics (less critical, reduces load)
- Graceful degradation if WebSocket fails

### 6. Port Management: Sequential Allocation with Validation

**Decision:** Allocate ports sequentially starting from configurable base (default 10333 for RPC, 10334 for P2P), validate availability before assignment.

**Rationale:**

- Predictable port assignments
- Easy to configure firewall rules
- Simple conflict detection

### 7. Node Directory Structure

**Decision:**

```
nodes/
  <node-id>/
    config/       # Node configuration files
    data/         # Blockchain data
    logs/         # Node logs
    plugins/      # neo-cli plugins (if applicable)
    metadata.json # Node metadata (type, ports, status)
```

**Rationale:**

- Complete isolation between nodes
- Easy backup/restore per node
- Clear organization

## Risks / Trade-offs

**Risk:** Node process crashes could leave orphaned processes
→ **Mitigation:** PM2 handles process lifecycle, implement health checks and cleanup on startup

**Risk:** Disk space exhaustion from blockchain data
→ **Mitigation:** Storage monitoring with alerts, configurable thresholds

**Risk:** Port conflicts if user manually starts nodes
→ **Mitigation:** Validate port availability before starting, clear error messages

**Risk:** Security - web interface exposed on network
→ **Mitigation:** Authentication required, HTTPS support, configurable bind address (default localhost)

**Trade-off:** Monolithic architecture limits horizontal scaling
→ **Acceptable:** Single-VM use case doesn't require scaling

**Trade-off:** PM2 dependency adds complexity
→ **Acceptable:** Benefits outweigh complexity, well-documented tool

## Migration Plan

**Phase 1: Core Infrastructure**

1. Set up Express server with authentication
2. Implement node process management layer
3. Create basic REST API

**Phase 2: Node Management**

1. Implement neo-cli deployment
2. Implement neo-go deployment
3. Add port management and validation

**Phase 3: Frontend**

1. Build React UI with shadcn/ui
2. Implement dashboard and node list
3. Add node configuration forms

**Phase 4: Monitoring & Advanced Features**

1. Add real-time monitoring
2. Implement plugin management
3. Add storage management

**Rollback Strategy:**

- Keep existing infrastructure configs until new system proven
- Export/import functionality for node configurations
- Manual node recovery procedures documented

## Open Questions

1. **Authentication method:** JWT tokens vs session-based? → **Decision needed before implementation**
2. **Binary management:** Auto-download neo-cli/neo-go binaries or require manual installation? → **Recommend auto-download for better UX**
3. **Backup strategy:** Automated backups or manual only? → **Start with manual, add automation later**
4. **Multi-user support:** Single admin or multiple users with roles? → **Start with single admin, add roles in v2**
