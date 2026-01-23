use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use tokio_util::io::ReaderStream;

use super::AppState;

#[derive(Debug, Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub is_archived: bool,
}

/// List all backups
pub async fn list_backups(
    State(state): State<AppState>,
) -> Result<Json<Vec<BackupInfo>>, (StatusCode, String)> {
    let backups_dir = state.server_files_dir.join("backups");

    if !backups_dir.exists() {
        return Ok(Json(Vec::new()));
    }

    let mut backups = Vec::new();

    // Read main backups directory
    if let Ok(mut entries) = tokio::fs::read_dir(&backups_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".zip") {
                            let created_at = metadata
                                .modified()
                                .ok()
                                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|duration| {
                                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_else(|| "Unknown".to_string())
                                })
                                .unwrap_or_else(|| "Unknown".to_string());

                            backups.push(BackupInfo {
                                filename: filename.to_string(),
                                size_bytes: metadata.len(),
                                created_at,
                                is_archived: false,
                            });
                        }
                    }
                }
            }
        }
    }

    // Read archived backups
    let archive_dir = backups_dir.join("archive");
    if archive_dir.exists() {
        if let Ok(mut entries) = tokio::fs::read_dir(&archive_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".zip") {
                                let created_at = metadata
                                    .modified()
                                    .ok()
                                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|duration| {
                                        chrono::DateTime::from_timestamp(
                                            duration.as_secs() as i64,
                                            0,
                                        )
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_else(|| "Unknown".to_string())
                                    })
                                    .unwrap_or_else(|| "Unknown".to_string());

                                backups.push(BackupInfo {
                                    filename: format!("archive/{}", filename),
                                    size_bytes: metadata.len(),
                                    created_at,
                                    is_archived: true,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by filename (newest first, since filenames are timestamps)
    backups.sort_by(|a, b| b.filename.cmp(&a.filename));

    Ok(Json(backups))
}

/// Download a backup file
pub async fn download_backup(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Prevent path traversal
    if filename.contains("..") || filename.starts_with('/') {
        return Err((StatusCode::BAD_REQUEST, "Invalid filename".to_string()));
    }

    let backups_dir = state.server_files_dir.join("backups");
    let file_path = backups_dir.join(&filename);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Backup not found".to_string()));
    }

    // Verify the file is within the backups directory (security check)
    if !file_path.starts_with(&backups_dir) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let file = tokio::fs::File::open(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open file: {}", e),
        )
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Get just the filename for the download header
    let download_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("backup.zip");

    let content_disposition = format!("attachment; filename=\"{}\"", download_filename);

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (header::CONTENT_DISPOSITION, content_disposition),
    ];

    Ok((headers, body))
}

/// Delete a backup file
pub async fn delete_backup(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Prevent path traversal
    if filename.contains("..") || filename.starts_with('/') {
        return Err((StatusCode::BAD_REQUEST, "Invalid filename".to_string()));
    }

    let backups_dir = state.server_files_dir.join("backups");
    let file_path = backups_dir.join(&filename);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Backup not found".to_string()));
    }

    // Verify the file is within the backups directory (security check)
    if !file_path.starts_with(&backups_dir) {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    tokio::fs::remove_file(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete backup: {}", e),
        )
    })?;

    tracing::info!("Deleted backup: {}", filename);

    Ok(Json(json!({
        "success": true,
        "message": format!("Backup '{}' deleted successfully", filename)
    })))
}
