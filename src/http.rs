use crate::file_manager::FileManager;
use crate::html::construct_html; // Import the construct_html function from html.rs
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::dev::ConnectionInfo;
use log::{error, info};
use openssl::ssl::SslAcceptorBuilder;

async fn file_or_directory_handler(
    req: HttpRequest,
    path: Option<web::Path<String>>,
    file_manager: web::Data<FileManager>,
) -> impl Responder {
    let conn_info = req.connection_info();
    let host_with_port = conn_info.host();
    let host = host_with_port.split(':').next().unwrap_or("");

    let path_str = path.map_or_else(|| ".".to_string(), |p| p.into_inner());

    info!("Handling request for host: {} and path: {}", host, path_str);

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
        Err(e) => {
            error!("Error determining file type for path {}: {}", path_str, e);
            error_response("Error determining file type", &e)
        }
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
        Err(e) => {
            error!("Failed to list directory at path {}: {}", path_str, e);
            error_response("Failed to list directory", &e)
        }
    }
}

async fn file_response(file_manager: &web::Data<FileManager>, path_str: &str) -> HttpResponse {
    match file_manager.read_file_contents(path_str) {
        Ok(contents) => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(contents),
        Err(e) => {
            error!("Failed to read file at path {}: {}", path_str, e);
            error_response("Failed to read file", &e)
        }
    }
}

fn error_response(message: &str, error: &std::io::Error) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!("{}: {}", message, error))
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
