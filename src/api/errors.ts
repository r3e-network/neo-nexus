export class ApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly suggestion: string,
    public readonly status: number = 400,
  ) {
    super(message);
    this.name = "ApiError";
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export const Errors = {
  // Node operations
  nodeNotFound: (id?: string) =>
    new ApiError("NODE_NOT_FOUND",
      id ? `Node ${id} not found` : "Node not found",
      "Check the node ID — it may have been deleted. Use GET /api/nodes to list active nodes.", 404),
  nodeRunning: () =>
    new ApiError("NODE_RUNNING",
      "Cannot update configuration while node is running",
      "Stop the node first, then retry the update."),
  nodeAlreadyRunning: () =>
    new ApiError("NODE_ALREADY_RUNNING",
      "Node is already running",
      "This node is already started. Use restart if you want to cycle it."),
  nodeNotRunning: () =>
    new ApiError("NODE_NOT_RUNNING",
      "Node is not running",
      "The node is already stopped. Use start to launch it."),

  // Validation
  missingFields: (...fields: string[]) =>
    new ApiError("MISSING_FIELDS",
      `Missing required fields: ${fields.join(", ")}`,
      'Provide all required fields. Type must be "neo-cli" or "neo-go", network must be "mainnet" or "testnet".'),
  missingField: (field: string) =>
    new ApiError("MISSING_FIELDS",
      `Missing required field: ${field}`,
      `The "${field}" field is required.`),
  nameExists: (name: string) =>
    new ApiError("NAME_EXISTS",
      `Node name "${name}" already exists`,
      "Choose a different name — each node must have a unique display name."),
  pathBlocked: (path: string) =>
    new ApiError("PATH_BLOCKED",
      `Access to path ${path} is not permitted`,
      "Paths must be under /home, /opt, or /var/lib. System directories are blocked for safety."),
  pathNotAllowed: () =>
    new ApiError("PATH_NOT_ALLOWED",
      "Path must be under an allowed directory",
      "Allowed directories: /home, /opt, /var/lib, and the NeoNexus data directory."),
  pathNotFound: (path: string) =>
    new ApiError("PATH_NOT_FOUND",
      `Path does not exist: ${path}`,
      "The directory does not exist on this machine. Double-check the path for typos.", 404),

  // Detection & Import
  detectionFailed: (path: string) =>
    new ApiError("DETECTION_FAILED",
      `Could not detect valid node installation at ${path}. Make sure the path contains a valid neo-cli or neo-go installation.`,
      "The path must contain a neo-cli (config.json + binary) or neo-go (config.yaml/protocol.yml + binary) installation."),
  detectionNotFound: () =>
    new ApiError("DETECTION_NOT_FOUND",
      "No valid node installation detected at the specified path",
      "No recognizable node files found. Verify the path points to the directory containing the node binary and config files.", 404),
  importInvalid: (errors: string[]) =>
    new ApiError("IMPORT_INVALID",
      `Invalid node configuration: ${errors.join(", ")}`,
      "The installation was detected but has issues. Check that data and config paths exist and ports are in the valid range (1-65535)."),

  // Secure signer
  signerRequiresProfile: () =>
    new ApiError("SIGNER_REQUIRES_PROFILE",
      "Secure signer protection requires a signer profile",
      "Create a signer profile in Settings > Secure Signers first, then reference its ID here."),
  signerNeoCliOnly: () =>
    new ApiError("SIGNER_NEO_CLI_ONLY",
      "Secure signer protection currently requires a neo-cli node with SignClient support",
      "Only neo-cli nodes support the SignClient plugin. Switch to neo-cli or use standard wallet mode."),
  signerNotAvailable: (id: string) =>
    new ApiError("SIGNER_NOT_AVAILABLE",
      `Secure signer profile ${id} is not available`,
      "The profile may be disabled or deleted. Check Settings > Secure Signers to verify it is active."),
  signerProfileNotFound: () =>
    new ApiError("SIGNER_NOT_FOUND",
      "Secure signer profile not found",
      "The requested signer profile does not exist. Check the profile ID.", 404),
  signerFieldsRequired: () =>
    new ApiError("MISSING_FIELDS",
      "Missing required fields: name, mode, endpoint",
      "Provide a name, the signing mode (e.g. nitro), and the endpoint URL."),

  // Auth
  noToken: () =>
    new ApiError("NO_TOKEN",
      "No token provided",
      "Include a Bearer token in the Authorization header. Log in via POST /api/auth/login to get one.", 401),
  tokenInvalid: () =>
    new ApiError("TOKEN_INVALID",
      "Invalid or expired token",
      "Your session has expired. Log in again to get a fresh token.", 401),
  sessionInvalid: () =>
    new ApiError("SESSION_INVALID",
      "Session expired or invalid",
      "Your session was invalidated (password change or admin action). Please log in again.", 401),
  invalidCredentials: () =>
    new ApiError("INVALID_CREDENTIALS",
      "Invalid credentials",
      "Username or password is incorrect. Default credentials are admin/admin if this is a fresh install.", 401),
  notAuthenticated: () =>
    new ApiError("NOT_AUTHENTICATED",
      "Not authenticated",
      "You must be logged in to access this resource.", 401),
  credentialsRequired: () =>
    new ApiError("CREDENTIALS_REQUIRED",
      "Username and password are required",
      "Both username and password must be provided."),
  passwordRequired: () =>
    new ApiError("PASSWORD_REQUIRED",
      "Current and new password are required",
      "Provide both your current password and the new password."),
  adminRequired: () =>
    new ApiError("ADMIN_REQUIRED",
      "Admin access required",
      "This action requires administrator privileges.", 403),
  setupCompleted: () =>
    new ApiError("SETUP_COMPLETED",
      "Setup already completed. Use /register to create new users.",
      "The initial admin account has already been created. Ask an admin to register additional users.", 403),
  cannotDeleteSelf: () =>
    new ApiError("CANNOT_DELETE_SELF",
      "Cannot delete your own account",
      "Ask another admin to delete your account, or deactivate it instead."),

  // Ports
  portConflictNode: (port: number, name: string) =>
    new ApiError("PORT_CONFLICT_NODE",
      `Port ${port} (${name}) is already in use by another node`,
      "Another managed node is using this port. Let NeoNexus auto-assign ports, or pick a different range."),
  portConflictSystem: (port: number, name: string) =>
    new ApiError("PORT_CONFLICT_SYSTEM",
      `Port ${port} (${name}) is already in use by another process`,
      `A process outside NeoNexus is binding this port. Run \`lsof -i :${port}\` to identify it.`),
  noPortRange: () =>
    new ApiError("NO_PORT_RANGE",
      "No available port range found",
      "All port slots are taken (max 100 nodes). Delete unused nodes to free up port ranges."),

  // Plugins
  pluginsCliOnly: () =>
    new ApiError("PLUGINS_CLI_ONLY",
      "Plugins are only supported for neo-cli nodes",
      "neo-go has built-in equivalents for most plugins. Check the neo-go documentation for the feature you need."),

  // Servers
  serverFieldsRequired: () =>
    new ApiError("MISSING_FIELDS",
      "Missing required fields: name, baseUrl",
      "Provide a display name and the base URL of the remote NeoNexus instance."),

  // Generic
  notFound: (resource: string) =>
    new ApiError("NOT_FOUND",
      `${resource} not found`,
      `The requested ${resource.toLowerCase()} does not exist.`, 404),

  // System
  snapshotRequired: () =>
    new ApiError("SNAPSHOT_REQUIRED",
      "A valid snapshot payload is required",
      "POST a JSON body containing the configuration snapshot from a previous export."),
};
