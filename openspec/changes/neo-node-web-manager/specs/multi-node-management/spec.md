## ADDED Requirements

### Requirement: Multiple node instances

The system SHALL allow running multiple Neo nodes simultaneously on the same machine.

#### Scenario: Deploy second node

- **WHEN** user creates a second node instance
- **THEN** system creates isolated directory and assigns unique ports

#### Scenario: Independent operation

- **WHEN** multiple nodes are running
- **THEN** each node operates independently without interference

### Requirement: Port conflict prevention

The system SHALL automatically detect and prevent port conflicts between node instances.

#### Scenario: Automatic port assignment

- **WHEN** user creates new node without specifying ports
- **THEN** system assigns next available ports automatically

#### Scenario: Port validation

- **WHEN** user specifies custom ports
- **THEN** system validates ports are not in use before deployment

### Requirement: Isolated node directories

The system SHALL create separate directories for each node instance containing config, data, and logs.

#### Scenario: Directory structure

- **WHEN** node is created
- **THEN** system creates nodes/<node-id>/ with config/, data/, logs/ subdirectories

### Requirement: Node lifecycle management

The system SHALL provide start, stop, restart, and delete operations for each node independently.

#### Scenario: Start specific node

- **WHEN** user starts node-1
- **THEN** only node-1 starts, other nodes remain in current state

#### Scenario: Delete node

- **WHEN** user deletes a node
- **THEN** system stops node and removes its directory
