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

# Determine config and mod paths
CONFIG_PATH="${CONFIG_PATH:-/home/hytale/server-files/mods.toml}"
MODS_DIR="${MODS_DIR:-/home/hytale/server-files/mods}"
LOG_FILE="${LOG_FILE:-/home/hytale/server-files/logs/mod-manager.log}"

# Function to run mod updates
run_mod_update() {
    echo ""
    echo "[Mod Manager] Checking for mod updates..."
    echo "-------------------------------------------"

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
}

# Run initial mod update
run_mod_update

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

# Signal files for server control
STOP_SIGNAL="/home/hytale/server-files/.hsmm-stop"
START_SIGNAL="/home/hytale/server-files/.hsmm-start"

# Clean up old signal files
rm -f "$STOP_SIGNAL" "$START_SIGNAL"

# Function to start the server
# Args: --skip-update to skip mod updates (used on initial start)
start_server() {
    local skip_update=false

    # Check if we should skip update
    if [ "$1" = "--skip-update" ]; then
        skip_update=true
        shift
    fi

    # Run mod updates before starting server (unless skipped)
    if [ "$skip_update" = false ]; then
        run_mod_update
    fi

    echo "[Server] Starting Hytale server..."
    echo ""

    # Check if original init.sh backup exists
    if [ -f "/home/hytale/server/init.sh.original" ]; then
        # Start server in background
        /home/hytale/server/init.sh.original "$@" &
        echo "[Server] Started with PID $!"
    else
        echo "[Error] Original Hytale server init.sh not found!"
        echo "[Error] This should not happen - the base image should provide init.sh.original"
        exit 1
    fi
}

# Start the server initially (skip update since we just ran it)
start_server --skip-update "$@"
SERVER_PID=$!

# Supervisor loop - monitors server and signals
echo "[Supervisor] Monitoring server process (PID: $SERVER_PID)..."
while true; do
    # Check for stop signal
    if [ -f "$STOP_SIGNAL" ]; then
        echo "[Supervisor] Stop signal detected, keeping server stopped..."
        rm -f "$STOP_SIGNAL"

        # Kill server if still running
        if kill -0 $SERVER_PID 2>/dev/null; then
            echo "[Supervisor] Stopping server (PID: $SERVER_PID)..."
            kill -TERM $SERVER_PID 2>/dev/null || true
            wait $SERVER_PID 2>/dev/null || true
        fi

        # Wait for start signal
        echo "[Supervisor] Server stopped. Waiting for start signal..."
        while [ ! -f "$START_SIGNAL" ]; do
            sleep 1
        done

        rm -f "$START_SIGNAL"
        echo "[Supervisor] Start signal detected, restarting server..."
        start_server "$@"
        SERVER_PID=$!
        continue
    fi

    # Check if server is still running
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "[Supervisor] Server process exited, restarting..."
        wait $SERVER_PID 2>/dev/null || true
        sleep 2
        start_server "$@"
        SERVER_PID=$!
    fi

    sleep 2
done
