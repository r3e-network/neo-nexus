#!/usr/bin/env bash

set -Eeuo pipefail

if [[ -t 1 ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  RED=''
  GREEN=''
  YELLOW=''
  BLUE=''
  BOLD=''
  NC=''
fi

SERVICE_NAME="${SERVICE_NAME:-neonexus}"
REPO_URL="${REPO_URL:-https://github.com/r3e-network/neo-nexus.git}"
BRANCH="${BRANCH:-main}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.neonexus/app}"
DATA_DIR="${NEONEXUS_DATA_DIR:-${DATA_DIR:-$HOME/.neonexus/data}}"
ENV_FILE="$INSTALL_DIR/.env"
HOST="${HOST:-0.0.0.0}"
PORT="${PORT:-8080}"
INSTALL_SYSTEMD="${INSTALL_SYSTEMD:-true}"

INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"
DATA_DIR="${DATA_DIR/#\~/$HOME}"

if [[ ${EUID:-$(id -u)} -eq 0 && -n "${SUDO_USER:-}" ]]; then
  SERVICE_USER="${SERVICE_USER:-$SUDO_USER}"
else
  SERVICE_USER="${SERVICE_USER:-$(id -un)}"
fi

info() {
  printf '%b\n' "${BLUE}$*${NC}"
}

success() {
  printf '%b\n' "${GREEN}$*${NC}"
}

warn() {
  printf '%b\n' "${YELLOW}$*${NC}"
}

fail() {
  printf '%b\n' "${RED}Error: $*${NC}" >&2
  exit 1
}

run_as_root() {
  if [[ ${EUID:-$(id -u)} -eq 0 ]]; then
    "$@"
    return
  fi

  command -v sudo >/dev/null 2>&1 || fail "sudo is required to install system packages or systemd services"
  sudo "$@"
}

install_apt_packages() {
  command -v apt-get >/dev/null 2>&1 || return 1
  run_as_root apt-get update
  run_as_root apt-get install -y "$@"
}

detect_os() {
  case "$(uname -s)" in
    Linux*) echo "linux" ;;
    Darwin*) echo "macos" ;;
    *) fail "Unsupported operating system. Linux and macOS are supported." ;;
  esac
}

check_architecture() {
  case "$(uname -m)" in
    x86_64|amd64|arm64|aarch64) ;;
    *) fail "Unsupported architecture: $(uname -m)" ;;
  esac
}

node_major_version() {
  node --version | sed -E 's/^v([0-9]+).*/\1/'
}

ensure_git() {
  if command -v git >/dev/null 2>&1; then
    success "git: $(git --version | awk '{print $3}')"
    return
  fi

  if [[ "$OS" == "linux" ]] && install_apt_packages git; then
    success "git installed"
    return
  fi

  fail "Git is required. Install Git and run this installer again."
}

install_node_with_nodesource() {
  command -v apt-get >/dev/null 2>&1 || fail "Automatic Node.js installation supports Debian/Ubuntu. Install Node.js 20+ manually on this Linux distribution."
  install_apt_packages curl ca-certificates gnupg
  local setup_script
  setup_script="$(mktemp)"
  curl -fsSL https://deb.nodesource.com/setup_20.x -o "$setup_script"
  run_as_root bash "$setup_script"
  rm -f "$setup_script"
  install_apt_packages nodejs
}

ensure_node() {
  if command -v node >/dev/null 2>&1 && [[ "$(node_major_version)" -ge 20 ]]; then
    success "Node.js: $(node --version)"
  elif [[ "$OS" == "linux" && "${AUTO_INSTALL_NODE:-true}" == "true" ]]; then
    warn "Node.js 20+ was not found. Installing Node.js 20 with NodeSource..."
    install_node_with_nodesource
    [[ "$(node_major_version)" -ge 20 ]] || fail "Node.js 20+ installation did not complete"
    success "Node.js: $(node --version)"
  elif [[ "$OS" == "macos" ]]; then
    fail "Node.js 20+ is required. Install it with 'brew install node' or from https://nodejs.org/"
  else
    fail "Node.js 20+ is required"
  fi

  command -v npm >/dev/null 2>&1 || fail "npm is required with Node.js"
  success "npm: $(npm --version)"
}

check_optional_dotnet() {
  if command -v dotnet >/dev/null 2>&1; then
    success ".NET: $(dotnet --version)"
  else
    warn ".NET runtime was not found. Install .NET 10+ before deploying neo-cli nodes."
  fi
}

clone_or_update_repo() {
  mkdir -p "$(dirname "$INSTALL_DIR")"

  if [[ -d "$INSTALL_DIR/.git" ]]; then
    info "Updating existing installation in $INSTALL_DIR..."
    git -C "$INSTALL_DIR" remote set-url origin "$REPO_URL"
    git -C "$INSTALL_DIR" fetch --depth 1 origin "$BRANCH"
    git -C "$INSTALL_DIR" checkout "$BRANCH"
    git -C "$INSTALL_DIR" pull --ff-only origin "$BRANCH"
    return
  fi

  if [[ -d "$INSTALL_DIR" ]] && [[ -n "$(find "$INSTALL_DIR" -mindepth 1 -maxdepth 1 2>/dev/null)" ]]; then
    fail "$INSTALL_DIR already exists and is not a Git checkout. Set INSTALL_DIR to an empty directory."
  fi

  info "Cloning $REPO_URL..."
  git clone --depth 1 "$REPO_URL" "$INSTALL_DIR" --branch "$BRANCH"
}

