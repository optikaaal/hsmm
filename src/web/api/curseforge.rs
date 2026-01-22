use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CurseForgeMod {
    pub id: i32,
    pub name: String,
    pub summary: String,
    pub download_count: i32,
    pub logo_url: Option<String>,
    pub author: String,
    pub date_modified: String,
}

#[derive(Debug, Serialize)]
pub struct CurseForgeModDetails {
    pub id: i32,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub download_count: i32,
    pub logo_url: Option<String>,
    pub author: String,
    pub authors: Vec<String>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub categories: Vec<String>,
    pub screenshots: Vec<ScreenshotInfo>,
    pub latest_files: Vec<FileInfo>,
    pub links: ModLinks,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotInfo {
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub id: i32,
    pub display_name: String,
    pub file_name: String,
    pub file_date: String,
    pub file_length: i64,
    pub download_count: i32,
}

#[derive(Debug, Serialize)]
pub struct ModLinks {
    pub website_url: Option<String>,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

pub async fn get_popular_mods(
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<CurseForgeMod>>, (StatusCode, String)> {
    let game_id = crate::curseforge::get_hytale_game_id().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get game ID: {}", e),
        )
    })?;

    // Build CurseForge API URL for searching mods
    let page_index = query.page.unwrap_or(0) * 20;
    let url = format!(
        "https://api.curseforge.com/v1/mods/search?gameId={}&classId=9137&sortField=6&sortOrder=desc&index={}&pageSize=20",
        game_id, page_index
    );

