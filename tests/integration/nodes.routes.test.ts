/**
 * Integration Tests: Node Management Routes
 * 
 * Tests the node management API endpoints
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import request from "supertest";
import express, { Request, Response } from "express";

describe("Nodes Routes Integration", () => {
  let app: express.Application;
  let mockNodeManager: any;

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockNodeManager = {
      createNode: vi.fn(),
      updateNode: vi.fn(),
      getNode: vi.fn(),
      getAllNodes: vi.fn(),
      deleteNode: vi.fn(),
      startNode: vi.fn(),
      stopNode: vi.fn(),
      restartNode: vi.fn(),
      getNodeLogs: vi.fn(),
    };

    // GET /api/nodes - List all nodes
    app.get("/api/nodes", (req: Request, res: Response) => {
      const nodes = mockNodeManager.getAllNodes();
      res.json({ nodes, count: nodes.length });
    });

    // POST /api/nodes - Create node
    app.post("/api/nodes", async (req: Request, res: Response) => {
      try {
        const node = await mockNodeManager.createNode(req.body);
        res.status(201).json({ node });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });

    // GET /api/nodes/:id - Get node details
    app.get("/api/nodes/:id", (req: Request, res: Response) => {
      const node = mockNodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: "Node not found" });
      }
      res.json({ node });
    });

    // PUT /api/nodes/:id - Update node
    app.put("/api/nodes/:id", async (req: Request, res: Response) => {
      try {
        const node = await mockNodeManager.updateNode(req.params.id, req.body);
        res.json({ node });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });

    // DELETE /api/nodes/:id - Delete node
    app.delete("/api/nodes/:id", async (req: Request, res: Response) => {
      try {
        await mockNodeManager.deleteNode(req.params.id);
        res.json({ success: true });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });

    // POST /api/nodes/:id/start - Start node
    app.post("/api/nodes/:id/start", async (req: Request, res: Response) => {
      try {
        const result = await mockNodeManager.startNode(req.params.id);
        res.json({ success: true, ...result });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });

    // POST /api/nodes/:id/stop - Stop node
    app.post("/api/nodes/:id/stop", async (req: Request, res: Response) => {
      try {
        const result = await mockNodeManager.stopNode(req.params.id);
        res.json({ success: true, ...result });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });

    // POST /api/nodes/:id/restart - Restart node
    app.post("/api/nodes/:id/restart", async (req: Request, res: Response) => {
      try {
        const result = await mockNodeManager.restartNode(req.params.id);
        res.json({ success: true, ...result });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });

    // GET /api/nodes/:id/logs - Get node logs
    app.get("/api/nodes/:id/logs", async (req: Request, res: Response) => {
      try {
        const logs = await mockNodeManager.getNodeLogs(req.params.id, req.query);
        res.json({ logs });
      } catch (error: any) {
        res.status(400).json({ error: error.message });
      }
    });
  });

  describe("GET /api/nodes", () => {
    it("should return all nodes", async () => {
      const mockNodes = [
        { id: "node-1", name: "Node 1", type: "neo-cli", status: "running" },
        { id: "node-2", name: "Node 2", type: "neo-go", status: "stopped" },
      ];
      mockNodeManager.getAllNodes.mockReturnValue(mockNodes);

      const response = await request(app)
        .get("/api/nodes")
        .expect(200);

      expect(response.body.nodes).toHaveLength(2);
      expect(response.body.count).toBe(2);
    });

    it("should handle empty node list", async () => {
      mockNodeManager.getAllNodes.mockReturnValue([]);

      const response = await request(app)
        .get("/api/nodes")
        .expect(200);

      expect(response.body.nodes).toHaveLength(0);
      expect(response.body.count).toBe(0);
    });
  });

  describe("POST /api/nodes", () => {
    it("should create a new node with valid config", async () => {
      const newNode = {
        id: "node-new",
        name: "New Node",
        type: "neo-cli",
        network: "mainnet",
        status: "created",
      };
      mockNodeManager.createNode.mockResolvedValue(newNode);

      const response = await request(app)
        .post("/api/nodes")
        .send({
          name: "New Node",
          type: "neo-cli",
          network: "mainnet",
          syncMode: "full",
        })
        .expect(201);

      expect(response.body.node.id).toBe("node-new");
    });

    it("should reject invalid node type", async () => {
      mockNodeManager.createNode.mockRejectedValue(new Error("Invalid node type"));

      const response = await request(app)
        .post("/api/nodes")
        .send({ type: "invalid" })
        .expect(400);

      expect(response.body.error).toContain("Invalid");
    });

    it("should reject duplicate node name", async () => {
      mockNodeManager.createNode.mockRejectedValue(new Error("Node name already exists"));

      const response = await request(app)
        .post("/api/nodes")
        .send({ name: "Existing Node" })
        .expect(400);

      expect(response.body.error).toContain("already exists");
    });
  });

  describe("GET /api/nodes/:id", () => {
    it("should return node details", async () => {
      const mockNode = {
        id: "node-1",
        name: "Test Node",
        type: "neo-cli",
        status: "running",
      };
      mockNodeManager.getNode.mockReturnValue(mockNode);

      const response = await request(app)
        .get("/api/nodes/node-1")
        .expect(200);

      expect(response.body.node.id).toBe("node-1");
    });

    it("should return 404 for non-existent node", async () => {
      mockNodeManager.getNode.mockReturnValue(null);

      const response = await request(app)
        .get("/api/nodes/non-existent")
        .expect(404);

      expect(response.body.error).toContain("not found");
    });
  });

  describe("POST /api/nodes/:id/start", () => {
    it("should start a stopped node", async () => {
      mockNodeManager.startNode.mockResolvedValue({ message: "Node started" });

      const response = await request(app)
        .post("/api/nodes/node-1/start")
        .expect(200);

      expect(response.body.success).toBe(true);
    });

    it("should fail if node is already running", async () => {
      mockNodeManager.startNode.mockRejectedValue(new Error("Node is already running"));

      const response = await request(app)
        .post("/api/nodes/node-1/start")
        .expect(400);

      expect(response.body.error).toContain("already running");
    });

    it("should fail for non-existent node", async () => {
      mockNodeManager.startNode.mockRejectedValue(new Error("Node not found"));

      const response = await request(app)
        .post("/api/nodes/non-existent/start")
        .expect(400);

      expect(response.body.error).toContain("not found");
    });
  });

  describe("PUT /api/nodes/:id", () => {
    it("should update a stopped node configuration", async () => {
      mockNodeManager.updateNode.mockResolvedValue({
        id: "node-1",
        name: "Updated Node",
        settings: {
          maxConnections: 80,
          minPeers: 8,
        },
      });

      const response = await request(app)
        .put("/api/nodes/node-1")
        .send({
          name: "Updated Node",
          settings: {
            maxConnections: 80,
            minPeers: 8,
          },
        })
        .expect(200);

      expect(mockNodeManager.updateNode).toHaveBeenCalledWith("node-1", {
        name: "Updated Node",
        settings: {
          maxConnections: 80,
          minPeers: 8,
        },
      });
      expect(response.body.node.name).toBe("Updated Node");
    });

    it("should reject updates while a node is running", async () => {
      mockNodeManager.updateNode.mockRejectedValue(
        new Error("Cannot update configuration while node is running"),
      );

      const response = await request(app)
        .put("/api/nodes/node-1")
        .send({
          name: "Updated Node",
        })
        .expect(400);

      expect(response.body.error).toContain("running");
    });
  });

  describe("POST /api/nodes/:id/stop", () => {
    it("should stop a running node", async () => {
      mockNodeManager.stopNode.mockResolvedValue({ message: "Node stopped" });

      const response = await request(app)
        .post("/api/nodes/node-1/stop")
        .expect(200);

      expect(response.body.success).toBe(true);
    });

    it("should handle graceful shutdown timeout", async () => {
      mockNodeManager.stopNode.mockRejectedValue(new Error("Shutdown timeout"));

      const response = await request(app)
        .post("/api/nodes/node-1/stop")
        .expect(400);

      expect(response.body.error).toContain("timeout");
    });
  });

  describe("POST /api/nodes/:id/restart", () => {
    it("should restart a node", async () => {
      mockNodeManager.restartNode.mockResolvedValue({ message: "Node restarted" });

      const response = await request(app)
        .post("/api/nodes/node-1/restart")
        .expect(200);

      expect(response.body.success).toBe(true);
    });
  });

  describe("DELETE /api/nodes/:id", () => {
    it("should delete a node and clean up resources", async () => {
      mockNodeManager.deleteNode.mockResolvedValue(undefined);

      const response = await request(app)
        .delete("/api/nodes/node-1")
        .expect(200);

      expect(response.body.success).toBe(true);
    });

    it("should fail to delete running node", async () => {
      mockNodeManager.deleteNode.mockRejectedValue(new Error("Cannot delete running node"));

      const response = await request(app)
        .delete("/api/nodes/node-1")
        .expect(400);

      expect(response.body.error).toContain("Cannot delete");
    });
  });

  describe("GET /api/nodes/:id/logs", () => {
    it("should return node logs", async () => {
      mockNodeManager.getNodeLogs.mockResolvedValue(["Log line 1", "Log line 2"]);

      const response = await request(app)
        .get("/api/nodes/node-1/logs")
        .expect(200);

      expect(response.body.logs).toHaveLength(2);
    });

    it("should support query parameters for log filtering", async () => {
      mockNodeManager.getNodeLogs.mockResolvedValue(["Recent log"]);

      const response = await request(app)
        .get("/api/nodes/node-1/logs?lines=100&since=1234567890")
        .expect(200);

      expect(mockNodeManager.getNodeLogs).toHaveBeenCalledWith(
        "node-1",
        expect.objectContaining({ lines: "100", since: "1234567890" })
      );
    });
  });
});
