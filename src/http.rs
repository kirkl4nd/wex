use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Responder, middleware::Logger};
use openssl::ssl::SslAcceptorBuilder;
use std::path::PathBuf;
use crate::file_manager::FileManager;
use std::fs;

async fn file_or_directory_handler(req: HttpRequest, path: Option<web::Path<String>>, file_manager: web::Data<FileManager>) -> impl Responder {
    let path_str = path.map_or_else(|| ".".to_string(), |p| p.into_inner());
    let host = req.headers().get("host").and_then(|v| v.to_str().ok()).unwrap_or("unknown host");

    match file_manager.path_type(&path_str) {
        Ok(file_type) => {
            if file_type.is_dir() {
                directory_response(&file_manager, &host, &path_str).await
            } else if file_type.is_file() {
                file_response(&file_manager, &path_str).await
            } else {
                HttpResponse::NotFound().body("Resource is neither a file nor a directory")
            }
        },
        Err(e) => error_response("Error determining file type", &e),
    }
}

async fn directory_response(file_manager: &web::Data<FileManager>, host: &str, path_str: &str) -> HttpResponse {
    match file_manager.list_directory(path_str) {
        Ok(entries) => {
            let html_content = construct_html(host, path_str, entries).await;
            HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html_content)
        },
        Err(e) => error_response("Failed to list directory", &e),
    }
}

async fn file_response(file_manager: &web::Data<FileManager>, path_str: &str) -> HttpResponse {
    match file_manager.read_file_contents(path_str) {
        Ok(contents) => HttpResponse::Ok().content_type("application/octet-stream").body(contents),
        Err(e) => error_response("Failed to read file", &e),
    }
}

fn error_response(message: &str, error: &std::io::Error) -> HttpResponse {
    log::error!("{}: {}", message, error);
    HttpResponse::InternalServerError().body(format!("Internal server error: {}", error))
}

async fn construct_html(host: &str, path_str: &str, entries: Vec<PathBuf>) -> String {
    let mut html_template = fs::read_to_string("src/web/index.html").unwrap_or_default();
    html_template = html_template.replace("{{host}}", host);
    let (breadcrumb_navigation, directory_contents) = generate_directory_contents(path_str, entries);
    html_template = html_template.replace("{{breadcrumb_navigation}}", &breadcrumb_navigation);
    html_template = html_template.replace("{{directory_contents}}", &directory_contents);
    html_template
}

fn generate_directory_contents(path_str: &str, entries: Vec<PathBuf>) -> (String, String) {
    let mut breadcrumb_navigation = String::from("<a href=\"/\">Home</a> / ");
    let mut directory_contents = String::new();
    if path_str != "." {
        let mut breadcrumb_path = String::new();
        for (index, component) in path_str.split('/').filter(|&c| !c.is_empty()).enumerate() {
            if index > 0 {
                breadcrumb_path.push('/');
            }
            breadcrumb_path.push_str(component);
            breadcrumb_navigation.push_str(&format!(" <a href=\"/{0}\">{1}</a> / ", breadcrumb_path, component));
        }
    }
    for entry in entries {
        let file_name = entry.file_name().unwrap().to_string_lossy();
        let link_path = format!("{}/{}", path_str, file_name);
        let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
        let icon_class = if is_dir { "folder-icon" } else { "file-icon" };
        directory_contents.push_str(&format!("<li><span class=\"icon {0}\"></span><a href=\"/{1}\">{2}</a></li>", icon_class, link_path, file_name));
    }
    (breadcrumb_navigation, directory_contents)
}

pub async fn run_http_server(file_manager: FileManager, builder: SslAcceptorBuilder) -> std::io::Result<()> {
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
