use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub lines: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub logs: String,
}

async fn read_log_file(
    log_file: std::path::PathBuf,
    num_lines: usize,
) -> Result<String, (StatusCode, String)> {
    if !log_file.exists() {
        return Ok(String::new());
    }

    let content = tokio::fs::read_to_string(&log_file).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read log file: {}", e),
        )
    })?;

    // Get last N lines
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(num_lines);
    let result = lines[start..].join("\n");

    Ok(result)
}

// HSMM mod manager logs
pub async fn get_hsmm_logs(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogResponse>, (StatusCode, String)> {
    let log_file = state.server_files_dir.join("logs/mod-manager.log");
    let num_lines = query.lines.unwrap_or(100).min(5000);
    let logs = read_log_file(log_file, num_lines).await?;

    Ok(Json(LogResponse { logs }))
}

// Hytale server logs
pub async fn get_server_logs(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogResponse>, (StatusCode, String)> {
    let logs_dir = state.server_files_dir.join("logs");

    // Find the most recent *_server.log file (Hytale server log pattern)
    let log_file = if logs_dir.exists() {
        let mut entries = tokio::fs::read_dir(&logs_dir).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read logs directory: {}", e),
            )
        })?;

        // Filter for *_server.log files and sort by modification time
        let mut server_logs = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.ends_with("_server.log") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        server_logs.push((entry.path(), modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        server_logs.sort_by(|a, b| b.1.cmp(&a.1));

        // Get the most recent log file, or fall back to latest.log
        server_logs
            .first()
            .map(|(path, _)| path.clone())
            .unwrap_or_else(|| logs_dir.join("latest.log"))
    } else {
        logs_dir.join("latest.log")
    };

    let num_lines = query.lines.unwrap_or(100).min(5000);
    let logs = read_log_file(log_file, num_lines).await?;

    Ok(Json(LogResponse { logs }))
}

// Web UI logs
pub async fn get_webui_logs(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogResponse>, (StatusCode, String)> {
    let log_file = state.server_files_dir.join("logs/web-ui.log");
    let num_lines = query.lines.unwrap_or(100).min(5000);
    let logs = read_log_file(log_file, num_lines).await?;

    Ok(Json(LogResponse { logs }))
}
