use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download a file with progress tracking
pub async fn download_file(
    url: &str,
    output_path: &Path,
    file_size: u64,
    timeout_secs: u64,
) -> Result<()> {
    tracing::info!("Downloading: {} -> {}", url, output_path.display());

    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Download with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to download from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    // Create progress bar
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!(
        "Downloading {}",
        output_path.file_name().unwrap().to_string_lossy()
    ));

    // Download to temporary file first
    let temp_path = output_path.with_extension("part");
    let mut file = File::create(&temp_path)
        .await
        .with_context(|| format!("Failed to create temporary file: {}", temp_path.display()))?;

    // Stream the download
    use futures_util::StreamExt;
    use futures_util::TryStreamExt;

    let mut stream = response
        .bytes_stream()
        .map_err(std::io::Error::other);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read download chunk")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write to temporary file")?;
        pb.inc(chunk.len() as u64);
    }

    file.flush().await.context("Failed to flush file")?;
    drop(file);

    pb.finish_with_message(format!(
        "Downloaded {}",
        output_path.file_name().unwrap().to_string_lossy()
    ));

    // Move temporary file to final location
    tokio::fs::rename(&temp_path, output_path)
        .await
        .with_context(|| {
            format!(
                "Failed to move {} to {}",
                temp_path.display(),
                output_path.display()
            )
        })?;

    Ok(())
}

/// Download multiple files in parallel
pub async fn download_files_parallel(
    downloads: Vec<(String, PathBuf, u64)>,
    timeout_secs: u64,
    max_parallel: usize,
) -> Result<Vec<Result<()>>> {
    use futures_util::stream::{self, StreamExt};

    let results = stream::iter(downloads)
        .map(
            |(url, path, size)| async move { download_file(&url, &path, size, timeout_secs).await },
        )
        .buffer_unordered(max_parallel)
        .collect::<Vec<_>>()
        .await;

    Ok(results)
}
