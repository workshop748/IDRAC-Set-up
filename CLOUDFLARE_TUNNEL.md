# Cloudflare Tunnel Setup for iDRAC Controller

This guide will help you expose your iDRAC Controller application to the internet securely using Cloudflare Tunnel (formerly Argo Tunnel) on Linux/Proxmox.

## Prerequisites

1. A Cloudflare account (free tier works)
2. A domain managed by Cloudflare
3. Docker and Docker Compose installed on your Proxmox/Linux server
4. SSH access to your server

## Step 1: Install Docker (if not already installed)

On your Proxmox VM or Linux server:

```bash
# Update package list
sudo apt update

# Install Docker
sudo apt install -y docker.io docker-compose

# Enable Docker to start on boot
sudo systemctl enable docker
sudo systemctl start docker

# Add your user to docker group (optional, to run docker without sudo)
sudo usermod -aG docker $USER
# Log out and back in for this to take effect
```

## Step 2: Install Cloudflared

### Option A: Using Docker (Recommended)

Add the cloudflared service to your `docker-compose.yml`:

```yaml
version: '3.8'

services:
  idrac-controller:
    build: .
    container_name: idrac-controller
    ports:
      - "8080:8080"
    environment:
      - IDRAC_HOST=${IDRAC_HOST:-https://192.168.1.100}
      - IDRAC_USERNAME=${IDRAC_USERNAME:-root}
      - IDRAC_PASSWORD=${IDRAC_PASSWORD:-calvin}
      - RUST_LOG=info
      - DATABASE_PATH=/data/idrac.db
    volumes:
      - ./data:/data
    restart: unless-stopped
    networks:
      - idrac-network

  cloudflared:
    image: cloudflare/cloudflared:latest
    container_name: cloudflared-tunnel
    command: tunnel --no-autoupdate run
    environment:
      - TUNNEL_TOKEN=${TUNNEL_TOKEN}
    restart: unless-stopped
    networks:
      - idrac-network

networks:
  idrac-network:
    driver: bridge
```

### Option B: Install Locally on Linux

```bash
# Download cloudflared for Linux
wget https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64

# Make it executable
chmod +x cloudflared-linux-amd64

# Move to system path
sudo mv cloudflared-linux-amd64 /usr/local/bin/cloudflared

# Verify installation
cloudflared --version
```

## Step 3: Authenticate with Cloudflare

```bash
This will output a URL. Copy it and open it on any device with a browser to:
1. Log in to your Cloudflare account
2. Select the domain you want to use
3. Authorize the tunnel

## Step 4: Create a Tunnel

```bash
cloudflared tunnel create idrac-controller
```

This creates a tunnel and generates:
- A tunnel UUID (save this!)
- A credentials file at `~/.cloudflared/[TUNNEL-UUID].json`

Example output:
```
Tunnel credentials written to /root/.cloudflared/12345678-1234-1234-1234-123456789abc.json
```

## Step 5: Configure the Tunnel (Optional - for local cloudflared)

If you're running cloudflared locally (not in Docker), create configuration:

```bash
# Create cloudflared directory if it doesn't exist
mkdir -p ~/.cloudflared

# Create config file
nano ~/.cloudflared/config.yml
```

Add this configuration:

```yaml
tunnel: YOUR-TUNNEL-UUID
credentials-file: /root/.cloudflared/YOUR-TUNNEL-UUID.json

ingress:
  - hostname: idrac.yourdomain.com
    service: http://localhost:8080
  - service: http_status:404