    // Get API key
    const DEFAULT_KEY: &str = "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.";
    let api_key = std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| DEFAULT_KEY.to_string());

    // Make API request
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("CurseForge API request failed: {}", e),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::from_u16(response.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!("CurseForge API returned status: {}", response.status()),
        ));
    }

    let json: Value = response.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse response: {}", e),
        )
    })?;

    // Parse mods from response
    let mods_array = json["data"].as_array().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Invalid API response format".to_string(),
    ))?;

    let mods: Vec<CurseForgeMod> = mods_array
        .iter()
        .filter_map(|m| {
            Some(CurseForgeMod {
                id: m["id"].as_i64()? as i32,
                name: m["name"].as_str()?.to_string(),
                summary: m["summary"].as_str().unwrap_or("").to_string(),
                download_count: m["downloadCount"].as_i64().unwrap_or(0) as i32,
                logo_url: m["logo"]["url"].as_str().map(|s| s.to_string()),
                author: m["authors"][0]["name"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string(),
                date_modified: m["dateModified"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect();

    Ok(Json(mods))
}

pub async fn search_mods(
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<CurseForgeMod>>, (StatusCode, String)> {
    let game_id = crate::curseforge::get_hytale_game_id().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get game ID: {}", e),
        )
    })?;

    let search_term = query.q.unwrap_or_default();
    let page_index = query.page.unwrap_or(0) * 20;

    // Build CurseForge API URL with search filter
    let mut url = format!(
        "https://api.curseforge.com/v1/mods/search?gameId={}&classId=9137&sortField=6&sortOrder=desc&index={}&pageSize=20",
        game_id, page_index
    );

    if !search_term.is_empty() {
        url.push_str(&format!(
            "&searchFilter={}",
            urlencoding::encode(&search_term)
        ));
    }

    // Get API key
    const DEFAULT_KEY: &str = "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.";
    let api_key = std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| DEFAULT_KEY.to_string());

    // Make API request
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("CurseForge API request failed: {}", e),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::from_u16(response.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!("CurseForge API returned status: {}", response.status()),
        ));
    }

    let json: Value = response.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse response: {}", e),
        )
    })?;

    // Parse mods from response
    let mods_array = json["data"].as_array().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Invalid API response format".to_string(),
    ))?;

    let mods: Vec<CurseForgeMod> = mods_array
        .iter()
        .filter_map(|m| {
            Some(CurseForgeMod {
                id: m["id"].as_i64()? as i32,
                name: m["name"].as_str()?.to_string(),
                summary: m["summary"].as_str().unwrap_or("").to_string(),
                download_count: m["downloadCount"].as_i64().unwrap_or(0) as i32,
                logo_url: m["logo"]["url"].as_str().map(|s| s.to_string()),
                author: m["authors"][0]["name"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string(),
                date_modified: m["dateModified"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect();

    Ok(Json(mods))
}

pub async fn get_mod_details(
    Path(mod_id): Path<i32>,
) -> Result<Json<CurseForgeModDetails>, (StatusCode, String)> {
    let game_id = crate::curseforge::get_hytale_game_id().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get game ID: {}", e),
        )
    })?;

    // Build CurseForge API URL for specific mod
    let url = format!("https://api.curseforge.com/v1/mods/{}", mod_id);

    // Get API key
    const DEFAULT_KEY: &str = "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.";
    let api_key = std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| DEFAULT_KEY.to_string());

    // Make API request
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("CurseForge API request failed: {}", e),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::from_u16(response.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!("CurseForge API returned status: {}", response.status()),
        ));
    }

    let json: Value = response.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse response: {}", e),
        )
    })?;

    // Parse mod details from response
    let data = &json["data"];

    // Verify game ID
    if let Some(mod_game_id) = data["gameId"].as_i64() {
        if mod_game_id as i32 != game_id {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Mod is not for Hytale (game ID: {})", game_id),
            ));
        }
    }

    // Extract authors
    let authors: Vec<String> = data["authors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Extract categories
    let categories: Vec<String> = data["categories"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Extract screenshots
    let screenshots: Vec<ScreenshotInfo> = data["screenshots"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(ScreenshotInfo {
                        title: s["title"].as_str().unwrap_or("").to_string(),
                        description: s["description"].as_str().unwrap_or("").to_string(),
                        thumbnail_url: s["thumbnailUrl"].as_str()?.to_string(),
                        url: s["url"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract latest files (limit to 5 most recent)
    let latest_files: Vec<FileInfo> = data["latestFiles"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .take(5)
                .filter_map(|f| {
                    Some(FileInfo {
                        id: f["id"].as_i64()? as i32,
                        display_name: f["displayName"].as_str()?.to_string(),
                        file_name: f["fileName"].as_str()?.to_string(),
                        file_date: f["fileDate"].as_str().unwrap_or("").to_string(),
                        file_length: f["fileLength"].as_i64().unwrap_or(0),
                        download_count: f["downloadCount"].as_i64().unwrap_or(0) as i32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract links
    let links = ModLinks {
        website_url: data["links"]["websiteUrl"].as_str().map(|s| s.to_string()),
        wiki_url: data["links"]["wikiUrl"].as_str().map(|s| s.to_string()),
        issues_url: data["links"]["issuesUrl"].as_str().map(|s| s.to_string()),
        source_url: data["links"]["sourceUrl"].as_str().map(|s| s.to_string()),
    };

    let details = CurseForgeModDetails {
        id: data["id"].as_i64().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid mod ID".to_string(),
        ))? as i32,
        name: data["name"]
            .as_str()
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing mod name".to_string(),
            ))?
            .to_string(),
        summary: data["summary"].as_str().unwrap_or("").to_string(),
        description: data["description"].as_str().unwrap_or("").to_string(),
        download_count: data["downloadCount"].as_i64().unwrap_or(0) as i32,
        logo_url: data["logo"]["url"].as_str().map(|s| s.to_string()),
        author: authors
            .first()
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string()),
        authors,
        date_created: data["dateCreated"].as_str().unwrap_or("").to_string(),
        date_modified: data["dateModified"].as_str().unwrap_or("").to_string(),
        date_released: data["dateReleased"].as_str().unwrap_or("").to_string(),
        categories,
        screenshots,
        latest_files,
        links,
    };

    Ok(Json(details))
}
