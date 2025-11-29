# iDRAC Server Controller

A secure web application written in Rust that provides remote power management for Dell iDRAC servers through a clean web interface. Features user authentication with first-run setup and Docker containerization.

## Features

- 🔐 **Secure Authentication**: First-run account creation with bcrypt password hashing
- 🖥️ **iDRAC Integration**: Control server power states via Dell iDRAC Redfish API
- 🐳 **Docker Ready**: Complete containerization with Docker and docker-compose
- 🎨 **Modern UI**: Responsive web interface with real-time status updates
- 💾 **Persistent Storage**: SQLite database for user management
- 🔒 **Session Management**: Secure cookie-based sessions with 24-hour persistence

## Power Control Features

- **Power On**: Turn on the server
- **Force Power Off**: Immediately power off the server
- **Graceful Shutdown**: Safely shutdown the operating system
- **Status Monitoring**: Real-time power state display with auto-refresh

## Prerequisites

- Docker and Docker Compose
- Dell iDRAC 8 or newer with Redfish API support
- iDRAC IP address and credentials

## Quick Start

### 1. Clone or create the project

All files should be in the `RustTest` directory.

### 2. Configure Environment Variables

Create a `.env` file in the project root:

```env
IDRAC_HOST=https://192.168.1.100
IDRAC_USERNAME=root
IDRAC_PASSWORD=your-idrac-password
```

**Important**: Replace with your actual iDRAC details.

### 3. Update docker-compose.yml

Edit `docker-compose.yml` and set your iDRAC host:

```yaml
environment:
  - IDRAC_HOST=https://192.168.1.100
  - IDRAC_USERNAME=root
  - IDRAC_PASSWORD=${IDRAC_PASSWORD}
```

### 4. Build and Run

```bash
# Build the Docker image
docker-compose build

# Start the container
docker-compose up -d

# View logs
docker-compose logs -f
```

### 5. Access the Application

Open your browser and navigate to:
```
http://localhost:8080
```

On first run, you'll be prompted to create an administrator account.

## Project Structure

```
RustTest/
├── src/
│   ├── main.rs          # Application entry point and server setup
│   ├── database.rs      # SQLite database and user management
│   ├── idrac.rs         # iDRAC API client implementation
│   └── handlers.rs      # HTTP request handlers
├── static/
│   ├── register.html    # First-run registration page
│   ├── login.html       # User login page
│   └── dashboard.html   # Main control dashboard
├── data/                # Database storage (created automatically)
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Multi-stage Docker build
├── docker-compose.yml   # Docker Compose configuration
└── README.md           # This file
```

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `IDRAC_HOST` | iDRAC base URL (https://ip) | - | Yes |
| `IDRAC_USERNAME` | iDRAC username | - | Yes |
| `IDRAC_PASSWORD` | iDRAC password | - | Yes |
| `DATABASE_PATH` | SQLite database file path | `/data/idrac.db` | No |
| `RUST_LOG` | Logging level | `info` | No |

## API Endpoints

### Authentication
- `GET /` - Main page (redirects based on auth state)
- `POST /api/register` - Create first user account
- `POST /api/login` - User login
- `POST /api/logout` - User logout

### Power Control (Authenticated)
- `GET /api/power/status` - Get current power state
- `POST /api/power/on` - Power on the server
- `POST /api/power/off` - Force power off
- `POST /api/power/shutdown` - Graceful shutdown

## Security Features

- **Password Hashing**: Bcrypt with default cost factor
- **Session Security**: HTTP-only cookies with 24-hour expiration
- **First-Run Only**: Registration is only available when no users exist
- **HTTPS Support**: Built-in TLS verification bypass for self-signed iDRAC certificates
- **Authentication Checks**: All power control endpoints require valid session

## Building Without Docker

If you prefer to run without Docker:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Set environment variables
export IDRAC_HOST=https://192.168.1.100
export IDRAC_USERNAME=root
export IDRAC_PASSWORD=your-password
export DATABASE_PATH=./data/idrac.db

# Create data directory
mkdir -p data

# Build and run
cargo build --release
./target/release/idrac-controller
```

## Troubleshooting

### Cannot connect to iDRAC
- Verify iDRAC IP address is correct
- Ensure iDRAC web interface is accessible
- Check firewall rules allow HTTPS (443) to iDRAC
- Verify credentials are correct

### Database errors
- Check that `/data` directory has write permissions
- Verify `DATABASE_PATH` environment variable is set correctly
- Check Docker volume mount is configured properly

### Authentication issues
- Clear browser cookies for the site
- Check session middleware is configured correctly
- Verify database is accessible and initialized

### Docker build fails
- Ensure you have enough disk space
- Check Docker daemon is running
- Try clearing Docker cache: `docker system prune -a`

## Development

### Running in Development Mode

```bash
# Install dependencies
cargo build

# Run with hot reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Run tests
cargo test
```

### Environment Setup

For local development, create a `.env` file and use a tool like `dotenv`:

```bash
# Add to Cargo.toml
[dependencies]
dotenv = "0.15"

# In main.rs
dotenv::dotenv().ok();
```

## iDRAC API Reference

This application uses the Dell Redfish API:
- **Redfish API Version**: 1.0+
- **Required iDRAC Version**: 8 or newer
- **Documentation**: https://www.dell.com/support/manuals/en-us/idrac9-lifecycle-controller-v3.x-series/idrac_3.00.00.00_redfishapiguide/

## License

This project is provided as-is for educational and personal use.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## Acknowledgments

- Built with [Actix-web](https://actix.rs/) - Fast, pragmatic web framework for Rust
- Uses Dell iDRAC Redfish API for server management
- Database powered by SQLite via [rusqlite](https://github.com/rusqlite/rusqlite)

## Support

For issues, questions, or contributions, please open an issue on the project repository.
