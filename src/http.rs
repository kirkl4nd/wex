use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder};
use std::fs;
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
                            let mut html_template = fs::read_to_string("src/web/index.html").unwrap_or_default();

                            // Replace placeholders
                            html_template = html_template.replace("{{host}}", &host);

                            let mut breadcrumb_navigation = String::from("<a href=\"/\">.</a> / ");
                            let mut directory_contents = String::new();
                            if path_str != "." {
                                let mut breadcrumb_path = String::new();
                                for (index, component) in path_str.split('/').filter(|&c| !c.is_empty()).enumerate() {
                                    if index > 0 {
                                        breadcrumb_path.push('/');
                                    }
                                    breadcrumb_path.push_str(component);
                                    let link = format!(" <a href=\"/{0}\">{1}</a> / ", breadcrumb_path, component);
                                    breadcrumb_navigation.push_str(&link);
                                }
                            }

                            for entry in entries {
                                let file_name = entry.file_name().unwrap().to_string_lossy();
                                let link_path = format!("{}/{}", path_str, file_name);
                                let link = format!("<li><a href=\"/{0}\">{1}</a></li>", link_path, file_name);
                                directory_contents.push_str(&link);
                            }

                            html_template = html_template.replace("{{breadcrumb_navigation}}", &breadcrumb_navigation);
                            html_template = html_template.replace("{{directory_contents}}", &directory_contents);

                            HttpResponse::Ok().content_type("text/html").body(html_template)
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
