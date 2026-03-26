#!/bin/bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${INSTALL_DIR:-$HOME/.neonexus}"
SERVICE_USER="${USER}"
PORT="${PORT:-8080}"

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                                                           ║"
echo "║              NeoNexus Node Manager Installer              ║"
echo "║                   Version 2.0.0                           ║"
echo "║                                                           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# Check OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    echo -e "${RED}Error: Unsupported operating system. This installer supports Linux and macOS.${NC}"
    exit 1
fi

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" && "$ARCH" != "arm64" ]]; then
    echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
    exit 1
fi

# Check dependencies
echo -e "${BLUE}Checking dependencies...${NC}"

# Check Node.js
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}Node.js is not installed. Installing...${NC}"
    
    if [[ "$OS" == "linux" ]]; then
        # Install Node.js 20.x
        curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
        sudo apt-get install -y nodejs
    else
        echo -e "${RED}Please install Node.js 20 or later manually: https://nodejs.org/${NC}"
        exit 1
    fi
fi

NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
if [[ "$NODE_VERSION" -lt 20 ]]; then
    echo -e "${RED}Error: Node.js 20 or later is required. Found: $(node --version)${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Node.js $(node --version)${NC}"

# Check git
if ! command -v git &> /dev/null; then
    echo -e "${YELLOW}Git is not installed. Installing...${NC}"
    if [[ "$OS" == "linux" ]]; then
        sudo apt-get update && sudo apt-get install -y git
    else
        echo -e "${RED}Please install Git manually${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ Git${NC}"

# Check for .NET runtime (for neo-cli)
if command -v dotnet &> /dev/null; then
    echo -e "${GREEN}✓ .NET Runtime ($(dotnet --version))${NC}"
else
    echo -e "${YELLOW}⚠ .NET Runtime not found. It will be required for neo-cli nodes.${NC}"
    echo "  Install it later with: sudo apt-get install -y dotnet-runtime-8.0"
fi

echo ""
echo -e "${BLUE}Installing NeoNexus...${NC}"

# Create installation directory
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

# Clone or update repository
if [[ -d ".git" ]]; then
    echo "Updating existing installation..."
    git pull origin main
else
    echo "Cloning repository..."
    git clone https://github.com/r3e-network/neonexus.git . --depth 1
fi

# Install dependencies
echo "Installing dependencies..."
npm install

# Build the project
echo "Building..."
npm run build

# Create systemd service (Linux only)
if [[ "$OS" == "linux" ]] && command -v systemctl &> /dev/null; then
    echo "Creating systemd service..."
    
    SERVICE_FILE="/etc/systemd/system/neonexus.service"
    
    sudo tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=NeoNexus Node Manager
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$(which node) --import tsx $INSTALL_DIR/dist/index.js
Restart=on-failure
RestartSec=10
Environment=NODE_ENV=production
Environment=PORT=$PORT

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable neonexus
    
    echo -e "${GREEN}✓ Systemd service created${NC}"
    echo "  Start: sudo systemctl start neonexus"
    echo "  Stop:  sudo systemctl stop neonexus"
    echo "  Logs:  sudo journalctl -u neonexus -f"
fi

# Create launcher script
cat > "$INSTALL_DIR/start.sh" <<'EOF'
#!/bin/bash
cd "$(dirname "$0")"
export NODE_ENV=production
export PORT=${PORT:-8080}
node --import tsx dist/index.js
EOF
chmod +x "$INSTALL_DIR/start.sh"

# Create desktop entry (if display is available)
if [[ -n "$DISPLAY" ]] && [[ -d "$HOME/.local/share/applications" ]]; then
    cat > "$HOME/.local/share/applications/neonexus.desktop" <<EOF
[Desktop Entry]
Name=NeoNexus Node Manager
Comment=Manage Neo N3 nodes
Exec=xdg-open http://localhost:$PORT
Type=Application
Icon=$INSTALL_DIR/web/dist/vite.svg
Categories=Network;Blockchain;
EOF
    echo -e "${GREEN}✓ Desktop entry created${NC}"
fi

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}              Installation Complete!                       ${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}Installation directory:${NC} $INSTALL_DIR"
echo -e "${BLUE}Web interface:${NC} http://localhost:$PORT"
echo ""
echo "Usage:"
echo "  cd $INSTALL_DIR"
echo "  ./start.sh              # Start NeoNexus"
echo "  npm run dev             # Start in development mode"
echo ""

if command -v systemctl &> /dev/null; then
    echo "Systemd service commands:"
    echo "  sudo systemctl start neonexus   # Start service"
    echo "  sudo systemctl stop neonexus    # Stop service"
    echo "  sudo systemctl status neonexus  # Check status"
    echo ""
    echo -e "${YELLOW}To start the service now, run:${NC}"
    echo "  sudo systemctl start neonexus"
else
    echo -e "${YELLOW}To start NeoNexus now, run:${NC}"
    echo "  cd $INSTALL_DIR && ./start.sh"
fi

echo ""
echo -e "${BLUE}For support, visit: https://github.com/r3e-network/neonexus${NC}"
