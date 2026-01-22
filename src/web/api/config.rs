use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

use super::AppState;

pub async fn get_config(
    State(state): State<AppState>,
    Path(file): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate file name to prevent directory traversal
    if file.contains("..") || file.contains('/') || file.contains('\\') {
        return Err((StatusCode::BAD_REQUEST, "Invalid file name".to_string()));
    }

    let allowed_files = [
        "config.json",
        "permissions.json",
        "bans.json",
        "whitelist.json",
    ];
    if !allowed_files.contains(&file.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("File '{}' is not allowed", file),
        ));
    }

    let file_path = state.server_files_dir.join(&file);

    if !file_path.exists() {
        // Return empty JSON object for non-existent files
        return Ok(Json(serde_json::json!({})));
    }

    let content = tokio::fs::read_to_string(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e),
        )
    })?;

    let json: Value = serde_json::from_str(&content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse JSON: {}", e),
        )
    })?;

    Ok(Json(json))
}

pub async fn update_config(
    State(state): State<AppState>,
    Path(file): Path<String>,
    Json(content): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate file name
    if file.contains("..") || file.contains('/') || file.contains('\\') {
        return Err((StatusCode::BAD_REQUEST, "Invalid file name".to_string()));
    }

    let allowed_files = [
        "config.json",
        "permissions.json",
        "bans.json",
        "whitelist.json",
    ];
    if !allowed_files.contains(&file.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("File '{}' is not allowed", file),
        ));
    }

    let file_path = state.server_files_dir.join(&file);

    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Write JSON with pretty formatting
    let json_string = serde_json::to_string_pretty(&content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize JSON: {}", e),
        )
    })?;

    tokio::fs::write(&file_path, json_string)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write file: {}", e),
            )
        })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("File '{}' updated successfully", file)
    })))
}
