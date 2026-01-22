# Hytale Server Mod Manager

**Automatic mod management from CurseForge for your Hytale dedicated server.**

[![CI](https://github.com/optikaaal/hsmm/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/optikaaal/hsmm/actions/workflows/ci.yml)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://github.com/optikaaal/hsmm/pkgs/container/hsmm)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![CurseForge](https://img.shields.io/badge/CurseForge-Hytale-orange)](https://www.curseforge.com/hytale)

---

## 🎯 What is This?

A powerful, production-ready mod manager for Hytale dedicated servers with both **CLI** and **Web UI** interfaces.

### Key Features

- 🌐 **Modern Web UI** - Beautiful React-based dashboard for mod management
- 🚀 **Automatic Updates** - Downloads latest mod versions on server startup
- 📦 **CurseForge Integration** - Browse, search, and install mods directly from CurseForge
- 🧹 **Smart Cleanup** - Archives old versions, configurable retention policy
- 🐳 **Docker Native** - Built for containerized deployment
- ⚡ **High Performance** - Rust-powered with parallel downloads
- 📊 **Server Control** - Restart server, view logs, edit configs from the UI
- 🔒 **Production Ready** - Comprehensive logging, error handling, CI/CD

---

## 📋 Quick Start

### Docker (Recommended)

```bash
docker-compose up -d
```

Access the Web UI at `http://localhost:8080`

### Installation

**Option 1: Download Binary**
```bash
# Download latest release
wget https://github.com/optikaaal/hytale-server-mod-manager/releases/latest/download/hsmm
chmod +x hsmm
sudo mv hsmm /usr/local/bin/
```

**Option 2: Build from Source**
```bash
git clone https://github.com/optikaaal/hytale-server-mod-manager
cd hytale-server-mod-manager
cargo build --release
sudo cp target/release/hsmm /usr/local/bin/
```

**Option 3: Docker**
```bash
docker-compose up -d
```

---

## 🚀 Usage

### 1. Create Configuration

Create `mods.toml`:

```toml
# Hytale game ID (auto-detected, optional)
game_id = 70216

# Add your mods
[[mods]]
name = "EyeSpy"
enabled = true
identifier = { curseforge = 1423494 }

[[mods]]
name = "Advanced Item Info"
enabled = true
identifier = { curseforge = 1409811 }

[settings]
cleanup_old_versions = true    # Remove old versions after upgrade
archive_old_versions = true    # Archive to .old/ (false = permanently delete)
max_old_versions = 3           # Keep 3 most recent backups in .old/
```

### 2. Run Mod Manager

```bash
# Update all mods
hsmm upgrade --config mods.toml --output ./mods

# List configured mods
hsmm list --config mods.toml

# Add a new mod
hsmm add "BetterMap" 1430352 --config mods.toml

# Remove a mod
hsmm remove "BetterMap" --config mods.toml
```

---

## 📖 CLI Commands

```bash
# Update all mods to latest versions
hsmm upgrade

# Add a mod by CurseForge project ID
hsmm add <name> <project-id>

# Remove a mod
hsmm remove <name-or-id>

# List all configured mods
hsmm list

# Show configuration
hsmm config

# Help
hsmm --help
```

### Options

```
-c, --config <FILE>   Configuration file [default: mods.toml]
-o, --output <DIR>    Mods output directory [default: mods]
-l, --log <FILE>      Log file path (optional)
-v, --verbose         Verbose logging
```

---

## 🌐 Web UI

The Web UI provides a complete management interface for your Hytale server mods.

### Features

- **Browse Mods** - Search and discover mods from CurseForge
  - Filter by popularity, newest, alphabetical
  - View mod details, screenshots, and files
  - One-click installation
 
<img width="1784" height="1069" alt="image" src="https://github.com/user-attachments/assets/2ef8842f-94d7-414a-aaad-77e998c6a3bb" />

- **Installed Mods** - Manage your mod collection
  - Enable/disable mods
  - Bulk operations
  - View installation status and versions
    
<img width="1772" height="1067" alt="image" src="https://github.com/user-attachments/assets/8b84d176-7683-45c5-a533-a653e0e6b82a" />

- **Server Control** - Manage your server
  - Restart server with mod updates
  - Check server status
  - One-click deployment
    
<img width="1773" height="1065" alt="image" src="https://github.com/user-attachments/assets/9cf995cd-1260-44b8-baa4-9585b680ee72" />

- **Config Editor** - Edit server configuration files
  - config.json, permissions.json, bans.json, whitelist.json
  - JSON validation and pretty printing
 
<img width="1783" height="1063" alt="image" src="https://github.com/user-attachments/assets/bcd87300-af3e-4205-8e3b-40af642bf283" />

- **Log Viewer** - Monitor server activity
  - Server logs (latest.log)
  - Mod manager logs (mod-manager.log)
  - Web UI access logs (web-ui.log)
  - Auto-refresh support

<img width="1774" height="1066" alt="image" src="https://github.com/user-attachments/assets/91a9cc8e-390a-4b8a-8be7-ee5866b67fbb" />

### Accessing the Web UI

**Docker:**
```bash
docker-compose up -d
# Access at http://localhost:8080
```

**Standalone:**
```bash
hsmm-web --config mods.toml --output mods --port 8080
# Access at http://localhost:8080
```

---

## 🐳 Docker Usage

### docker-compose.yml

```yaml
services:
  hytale-server-modded:
    image: ghcr.io/optikaaal/hsmm:latest
    volumes:
      - ./server-files:/home/hytale/server-files
    ports:
      - "5520:5520/udp"  # Hytale server
      - "8080:8080/tcp"  # Web UI
    environment:
      - PUID=1000
      - PGID=1000
      - SERVER_NAME=My Hytale Server
      - MAX_PLAYERS=20
    restart: unless-stopped
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WEB_UI_PORT` | 8080 | Web UI port |
| `CONFIG_PATH` | /home/hytale/server-files/mods.toml | Config file path |
| `MODS_DIR` | /home/hytale/server-files/mods | Mods directory |
| `PUID` | 1000 | User ID for file permissions |
| `PGID` | 1000 | Group ID for file permissions |

### How It Works

1. Container starts
2. `hsmm upgrade` runs automatically
3. Mods downloaded to `server-files/mods/`
4. Old versions archived to `mods/.old/`
5. Web UI starts on port 8080
6. Hytale server starts with updated mods

---

## 🔍 Finding Mod IDs

### Method 1: CurseForge Website
1. Visit https://www.curseforge.com/hytale/mods
2. Click on a mod
3. Find project ID in "About Project" section

### Method 2: From URL
The URL contains the mod slug:
```
https://www.curseforge.com/hytale/mods/eyespy
                                         ^^^^^^
                                       mod slug
```

---

## ⚙️ Configuration

### Full Example

```toml
# Hytale game ID on CurseForge
game_id = 70216

# Mods to manage
[[mods]]
name = "EyeSpy"
enabled = true
identifier = { curseforge = 1423494 }

[[mods]]
name = "Debug Mod"
enabled = false  # Disabled, won't download
identifier = { curseforge = 1234567 }

# Settings
[settings]
cleanup_old_versions = true
max_old_versions = 3
download_timeout_secs = 300
parallel_downloads = 4
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HYTALE_GAME_ID` | 70216 | CurseForge game ID |
| `CURSEFORGE_API_KEY` | (default) | Custom API key |
| `CONFIG_PATH` | mods.toml | Config file path |
| `MODS_DIR` | mods | Mods directory |

---

## 🏗️ How It Works

### Mod Update Process

1. **Load Config** - Read `mods.toml`
2. **Check CurseForge** - Query API for latest versions
3. **Compare** - Determine what needs updating
4. **Download** - Parallel downloads with progress bars
5. **Cleanup** - Move old versions to `.old/`
6. **Done** - Server ready with latest mods

### Directory Structure

```
server-files/
├── mods.toml          # Configuration
├── mods/              # Active mods
│   ├── EyeSpy-2026.1.20-5708.jar
│   └── .old/          # Archived old versions
│       └── EyeSpy-2026.1.18-5700.jar
├── logs/
│   └── mod-manager.log
└── Server/
    └── HytaleServer.jar
```

---

## 🧪 Testing

```bash
# Quick CLI validation (fast, no network)
./scripts/smoke-test.sh

# Full integration test with real CurseForge downloads
./scripts/test-local.sh

# Rust unit/integration tests
cargo test
```

---

## 🐛 Troubleshooting

### Mod not downloading

**Check project ID:**
```bash
# Visit mod page and verify the project ID
# Use numeric ID, not slug
identifier = { curseforge = 1423494 }  # ✅ Correct
identifier = { curseforge = "eyespy" }  # ❌ Won't work
```

**Check logs:**
```bash
tail -f server-files/logs/mod-manager.log
```

### Permission errors

```bash
# Fix file permissions
chown -R 1000:1000 server-files/
```

### Download timeout

```toml
[settings]
download_timeout_secs = 600  # Increase timeout
```

---

## 📊 Performance

- **Startup Time:** <1ms
- **API Calls:** ~100-300ms each
- **Download Speed:** Limited by network
- **Memory Usage:** <10MB
- **Parallel Downloads:** 4 concurrent (configurable)

**Example:**
- 3 mods to update
- Total time: ~2 seconds
- Network usage: ~500KB

---

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

---

## 🙏 Credits

- Built with [Rust](https://www.rust-lang.org/)
- Uses [furse](https://github.com/EssiumLLC/furse) for CurseForge API
- Inspired by [Ferium](https://github.com/gorilla-devs/ferium)
- Made for the Hytale community

---

## 🔗 Links

- **CurseForge Hytale Mods:** https://www.curseforge.com/hytale
- **Issues:** https://github.com/optikaaal/hytale-server-mod-manager/issues
- **Releases:** https://github.com/optikaaal/hytale-server-mod-manager/releases

---

**Made with ❤️ for Hytale server administrators**
