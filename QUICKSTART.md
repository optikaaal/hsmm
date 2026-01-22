# Quick Start Guide - hsmm

Get your Hytale server with automatic mod updates running in 5 minutes!

---

## Step 1: Find Your Mod IDs

Visit [CurseForge Hytale Mods](https://www.curseforge.com/hytale/mods) and browse mods.

For each mod:
1. Click on the mod page
2. Look for the project ID in "About Project" or the URL
3. Write it down

**Example Mods:**
- **EyeSpy** - Project ID: `1423494`
- **Advanced Item Info** - Project ID: `1409811`
- **BetterMap** - Project ID: `1430352`

---

## Step 2: Create Configuration

Create `server-files/mods.toml`:

```toml
game_id = 70216  # Hytale

[[mods]]
name = "EyeSpy"
enabled = true
identifier = { curseforge = 1423494 }

[[mods]]
name = "Advanced Item Info"
enabled = true
identifier = { curseforge = 1409811 }

[settings]
cleanup_old_versions = true
max_old_versions = 3
```

---

## Step 3: Choose Your Method

### Option A: Docker (Recommended)

```bash
# Start server with auto-updating mods
docker-compose up -d

# Watch logs
docker-compose logs -f
```

**That's it!** Mods will auto-update on every server restart.

---

### Option B: Standalone Binary

```bash
# Download and install
wget https://github.com/yourusername/hytale-server-mod-manager/releases/latest/download/hsmm
chmod +x hsmm
sudo mv hsmm /usr/local/bin/

# Update mods
hsmm upgrade --config server-files/mods.toml --output server-files/mods

# Start your Hytale server
./start-server.sh
```

---

## Step 4: Verify

```bash
# Check downloaded mods
ls -lh server-files/mods/

# Should see:
# EyeSpy-2026.1.20-5708.jar
# AdvancedItemInfo-1.0.0.jar
```

---

## Managing Mods

### Add a Mod

```bash
hsmm add "New Mod Name" 1234567 --config server-files/mods.toml
```

### Remove a Mod

```bash
hsmm remove "Mod Name" --config server-files/mods.toml
```

### List Mods

```bash
hsmm list --config server-files/mods.toml
```

### Update Mods

```bash
hsmm upgrade --config server-files/mods.toml --output server-files/mods
```

---

## Troubleshooting

### "Mod not found"
- Verify project ID is correct
- Check CurseForge mod page
- Use numeric ID, not slug

### "Permission denied"
```bash
sudo chown -R 1000:1000 server-files/
```

### "Download timeout"
Edit `mods.toml`:
```toml
[settings]
download_timeout_secs = 600
```

---

## Next Steps

- Add more mods to `mods.toml`
- Set up automated restarts
- Configure server settings
- Check logs: `server-files/logs/mod-manager.log`

---

**Done!** Your server now has automatic mod updates. 🎉

For more details, see [README.md](README.md)