install_dependencies_and_build() {
  cd "$INSTALL_DIR"

  info "Installing backend dependencies from package-lock.json..."
  npm ci

  info "Installing frontend dependencies from web/package-lock.json..."
  npm --prefix web ci

  info "Building backend and frontend..."
  npm run build
}

generate_secret() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 32
  else
    node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"
  fi
}

escape_env_value() {
  printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\$/\\$/g' -e 's/`/\\`/g'
}

format_env_line() {
  printf '%s="%s"' "$1" "$(escape_env_value "$2")"
}

upsert_env_value() {
  local key="$1"
  local value="$2"
  local line
  local tmp_file

  line="$(format_env_line "$key" "$value")"
  tmp_file="$(mktemp)"

  if [[ -f "$ENV_FILE" ]]; then
    awk -v key="$key" -v line="$line" '
      BEGIN { done = 0 }
      $0 ~ "^" key "=" {
        if (!done) {
          print line
          done = 1
        }
        next
      }
      { print }
      END {
        if (!done) {
          print line
        }
      }
    ' "$ENV_FILE" > "$tmp_file"
  else
    printf '%s\n' "$line" > "$tmp_file"
  fi

  mv "$tmp_file" "$ENV_FILE"
  chmod 600 "$ENV_FILE"
}

ensure_environment_file() {
  mkdir -p "$DATA_DIR"
  touch "$ENV_FILE"
  chmod 600 "$ENV_FILE"

  upsert_env_value "NODE_ENV" "production"
  upsert_env_value "HOST" "$HOST"
  upsert_env_value "PORT" "$PORT"
  upsert_env_value "NEONEXUS_DATA_DIR" "$DATA_DIR"

  if [[ -n "${JWT_SECRET:-}" ]]; then
    upsert_env_value "JWT_SECRET" "$JWT_SECRET"
  elif ! grep -qE '^JWT_SECRET="?[^"]+' "$ENV_FILE" 2>/dev/null; then
    upsert_env_value "JWT_SECRET" "$(generate_secret)"
  fi

  success "Environment file: $ENV_FILE"
}

write_launcher() {
  cat > "$INSTALL_DIR/start.sh" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail

cd "$(dirname "$0")"
ENV_FILE="${ENV_FILE:-$(pwd)/.env}"

if [[ -f "$ENV_FILE" ]]; then
  set -a; source "$ENV_FILE"; set +a
fi

export NODE_ENV="${NODE_ENV:-production}"
export PORT="${PORT:-8080}"

exec node --import tsx dist/index.js
EOF
  chmod +x "$INSTALL_DIR/start.sh"
  success "Launcher: $INSTALL_DIR/start.sh"
}

write_systemd_service() {
  [[ "$OS" == "linux" ]] || return
  command -v systemctl >/dev/null 2>&1 || return
  [[ "$INSTALL_SYSTEMD" == "true" ]] || return

  local node_path
  local service_file

  node_path="$(command -v node)"
  service_file="/etc/systemd/system/$SERVICE_NAME.service"

  info "Creating systemd service $SERVICE_NAME..."
  run_as_root tee "$service_file" >/dev/null <<EOF
[Unit]
Description=NeoNexus Node Manager
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$ENV_FILE
Environment=NODE_ENV=production
ExecStart=$node_path --import tsx $INSTALL_DIR/dist/index.js
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

  run_as_root systemctl daemon-reload
  run_as_root systemctl enable "$SERVICE_NAME"
  success "systemd service: $SERVICE_NAME"
}

write_desktop_entry() {
  [[ -n "${DISPLAY:-}" ]] || return
  [[ -d "$HOME/.local/share/applications" ]] || return

  cat > "$HOME/.local/share/applications/neonexus.desktop" <<EOF
[Desktop Entry]
Name=NeoNexus Node Manager
Comment=Manage Neo N3 nodes
Exec=xdg-open http://localhost:$PORT
Type=Application
Icon=$INSTALL_DIR/web/dist/neonexus.svg
Categories=Network;Blockchain;
EOF
  success "Desktop launcher created"
}

print_summary() {
  printf '\n%b\n' "${GREEN}${BOLD}NeoNexus installation is ready.${NC}"
  printf '  Install dir: %s\n' "$INSTALL_DIR"
  printf '  Data dir:    %s\n' "$DATA_DIR"
  printf '  Env file:    %s\n' "$ENV_FILE"
  printf '  Web UI:      http://localhost:%s\n\n' "$PORT"

  if [[ "$OS" == "linux" && "$INSTALL_SYSTEMD" == "true" ]] && command -v systemctl >/dev/null 2>&1; then
    printf 'Start service:\n'
    printf '  sudo systemctl start %s\n' "$SERVICE_NAME"
    printf 'View logs:\n'
    printf '  sudo journalctl -u %s -f\n\n' "$SERVICE_NAME"
  fi

  printf 'Start without systemd:\n'
  printf '  %s/start.sh\n\n' "$INSTALL_DIR"
  printf 'Open http://localhost:%s and create the first admin account.\n' "$PORT"
}

main() {
  printf '%b\n' "${BLUE}${BOLD}NeoNexus one-command installer${NC}"
  printf 'Repository: %s\n' "$REPO_URL"
  printf 'Branch:     %s\n' "$BRANCH"
  printf 'Target:     %s\n\n' "$INSTALL_DIR"

  OS="$(detect_os)"
  check_architecture
  ensure_git
  ensure_node
  check_optional_dotnet
  clone_or_update_repo
  install_dependencies_and_build
  ensure_environment_file
  write_launcher
  write_systemd_service
  write_desktop_entry
  print_summary
}

main "$@"
