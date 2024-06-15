use crate::file_manager::FileManager;
use crate::html::construct_html; // Import the construct_html function from html.rs
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error, HttpMessage};
use actix_web::dev::ConnectionInfo;
use log::{error, info};
use openssl::ssl::SslAcceptorBuilder;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};

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

async fn upload_file_handler(
    req: HttpRequest,
    mut payload: Multipart,
    file_manager: web::Data<FileManager>,
) -> HttpResponse {
    let path = Some(req.match_info().query("path")).unwrap_or("");
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut file_contents = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_contents.extend_from_slice(&data);
        }
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap();
        let dir_path = path; // Use the path as directory path

        match file_manager.write_file_contents(dir_path, filename, &file_contents) {
            Ok(_) => {
                return HttpResponse::Ok().body(format!("File {} uploaded successfully", filename));
            },
            Err(e) => {
                return HttpResponse::InternalServerError().body(format!("Failed to write file: {}", e));
            }
        }
    }

    HttpResponse::BadRequest().body("No files were uploaded")
}

/// Handler for moving or renaming a file or directory.
async fn move_file_or_directory_handler(
    req: HttpRequest,
    body: web::Payload,
    file_manager: web::Data<FileManager>,
) -> HttpResponse {
    let from_path = req.match_info().query("path");
    let mut body = body;
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = item.unwrap();
        bytes.extend_from_slice(&item);
    }
    let to_path = std::str::from_utf8(&bytes).unwrap_or("");

    match file_manager.move_file_or_directory(from_path, to_path) {
        Ok(_) => HttpResponse::Ok().body("File or directory moved successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to move file or directory: {}", e)),
    }
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
            .route("/{path:.*}", web::post().to(upload_file_handler))
            .route("/{path:.*}", web::put().to(move_file_or_directory_handler)) // Handle PUT requests for moving files or directories
    })
    .bind_openssl("127.0.0.1:8080", builder)?
    .run()
    .await
}
