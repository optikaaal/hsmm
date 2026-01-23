use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

pub struct WebServer {
    config_path: PathBuf,
    mods_dir: PathBuf,
    server_files_dir: PathBuf,
    static_dir: PathBuf,
}

impl WebServer {
    pub fn new(
        config_path: PathBuf,
        mods_dir: PathBuf,
        server_files_dir: PathBuf,
        static_dir: PathBuf,
    ) -> Self {
        Self {
            config_path,
            mods_dir,
            server_files_dir,
            static_dir,
        }
    }

    pub async fn run(self, port: u16) -> anyhow::Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        tracing::info!("Starting web server on http://{}", addr);
        tracing::info!("Config path: {}", self.config_path.display());
        tracing::info!("Mods directory: {}", self.mods_dir.display());
        tracing::info!("Server files: {}", self.server_files_dir.display());

        let app = self.create_router();

        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Web UI available at http://{}", addr);

        // Use into_make_service_with_connect_info to provide ConnectInfo to middleware
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }

    fn create_router(self) -> Router {
        // API routes
        let api_routes = Router::new()
            .route("/health", get(super::api::health::health_check))
            .route("/mods", get(super::api::mods::list_mods))
            .route("/mods", post(super::api::mods::add_mod))
            .route("/mods/:name", post(super::api::mods::toggle_mod))
            .route(
                "/mods/:name",
                axum::routing::delete(super::api::mods::remove_mod),
            )
            .route(
                "/curseforge/popular",
                get(super::api::curseforge::get_popular_mods),
            )
            .route(
                "/curseforge/search",
                get(super::api::curseforge::search_mods),
            )
            .route(
                "/curseforge/mod/:mod_id",
                get(super::api::curseforge::get_mod_details),
            )
            .route("/config/:file", get(super::api::config::get_config))
            .route("/config/:file", post(super::api::config::update_config))
            .route("/server/restart", post(super::api::server::restart_server))
            .route("/server/status", get(super::api::server::get_status))
            .route("/server/stop", post(super::api::server::stop_server))
            .route("/server/start", post(super::api::server::start_server))
            .route("/logs/hsmm", get(super::api::logs::get_hsmm_logs))
            .route("/logs/server", get(super::api::logs::get_server_logs))
            .route("/logs/webui", get(super::api::logs::get_webui_logs))
            .route("/backups", get(super::api::backups::list_backups))
            .route(
                "/backups/:filename",
                get(super::api::backups::download_backup),
            )
            .route(
                "/backups/:filename",
                axum::routing::delete(super::api::backups::delete_backup),
            )
            .with_state(super::api::AppState {
                config_path: self.config_path,
                mods_dir: self.mods_dir,
                server_files_dir: self.server_files_dir,
            });

        // Serve static files from static_dir (built React app)
        let static_dir_opt = if self.static_dir.exists() {
            Some(self.static_dir.clone())
        } else {
            None
        };

        let app = Router::new()
            .nest("/api", api_routes)
            .layer(middleware::from_fn(super::middleware::error_logger))
            .layer(middleware::from_fn(super::middleware::access_logger))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        // Add static file serving if dist directory exists
        if let Some(static_dir) = static_dir_opt {
            tracing::info!("Serving static files from: {}", static_dir.display());
            app.fallback_service(ServeDir::new(static_dir))
        } else {
            tracing::warn!("Static files directory not found. API-only mode.");
            app
        }
    }
}
