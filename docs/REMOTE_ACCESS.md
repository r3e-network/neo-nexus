# Remote Access Guide

This guide explains how to securely access NeoNexus Node Manager from the public internet when running on a cloud VM.

## Table of Contents

1. [Security Considerations](#security-considerations)
2. [Quick Start (SSH Tunnel)](#quick-start-ssh-tunnel)
3. [Nginx Reverse Proxy with HTTPS](#nginx-reverse-proxy-with-https)
4. [Cloudflare Tunnel](#cloudflare-tunnel)
5. [Firewall Configuration](#firewall-configuration)
6. [Troubleshooting](#troubleshooting)

## Security Considerations

⚠️ **Important:** Before exposing NeoNexus to the internet:

1. **Always use HTTPS** - Encrypt all traffic with TLS
2. **Strong passwords** - Use complex passwords for all accounts
3. **Firewall rules** - Only expose necessary ports
4. **Regular updates** - Keep the system and NeoNexus updated
5. **Consider VPN** - Use a VPN for additional security layer

## Quick Start (SSH Tunnel)

The simplest and most secure method for occasional access:

```bash
# From your local machine, create an SSH tunnel
ssh -L 8080:localhost:8080 user@your-vms-ip

# Then access NeoNexus locally at:
# http://localhost:8080

# The connection goes through your encrypted SSH tunnel
```

For a persistent tunnel:

```bash
# Using autossh for automatic reconnection
autossh -M 0 -N -L 8080:localhost:8080 user@your-vms-ip
```

## Nginx Reverse Proxy with HTTPS

### 1. Install Nginx and Certbot

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y nginx certbot python3-certbot-nginx

# Or use the provided setup script
sudo bash scripts/setup-nginx.sh
```

### 2. Obtain SSL Certificate

```bash
# Using Let's Encrypt (requires domain pointing to your server)
sudo certbot certonly --standalone -d your-domain.com

# Or use DNS challenge if port 80 is blocked
sudo certbot certonly --manual --preferred-challenges dns -d your-domain.com
```

### 3. Configure Nginx

Copy the example configuration:

```bash
sudo cp docs/nginx-example.conf /etc/nginx/sites-available/neonexus
sudo ln -s /etc/nginx/sites-available/neonexus /etc/nginx/sites-enabled/
```

Edit `/etc/nginx/sites-available/neonexus`:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    # SSL Certificates
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # SSL Security
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        proxy_read_timeout 86400;
    }

    location /api/auth/login {
        limit_req zone=login burst=3 nodelay;
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 4. Test and Reload Nginx

```bash
sudo nginx -t
sudo systemctl reload nginx
```

### 5. Access NeoNexus

Visit `https://your-domain.com`

## Cloudflare Tunnel

For servers behind NAT or without a public IP:

### 1. Install Cloudflare Tunnel

```bash
# Download and install cloudflared
curl -L --output cloudflared.deb https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb
sudo dpkg -i cloudflared.deb
```

### 2. Authenticate

```bash
cloudflared tunnel login
# Follow the link to authorize with your Cloudflare account
```

### 3. Create and Configure Tunnel

```bash
# Create a tunnel
cloudflared tunnel create neonexus

# Get the tunnel ID
cloudflared tunnel list

# Create config file
mkdir -p ~/.cloudflared
cat > ~/.cloudflared/config.yml << EOF
tunnel: YOUR_TUNNEL_ID
credentials-file: /home/YOUR_USER/.cloudflared/YOUR_TUNNEL_ID.json

ingress:
  - hostname: neonexus.yourdomain.com
    service: http://localhost:8080
  - service: http_status:404
EOF

# Add DNS record
cloudflared tunnel route dns neonexus neonexus.yourdomain.com
```

### 4. Run the Tunnel

```bash
# Run manually for testing
cloudflared tunnel run neonexus

# Or install as a service
sudo cloudflared service install
sudo systemctl start cloudflared
```

## Firewall Configuration

### UFW (Ubuntu/Debian)

```bash
# Default deny incoming
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow ssh

# Option 1: Only allow access through Nginx (port 80/443)
sudo ufw allow 'Nginx Full'

# Option 2: Direct access (not recommended for production)
# sudo ufw allow 8080/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

### Cloud Provider Security Groups

**AWS Security Group:**
- Inbound: TCP 22 (SSH) from your IP
- Inbound: TCP 80 (HTTP) from anywhere
- Inbound: TCP 443 (HTTPS) from anywhere
- Outbound: All traffic

**DigitalOcean Firewall:**
- Type: SSH, Protocol: TCP, Port: 22, Sources: Your IP
- Type: HTTP, Protocol: TCP, Port: 80, Sources: All IPv4, All IPv6
- Type: HTTPS, Protocol: TCP, Port: 443, Sources: All IPv4, All IPv6

**Hetzner Firewall:**
- TCP 22 from your IP
- TCP 80 from anywhere
- TCP 443 from anywhere

## Troubleshooting

### Cannot Access from Browser

1. Check NeoNexus is running:
   ```bash
   sudo systemctl status neonexus
   ```

2. Check it's listening on the correct interface:
   ```bash
   sudo netstat -tlnp | grep 8080
   # Should show 0.0.0.0:8080 or your public IP
   ```

3. Check firewall rules:
   ```bash
   sudo ufw status
   sudo iptables -L -n | grep 8080
   ```

4. Test locally:
   ```bash
   curl http://localhost:8080/api/health
   ```

### HTTPS Not Working

1. Check certificate:
   ```bash
   sudo certbot certificates
   ```

2. Renew certificate:
   ```bash
   sudo certbot renew --dry-run
   ```

3. Check Nginx error logs:
   ```bash
   sudo tail -f /var/log/nginx/error.log
   ```

### WebSocket Connection Issues

If real-time updates don't work:

1. Ensure Nginx WebSocket proxy headers are set:
   ```nginx
   proxy_set_header Upgrade $http_upgrade;
   proxy_set_header Connection "upgrade";
   ```

2. Check browser console for WebSocket errors

3. Test WebSocket directly:
   ```bash
   wss://your-domain.com/ws
   ```

## Security Checklist

Before going to production:

- [ ] HTTPS enabled with valid SSL certificate
- [ ] Strong admin password (16+ characters)
- [ ] Firewall configured to block port 8080 from public
- [ ] Nginx rate limiting configured
- [ ] Regular system updates scheduled
- [ ] Backups configured
- [ ] Monitoring enabled
- [ ] Fail2ban installed and configured

```bash
# Install fail2ban for additional security
sudo apt install -y fail2ban

# Create custom jail for NeoNexus
sudo tee /etc/fail2ban/jail.local << EOF
[neonexus]
enabled = true
port = http,https
filter = neonexus
logpath = /var/log/nginx/access.log
maxretry = 5
bantime = 3600
EOF
```

## Recommended Setup

For most users, we recommend this setup:

1. **NeoNexus** running on localhost:8080 (not exposed to internet)
2. **Nginx** as reverse proxy with HTTPS on ports 80/443
3. **UFW firewall** blocking direct access to 8080
4. **Fail2ban** protecting against brute force
5. **Regular backups** of ~/.neonexus directory

This provides the best balance of security and usability.
