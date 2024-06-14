use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder};
use std::path::{PathBuf, Path};
use crate::fs::FileManager;

async fn file_or_directory_handler(req: HttpRequest, path: Option<web::Path<String>>, file_manager: web::Data<FileManager>) -> impl Responder {
    let path_str = path.map_or(".".to_string(), |p| p.into_inner());

    // Extract host information from the request headers
    let host = req.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("unknown host");

    match file_manager.parse_path(&path_str) {
        Some(full_path) => {
            if let Some(file_type) = file_manager.path_type(&full_path) {
                if file_type.is_dir() {
                    match file_manager.list_directory(&full_path) {
                        Ok(entries) => {
                            // Start building the response
                            let mut response = String::from("<html><head><title>");
                            response.push_str(host);
                            response.push_str("</title></head><body><h1>");
                            response.push_str(host);
                            response.push_str("</h1><ul>");

                            // Breadcrumb navigation
                            let mut breadcrumb_path = String::new();
                            response.push_str("<div><a href=\"/\">.</a> / ");
                            if path_str != "." {
                                for (index, component) in path_str.split('/').filter(|&c| !c.is_empty()).enumerate() {
                                    if index > 0 {
                                        breadcrumb_path.push('/');
                                    }
                                    breadcrumb_path.push_str(component);
                                    let link = format!(" <a href=\"/{0}\">{1}</a> / ", breadcrumb_path, component);
                                    response.push_str(&link);
                                }
                            }
                            response.push_str("</div>");

                            // Directory contents
                            for entry in entries {
                                let file_name = entry.file_name().unwrap().to_string_lossy();
                                let link_path = format!("{}/{}", path_str, file_name);
                                let link = format!("<li><a href=\"/{0}\">{1}</a></li>", link_path, file_name);
                                response.push_str(&link);
                            }
                            response.push_str("</ul></body></html>");
                            HttpResponse::Ok().content_type("text/html").body(response)
                        },
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                } else if file_type.is_file() {
                    match file_manager.read_file_contents(&full_path) {
                        Ok(contents) => {
                            let file_name = full_path.file_name().unwrap().to_string_lossy().to_string();
                            let content_disposition = format!("attachment; filename=\"{}\"", file_name);
                            HttpResponse::Ok()
                                .content_type("application/octet-stream")
                                .header("Content-Disposition", content_disposition)
                                .body(contents)
                        },
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
