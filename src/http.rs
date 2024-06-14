use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder};
use std::path::{PathBuf, Path};
use crate::fs::FileManager;

async fn file_or_directory_handler(path: Option<web::Path<String>>, file_manager: web::Data<FileManager>) -> impl Responder {
    // Extract the path or default to the root directory
    let path_str = path.map_or(".".to_string(), |p| p.into_inner());
    
    match file_manager.parse_path(&path_str) {
        Some(full_path) => {
            if let Some(file_type) = file_manager.path_type(&full_path) {
                if file_type.is_dir() {
                    match file_manager.list_directory(&full_path) {
                        Ok(entries) => {
                            let mut response = String::from("<ul>");
                            for entry in entries {
                                let file_name = entry.file_name().unwrap().to_string_lossy();
                                let link_path = format!("{}/{}", path_str, file_name); // Correctly format the path relative to the current directory
                                let link = format!("<li><a href=\"/{0}\">{1}</a></li>", link_path, file_name);
                                response.push_str(&link);
                            }
                            response.push_str("</ul>");
                            HttpResponse::Ok().content_type("text/html").body(response)
                        },
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                } else if file_type.is_file() {
                    match file_manager.read_file_contents(&full_path) {
                        Ok(contents) => HttpResponse::Ok().content_type("application/octet-stream").body(contents),
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                } else {
                    HttpResponse::NotFound().finish()
                }
            } else {
                HttpResponse::NotFound().finish()
            }
        },
        None => HttpResponse::BadRequest().body("Invalid path"),
    }
}

pub async fn run_http_server(file_manager: FileManager) -> std::io::Result<()> {
    let file_manager_data = web::Data::new(file_manager);

    HttpServer::new(move || {
        App::new()
            .app_data(file_manager_data.clone())
            .route("/", web::get().to(file_or_directory_handler))
            .route("/{path:.*}", web::get().to(file_or_directory_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
