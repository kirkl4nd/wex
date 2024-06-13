use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use std::path::PathBuf;
use crate::fs::FileManager;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the Wex File Manager!")
}

async fn handle_file_request(req: HttpRequest, file_manager: web::Data<FileManager>) -> impl Responder {
    let path = req.match_info().query("filename");
    let full_path = file_manager.parse_path(path);

    match full_path {
        Some(path) => {
            if let Some(file_type) = file_manager.path_type(&path) {
                if file_type.is_dir() {
                    match file_manager.list_directory(&path) {
                        Ok(entries) => {
                            let mut response = String::from("<ul>");
                            for entry in entries {
                                let file_name = entry.file_name().unwrap().to_string_lossy();
                                let link = format!("<li><a href=\"{}\">{}</a></li>", file_name, file_name);
                                response.push_str(&link);
                            }
                            response.push_str("</ul>");
                            HttpResponse::Ok().body(response)
                        },
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                } else if file_type.is_file() {
                    match file_manager.read_file_contents(&path) {
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
            .route("/", web::get().to(index))
            .route("/{filename:.*}", web::get().to(handle_file_request))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
