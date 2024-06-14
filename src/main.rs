use log::{info, warn, error};
use std::env;
use std::path::PathBuf;

mod file_manager;
mod html;
mod http;
mod ssl;

use file_manager::FileManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Initialize the logger

    let base_dir = env::current_dir().unwrap_or_else(|e| {
        warn!("Failed to get current directory: {:?}", e);
        PathBuf::from(".")
    });

    // SSL setup
    let builder = match ssl::load_or_create_certificates() {
        Ok(builder) => builder,
        Err(e) => {
            error!("Failed to set up SSL certificates: {:?}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "SSL setup failed"));
        }
    };

    info!("Starting server with base directory: {:?}", base_dir);
    let file_manager = FileManager::new(base_dir);
    match http::run_http_server(file_manager, builder).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Server failed to start: {:?}", e);
            Err(e)
        }
    }
}

