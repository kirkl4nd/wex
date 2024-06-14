use crate::file_manager::FileManager;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use openssl::ssl::SslAcceptorBuilder;
use crate::html::construct_html; // Import the construct_html function from html.rs

async fn file_or_directory_handler(
    req: HttpRequest,
    path: Option<web::Path<String>>,
    file_manager: web::Data<FileManager>,
) -> impl Responder {
    let path_str = path.map_or_else(|| ".".to_string(), |p| p.into_inner());
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown host");

    match file_manager.path_type(&path_str) {
        Ok(file_type) => {
            if file_type.is_dir() {
                directory_response(&file_manager, &host, &path_str).await
            } else if file_type.is_file() {
                file_response(&file_manager, &path_str).await
            } else {
                HttpResponse::NotFound().body("Resource is neither a file nor a directory")
            }
        }
        Err(e) => error_response("Error determining file type", &e),
    }
}

async fn directory_response(
    file_manager: &web::Data<FileManager>,
    host: &str,
    path_str: &str,
) -> HttpResponse {
    match file_manager.list_directory(path_str) {
        Ok(entries) => {
            let html_content = construct_html(host, path_str, entries).await;
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html_content)
        }
        Err(e) => error_response("Failed to list directory", &e),
    }
}

async fn file_response(file_manager: &web::Data<FileManager>, path_str: &str) -> HttpResponse {
    match file_manager.read_file_contents(path_str) {
        Ok(contents) => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(contents),
        Err(e) => error_response("Failed to read file", &e),
    }
}

fn error_response(message: &str, error: &std::io::Error) -> HttpResponse {
    log::error!("{}: {}", message, error);
    HttpResponse::InternalServerError().body(format!("Internal server error: {}", error))
}

pub async fn run_http_server(
    file_manager: FileManager,
    builder: SslAcceptorBuilder,
) -> std::io::Result<()> {
    let file_manager_data = web::Data::new(file_manager);
    HttpServer::new(move || {
        App::new()
            .app_data(file_manager_data.clone())
            .route("/", web::get().to(file_or_directory_handler))
            .route("/{path:.*}", web::get().to(file_or_directory_handler))
    })
    .bind_openssl("127.0.0.1:8080", builder)?
    .run()
    .await
}
