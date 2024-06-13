mod fs;
mod http;

use fs::FileManager;
use std::path::PathBuf;
use std::env;

#[actix_web::main] // This attribute is necessary for running the async main function with actix-web
async fn main() -> std::io::Result<()> {
    // Determine the base directory for the FileManager
    let base_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Create an instance of FileManager
    let file_manager = FileManager::new(base_dir);

    // Start the HTTP server with the FileManager instance
    http::run_http_server(file_manager).await
}