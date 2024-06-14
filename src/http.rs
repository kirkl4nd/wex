use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder, middleware::Logger};
use std::path::PathBuf;
use crate::fs::FileManager;
use std::fs;

async fn file_or_directory_handler(req: HttpRequest, path: Option<web::Path<String>>, file_manager: web::Data<FileManager>) -> impl Responder {
    let path_str = path.map_or_else(|| ".".to_string(), |p| p.into_inner());

    // Extract host information from the request headers
    let host = req.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("unknown host");

    match file_manager.path_type(&path_str) {
        Ok(file_type) => {
            if file_type.is_dir() {
                match file_manager.list_directory(&path_str) {
                    Ok(entries) => {
                        let html_content = construct_html(&host, &path_str, entries).await;
                        HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html_content)
                    },
                    Err(e) => {
                        log::error!("Failed to list directory: {}", e);
                        HttpResponse::InternalServerError().body(format!("Internal server error: {}", e))
                    },
                }
            } else if file_type.is_file() {
                match file_manager.read_file_contents(&path_str) {
                    Ok(contents) => HttpResponse::Ok().content_type("application/octet-stream").body(contents),
                    Err(e) => {
                        log::error!("Failed to read file: {}", e);
                        HttpResponse::InternalServerError().body(format!("Internal server error: {}", e))
                    },
                }
            } else {
                HttpResponse::NotFound().body("Resource is neither a file nor a directory")
            }
        },
        Err(e) => {
            log::error!("Error determining file type: {}", e);
            HttpResponse::InternalServerError().body(format!("Internal server error: {}", e))
        },
    }
}

async fn construct_html(host: &str, path_str: &str, entries: Vec<PathBuf>) -> String {
    let mut html_template = fs::read_to_string("src/web/index.html").unwrap_or_default();

    // Replace placeholders
    html_template = html_template.replace("{{host}}", host);

    let mut breadcrumb_navigation = String::from("<a href=\"/\">Home</a> / ");
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
        let parent_path = std::path::Path::new(&path_str).parent().map_or(".", |p| p.to_str().unwrap_or("."));
        directory_contents.push_str(&format!("<li class=\"up-directory\"><span class=\"icon folder-icon\"></span><a href=\"/{0}\">../</a></li>", parent_path));
    }

    for entry in entries {
        let file_name = entry.file_name().unwrap().to_string_lossy();
        let link_path = format!("{}/{}", path_str, file_name);
        let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
        let icon_class = if is_dir { "folder-icon" } else { "file-icon" };
        let link = format!("<li><span class=\"icon {0}\"></span><a href=\"/{1}\">{2}</a></li>", icon_class, link_path, file_name);
        directory_contents.push_str(&link);
    }

    html_template = html_template.replace("{{breadcrumb_navigation}}", &breadcrumb_navigation);
    html_template = html_template.replace("{{directory_contents}}", &directory_contents);

    html_template
}

pub async fn run_http_server(file_manager: FileManager) -> std::io::Result<()> {
    let file_manager_data = web::Data::new(file_manager);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(file_manager_data.clone())
            .route("/", web::get().to(file_or_directory_handler))
            .route("/{path:.*}", web::get().to(file_or_directory_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
