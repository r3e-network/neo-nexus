#!/bin/bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}NeoNexus Nginx + HTTPS Setup${NC}"
echo "=============================="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}"
   exit 1
fi

# Get domain name
read -p "Enter your domain name (e.g., neonexus.example.com): " DOMAIN

if [[ -z "$DOMAIN" ]]; then
    echo -e "${RED}Domain name is required${NC}"
    exit 1
fi

# Get email for Let's Encrypt
read -p "Enter your email for Let's Encrypt notifications: " EMAIL

if [[ -z "$EMAIL" ]]; then
    echo -e "${RED}Email is required${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Installing dependencies...${NC}"
apt update
apt install -y nginx certbot python3-certbot-nginx

# Create nginx configuration
echo -e "${BLUE}Creating Nginx configuration...${NC}"
NGINX_CONFIG="/etc/nginx/sites-available/neonexus"

cat > "$NGINX_CONFIG" << EOF
# NeoNexus Nginx Configuration

# Rate limiting zones
limit_req_zone \$binary_remote_addr zone=login:10m rate=10r/m;

# HTTP - Redirect to HTTPS
server {
    listen 80;
    server_name $DOMAIN;
    
    # For Let's Encrypt validation
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }
    
    location / {
        return 301 https://\$server_name\$request_uri;
    }
}

# HTTPS - Main Server Block
server {
    listen 443 ssl http2;
    server_name $DOMAIN;

    # SSL Configuration (will be updated by certbot)
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;
    
    # Modern SSL Configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    
    # Security Headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Proxy to NeoNexus
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection \$connection_upgrade;
        
        proxy_read_timeout 86400s;
        proxy_buffering off;
    }

    # Rate limit login attempts
    location /api/auth/login {
        limit_req zone=login burst=5 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Create symlink
if [[ -f "/etc/nginx/sites-enabled/neonexus" ]]; then
    rm /etc/nginx/sites-enabled/neonexus
fi
ln -s "$NGINX_CONFIG" /etc/nginx/sites-enabled/neonexus

# Remove default site if it exists
if [[ -f "/etc/nginx/sites-enabled/default" ]]; then
    rm /etc/nginx/sites-enabled/default
fi

# Create directory for Let's Encrypt webroot
mkdir -p /var/www/certbot

# Test nginx configuration
echo -e "${BLUE}Testing Nginx configuration...${NC}"
nginx -t

# Reload nginx
echo -e "${BLUE}Reloading Nginx...${NC}"
systemctl reload nginx

# Obtain SSL certificate
echo ""
echo -e "${BLUE}Obtaining SSL certificate from Let's Encrypt...${NC}"
echo -e "${YELLOW}Make sure DNS is pointing to this server!${NC}"
echo ""

certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "$EMAIL" --redirect

# Setup auto-renewal
echo -e "${BLUE}Setting up auto-renewal...${NC}"
SYSTEMD_TIMER=$(systemctl list-timers | grep certbot)
if [[ -z "$SYSTEMD_TIMER" ]]; then
    # Add cron job as fallback
    echo "0 3 * * * certbot renew --quiet --nginx" | crontab -
fi

# Configure firewall
echo -e "${BLUE}Configuring firewall...${NC}"
ufw allow 'Nginx Full' comment 'NeoNexus Web Interface'
ufw delete allow 8080 comment 'NeoNexus Direct Access' 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ Setup complete!${NC}"
echo ""
echo -e "NeoNexus should now be accessible at: ${BLUE}https://$DOMAIN${NC}"
echo ""
echo -e "${YELLOW}Important:${NC}"
echo "1. NeoNexus must be running (npm start or systemd service)"
echo "2. Port 8080 is now blocked from direct access - use Nginx only"
echo "3. SSL certificate will auto-renew"
echo ""
echo -e "${YELLOW}Useful commands:${NC}"
echo "  Check NeoNexus:  sudo systemctl status neonexus"
echo "  Check Nginx:     sudo systemctl status nginx"
echo "  View logs:       sudo journalctl -u neonexus -f"
echo "  Renew SSL:       sudo certbot renew --dry-run"
