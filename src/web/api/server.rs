use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::process::Stdio;

pub async fn restart_server(
    State(state): State<super::AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Step 1: Run mod update
    tracing::info!("Running mod update before restart...");

    let update_result = tokio::process::Command::new("hsmm")
        .arg("--config")
        .arg(&state.config_path)
        .arg("--output")
        .arg(&state.mods_dir)
        .arg("upgrade")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match update_result {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("Mod update completed successfully");
            } else {
                tracing::warn!(
                    "Mod update failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to run mod update: {}", e);
        }
    }

    // Step 2: Check if we're in a container (has /.dockerenv or /run/.containerenv)
    let in_container = tokio::fs::metadata("/.dockerenv").await.is_ok()
        || tokio::fs::metadata("/run/.containerenv").await.is_ok();

    if in_container {
        // In Docker: Kill the main server process, which will cause the container to restart
        tracing::info!("Stopping Hytale server to trigger container restart...");

        let kill_result = tokio::process::Command::new("pkill")
            .arg("-TERM")
            .arg("-f")
            .arg("HytaleServer.jar")
            .output()
            .await;

        match kill_result {
            Ok(output) if output.status.success() => Ok(Json(json!({
                "success": true,
                "message": "Server is restarting... The container will automatically restart with updated mods.",
                "note": "This may take 1-2 minutes. The server will reload with updated mods."
            }))),
            _ => Ok(Json(json!({
                "success": false,
                "message": "Failed to stop server. Server may not be running or insufficient permissions.",
                "note": "Try manually restarting: docker-compose restart hytale-server-modded"
            }))),
        }
    } else {
        // Not in Docker: Just return instructions
        Ok(Json(json!({
            "success": true,
            "message": "Mod update completed. Restart the server to apply changes.",
            "note": "In Docker, use: docker-compose restart hytale-server-modded"
        })))
    }
}

pub async fn get_status() -> Result<Json<Value>, (StatusCode, String)> {
    // Check if HytaleServer.jar process is running
    let output = tokio::process::Command::new("pgrep")
        .arg("-f")
        .arg("HytaleServer.jar")
        .output()
        .await;

    let running = match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };

    Ok(Json(json!({
        "running": running,
        "status": if running { "online" } else { "offline" }
    })))
}

pub async fn stop_server(
    State(state): State<super::AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    tracing::info!("Stopping Hytale server...");

    // Create stop signal file for supervisor
    let stop_signal = state.server_files_dir.join(".hsmm-stop");

    match tokio::fs::write(&stop_signal, "").await {
        Ok(_) => {
            tracing::info!("Stop signal file created at {}", stop_signal.display());
            Ok(Json(json!({
                "success": true,
                "message": "Server is stopping gracefully...",
                "note": "Server will save and shut down. The container will keep running. Use /api/server/start to restart."
            })))
        }
        Err(e) => {
            tracing::error!("Failed to create stop signal: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to stop server: {}", e),
            ))
        }
    }
}

pub async fn start_server(
    State(state): State<super::AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Check if server is already running
    let status_check = tokio::process::Command::new("pgrep")
        .arg("-f")
        .arg("HytaleServer.jar")
        .output()
        .await;

    if let Ok(output) = status_check {
        if output.status.success() {
            return Ok(Json(json!({
                "success": false,
                "message": "Server is already running",
                "note": "Use /api/server/stop to stop it first, or /api/server/restart"
            })));
        }
    }

    tracing::info!("Starting Hytale server...");

    // Create start signal file for supervisor
    let start_signal = state.server_files_dir.join(".hsmm-start");

    match tokio::fs::write(&start_signal, "").await {
        Ok(_) => {
            tracing::info!("Start signal file created at {}", start_signal.display());
            Ok(Json(json!({
                "success": true,
                "message": "Server is starting...",
                "note": "Server will be online in 1-2 minutes. Check status with /api/server/status"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to create start signal: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to start server: {}", e),
            ))
        }
    }
}