```
Or manually in Cloudflare Dashboard:
1. Go to DNS settings
2. Add a CNAME record:
   - Name: `idrac` (or your subdomain)
   - Target: `YOUR-TUNNEL-UUID.cfargotunnel.com`
   - Proxy status: Proxied (orange cloud)

## Step 7: Deploy on Proxmox/Linux

### Method 1: Using Docker Compose (Recommended)

1. Create your project directory on the server:
   ```bash
   mkdir -p ~/idrac-controller
   cd ~/idrac-controller
   ```

2. Upload your project files or clone from git

3. Get your tunnel token from Cloudflare Dashboard:
   - Go to Zero Trust > Access > Tunnels
   - Click on your tunnel
   - Click "Configure"
   - Copy the token from the install command

4. Create `.env` file:
   ```bash
   nano .env
   ```

   Add your configuration:
   ```env
   IDRAC_HOST=https://192.168.1.100
   IDRAC_USERNAME=root
   IDRAC_PASSWORD=your-idrac-password
   TUNNEL_TOKEN=your-cloudflare-tunnel-token
   RUST_LOG=info
   DATABASE_PATH=/data/idrac.db
   ```

5. Start the services:
   ```bash
   docker-compose up -d
   ```

6. Check logs to verify everything is running:
   ```bash
   docker-compose logs -f
   ```

### Method 2: Using Local Cloudflared Service

1. Start the iDRAC controller:
   ```bash
   docker-compose up -d idrac-controller
   ```

2. Run cloudflared as a service:
   ```bash
   # Test the tunnel first
   cloudflared tunnel run idrac-controller
   
   # If successful, install as a systemd service
   sudo cloudflared service install
   sudo systemctl start cloudflared
   sudo systemctl enable cloudflared
   ```

3. Check service status:
   ```bash
   sudo systemctl status cloudflared
   ```

## Step 8: Verify

### Using Local Installation:

```powershell
cloudflared tunnel run idrac-controller
```

Or install as a Windows service:

```powershell
cloudflared service install
cloudflared service start
```

## Step 7: Verify

Visit your domain: `https://idrac.yourdomain.com`

You should see the iDRAC Controller login page!
## Troubleshooting

### Tunnel won't connect
```bash
# Check tunnel status
cloudflared tunnel info idrac-controller

# Check Docker logs
docker logs cloudflared-tunnel
docker logs idrac-controller

# Check systemd service logs (if using local cloudflared)
sudo journalctl -u cloudflared -f
```

### 502 Bad Gateway
```bash
# Ensure containers are running
docker ps

# Check if app is responding locally
curl http://localhost:8080

# For Docker setup: verify network connectivity
docker network inspect rusttest_idrac-network
```

**Common fixes:**
- Ensure the iDRAC Controller is running: `docker ps`
- Check the service URL in config.yml matches your app
- For Docker: use `http://idrac-controller:8080` instead of `localhost:8080`
- Verify both containers are on the same network

### DNS not resolving
```bash
# Check DNS resolution
nslookup idrac.yourdomain.com
dig idrac.yourdomain.com

# Force DNS refresh
sudo systemd-resolve --flush-caches
```

### Permission issues
```bash
# Fix data directory permissions
sudo chown -R 1000:1000 ./data

# Or run with current user
sudo chown -R $USER:$USER ./data
```

### Container won't start
```bash
# Check detailed logs
## Proxmox-Specific Tips

### Running in a LXC Container

1. **Create a privileged LXC container** (required for Docker):
   - In Proxmox web UI: Create CT
   - Choose Ubuntu 22.04 or Debian 12
   - Enable "Nesting" and "keyctl" features
   - Allocate at least 2GB RAM and 2 CPU cores

2. **Configure LXC for Docker:**
   ```bash
   # Edit LXC config on Proxmox host
   nano /etc/pve/lxc/YOUR-CTID.conf
   
   # Add these lines:
   lxc.apparmor.profile: unconfined
   lxc.cgroup.devices.allow: a
   lxc.cap.drop:
   ```

3. **Install Docker in LXC:**
   ```bash
   # Inside the container
   curl -fsSL https://get.docker.com -o get-docker.sh
   sh get-docker.sh
   ```

### Running in a VM (Recommended for Production)

1. Create a VM with Ubuntu Server 22.04 LTS
2. Allocate resources: 2GB RAM, 2 CPUs, 20GB disk
3. Follow the standard Docker installation steps above

### Firewall Configuration

If using Proxmox firewall:
```bash
# On Proxmox host - allow traffic to VM/CT
# Datacenter > Firewall > Add rule
# Direction: in
# Action: ACCEPT
# Protocol: tcp
# Dest. port: 8080
# Comment: iDRAC Controller
```

## Advanced: Complete Docker Compose Setup

Full `docker-compose.yml` example for Proxmox/Linux:
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```
1. Go to Security > WAF
2. Enable managed rules
3. Consider rate limiting

