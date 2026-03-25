## ADDED Requirements

### Requirement: Node type selection

The system SHALL support deployment of both neo-cli and neo-go node types.

#### Scenario: Select neo-cli

- **WHEN** user selects neo-cli as node type
- **THEN** system configures deployment for neo-cli with .NET runtime

#### Scenario: Select neo-go

- **WHEN** user selects neo-go as node type
- **THEN** system configures deployment for neo-go with Go runtime

### Requirement: Network configuration

The system SHALL allow users to configure node for MainNet, TestNet, or private network.

#### Scenario: MainNet deployment

- **WHEN** user selects MainNet
- **THEN** system downloads MainNet blockchain configuration

#### Scenario: TestNet deployment

- **WHEN** user selects TestNet
- **THEN** system downloads TestNet blockchain configuration

### Requirement: Node configuration

The system SHALL provide UI for configuring all node parameters including RPC port, P2P port, and consensus settings.

#### Scenario: Configure RPC port

- **WHEN** user sets custom RPC port
- **THEN** system validates port availability and updates node config

#### Scenario: Configure consensus

- **WHEN** user enables consensus mode
- **THEN** system prompts for consensus wallet and password

### Requirement: Binary management

The system SHALL automatically download and manage neo-cli and neo-go binaries.

#### Scenario: Download neo-cli

- **WHEN** user deploys neo-cli node
- **THEN** system downloads latest neo-cli release from GitHub

#### Scenario: Version selection

- **WHEN** user selects specific version
- **THEN** system downloads that version
