## ADDED Requirements

### Requirement: REST API endpoints

The system SHALL provide REST API for all node management operations.

#### Scenario: Create node via API

- **WHEN** client POSTs to /api/nodes with configuration
- **THEN** system creates node and returns node ID

#### Scenario: List nodes via API

- **WHEN** client GETs /api/nodes
- **THEN** system returns array of all nodes with status

### Requirement: API authentication

The system SHALL require API key authentication for all API requests.

#### Scenario: Authenticated request

- **WHEN** client includes valid API key in header
- **THEN** system processes request

#### Scenario: Unauthenticated request

- **WHEN** client omits API key
- **THEN** system returns 401 Unauthorized

### Requirement: WebSocket support

The system SHALL provide WebSocket endpoint for real-time updates.

#### Scenario: Subscribe to node updates

- **WHEN** client connects to WebSocket and subscribes to node
- **THEN** system sends real-time status updates

### Requirement: API documentation

The system SHALL provide OpenAPI/Swagger documentation for all endpoints.

#### Scenario: View API docs

- **WHEN** user navigates to /api/docs
- **THEN** system displays interactive API documentation