### 3. Use Strong Passwords

Change the default admin password immediately:
1. Log in with `admin` / `5PmKySnn5fgsfb`
2. (Note: Password change functionality would need to be added to the app)

## Troubleshooting

### Tunnel won't connect
```powershell
# Check tunnel status
cloudflared tunnel info idrac-controller

# Check logs
docker logs cloudflared-tunnel
# or
Get-Content C:\Users\YOUR-USERNAME\.cloudflared\cloudflared.log -Tail 50
```

### 502 Bad Gateway
- Ensure the iDRAC Controller is running: `docker ps`
- Check the service URL in config.yml matches your app
Update your `.env` file:
```env
IDRAC_HOST=https://192.168.1.100
IDRAC_USERNAME=root
IDRAC_PASSWORD=your-password
TUNNEL_TOKEN=your-cloudflare-tunnel-token
RUST_LOG=info
DATABASE_PATH=/data/idrac.db
```

## Maintenance Commands

```bash
# View logs
docker-compose logs -f

# Restart services
docker-compose restart

# Update and rebuild
docker-compose down
docker-compose pull
docker-compose build --no-cache
docker-compose up -d

# Backup database
docker-compose exec idrac-controller cp /data/idrac.db /data/idrac.db.backup
# Or from host
cp ./data/idrac.db ./data/idrac.db.backup

# Check resource usage
docker stats
```

## Auto-start on Proxmox Boot

### For VM/LXC with Docker Compose:

Create a systemd service:

```bash
sudo nano /etc/systemd/system/idrac-controller.service
```

Add:
```ini
[Unit]
Description=iDRAC Controller
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/root/idrac-controller
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
```

Enable the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable idrac-controller
sudo systemctl start idrac-controller
```

## Resources

- [Cloudflare Tunnel Documentation](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/)
- [Cloudflare Zero Trust](https://developers.cloudflare.com/cloudflare-one/)
- [Cloudflared GitHub](https://github.com/cloudflare/cloudflared)
- [Proxmox Documentation](https://pve.proxmox.com/wiki/Main_Page)
- [Docker on Proxmox LXC](https://pve.proxmox.com/wiki/Linux_Container#pct_container_storage)

## Support

For issues specific to:
- **Cloudflare Tunnel**: Check Cloudflare Community Forums
- **iDRAC Controller**: Check the application repository
- **Proxmox**: Check Proxmox Forums
      - IDRAC_HOST=${IDRAC_HOST}
      - IDRAC_USERNAME=${IDRAC_USERNAME}
      - IDRAC_PASSWORD=${IDRAC_PASSWORD}
      - RUST_LOG=info
      - DATABASE_PATH=/data/idrac.db
    volumes:
      - ./data:/data
    restart: unless-stopped
    networks:
      - idrac-network

  cloudflared:
    image: cloudflare/cloudflared:latest
    container_name: cloudflared-tunnel
    command: tunnel --no-autoupdate run
    environment:
      - TUNNEL_TOKEN=${TUNNEL_TOKEN}
    restart: unless-stopped
    depends_on:
      - idrac-controller
    networks:
      - idrac-network

networks:
  idrac-network:
    driver: bridge
```

Update your `.env` file:
```env
IDRAC_HOST=https://192.168.1.100
IDRAC_USERNAME=root
IDRAC_PASSWORD=your-password
TUNNEL_TOKEN=your-cloudflare-tunnel-token
```

## Resources

- [Cloudflare Tunnel Documentation](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/)
- [Cloudflare Zero Trust](https://developers.cloudflare.com/cloudflare-one/)
- [Cloudflared GitHub](https://github.com/cloudflare/cloudflared)

## Support

For issues specific to:
- **Cloudflare Tunnel**: Check Cloudflare Community Forums
- **iDRAC Controller**: Check the application repository
