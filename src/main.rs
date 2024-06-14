use log::{warn, info};
use std::env;
use std::path::PathBuf;

mod fs;
mod http;

use fs::FileManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Initialize the logger

    let base_dir = env::current_dir().unwrap_or_else(|e| {
        warn!("Failed to get current directory: {:?}", e);
        PathBuf::from(".")
    });

    info!("Starting server with base directory: {:?}", base_dir);
    let file_manager = FileManager::new(base_dir);
    http::run_http_server(file_manager).await
}