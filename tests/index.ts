/**
 * Test Entry Point
 * 
 * Centralizes all test suites for organized execution
 */

// Unit Tests
import "./unit/UserManager.test";
import "./unit/utils.test";

// Integration Tests
import "./integration/auth.routes.test";
import "./integration/nodes.routes.test";
import "./integration/public.routes.test";

// Smoke Tests
import "./smoke/critical-paths.test";

// Edge Cases
import "./edge-cases/error-handling.test";

console.log("✅ All test suites loaded");
