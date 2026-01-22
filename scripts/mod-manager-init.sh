#!/bin/bash
set -e

echo "==========================================="
echo " Hytale Server with Mod Manager (hsmm)"
echo "==========================================="
echo ""

# Set file permissions
PUID=${PUID:-1000}
PGID=${PGID:-1000}

echo "[Init] Setting file permissions (PUID=$PUID, PGID=$PGID)..."

# Check if running as root
if [ "$(id -u)" -eq 0 ]; then
    # Update hytale user/group IDs if needed
    if [ "$(id -u hytale)" -ne "$PUID" ] || [ "$(id -g hytale)" -ne "$PGID" ]; then
        groupmod -o -g "$PGID" hytale 2>/dev/null || true
        usermod -o -u "$PUID" hytale 2>/dev/null || true
    fi

    # Ensure server-files directory exists and has correct permissions
    if [ -d "/home/hytale/server-files" ]; then
        chown -R hytale:hytale /home/hytale/server-files
    fi
fi

echo ""
echo "[Mod Manager] Checking for mod updates..."
echo "-------------------------------------------"

# Determine config and mod paths
CONFIG_PATH="${CONFIG_PATH:-/home/hytale/server-files/mods.toml}"
MODS_DIR="${MODS_DIR:-/home/hytale/server-files/mods}"
LOG_FILE="${LOG_FILE:-/home/hytale/server-files/logs/mod-manager.log}"

# Create directories if they don't exist
mkdir -p "$(dirname "$CONFIG_PATH")"
mkdir -p "$MODS_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

# Fix permissions if running as root
if [ "$(id -u)" -eq 0 ]; then
    chown -R hytale:hytale "$(dirname "$CONFIG_PATH")" "$MODS_DIR" "$(dirname "$LOG_FILE")" 2>/dev/null || true
fi

# Run mod manager as hytale user
if [ "$(id -u)" -eq 0 ]; then
    # Running as root, switch to hytale user
    su hytale -c "hsmm \
        --config '$CONFIG_PATH' \
        --output '$MODS_DIR' \
        --log '$LOG_FILE' \
        ${VERBOSE:+--verbose} \
        upgrade"
else
    # Already running as hytale user
    hsmm \
        --config "$CONFIG_PATH" \
        --output "$MODS_DIR" \
        --log "$LOG_FILE" \
        ${VERBOSE:+--verbose} \
        upgrade
fi

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "[Mod Manager] ✓ Mod update complete!"
else
    echo ""
    echo "[Mod Manager] ✗ Mod update failed with exit code $EXIT_CODE"
    echo "[Mod Manager] Check log file: $LOG_FILE"
    # Don't fail the container startup, just warn
fi

echo "==========================================="
echo ""

# Start the Web UI in the background
WEB_UI_PORT="${WEB_UI_PORT:-8080}"
STATIC_DIR="${STATIC_DIR:-/home/hytale/web-ui}"

if [ -f "/usr/local/bin/hsmm-web" ] && [ -d "$STATIC_DIR" ]; then
    echo "[Web UI] Starting management interface on port $WEB_UI_PORT..."

    # Start web server in background as hytale user
    if [ "$(id -u)" -eq 0 ]; then
        su hytale -c "hsmm-web \
            --config '$CONFIG_PATH' \
            --output '$MODS_DIR' \
            --server-files '/home/hytale/server-files' \
            --static-dir '$STATIC_DIR' \
            --port $WEB_UI_PORT > /home/hytale/server-files/logs/web-ui.log 2>&1" &
    else
        hsmm-web \
            --config "$CONFIG_PATH" \
            --output "$MODS_DIR" \
            --server-files "/home/hytale/server-files" \
            --static-dir "$STATIC_DIR" \
            --port $WEB_UI_PORT > /home/hytale/server-files/logs/web-ui.log 2>&1 &
    fi

    WEB_PID=$!
    echo "[Web UI] Started with PID $WEB_PID"
    echo "[Web UI] Access at http://localhost:$WEB_UI_PORT"
    echo ""
else
    echo "[Web UI] Skipping (hsmm-web or static files not found)"
    echo ""
fi

# Start the Hytale server using the original entrypoint
echo "[Server] Starting Hytale server..."
echo ""

# Check if original init.sh backup exists
if [ -f "/home/hytale/server/init.sh.original" ]; then
    exec /home/hytale/server/init.sh.original "$@"
else
    echo "[Error] Original Hytale server init.sh not found!"
    echo "[Error] This should not happen - the base image should provide init.sh.original"
    exit 1
fi
