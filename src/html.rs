use std::fs;
use std::path::PathBuf;

/// Generates the complete HTML for a directory listing.
pub async fn generate_directory_listing_html(host: &str, path_str: &str, entries: Vec<PathBuf>) -> String {
    let static_html = fs::read_to_string("src/web/index.html").unwrap_or_default();
    let (breadcrumb_navigation, directory_contents) = generate_directory_contents(path_str, entries);
    let html = static_html
        .replace("{{host}}", host)
        .replace("{{breadcrumb_navigation}}", &breadcrumb_navigation)
        .replace("{{directory_contents}}", &directory_contents);
    html
}

fn generate_directory_contents(path_str: &str, entries: Vec<PathBuf>) -> (String, String) {
    let mut breadcrumb_navigation = String::from("<a href=\"/\">Home</a> / ");
    let mut directory_contents = String::new();

    // Add the upload button at the top of the list
    directory_contents.push_str(
        "<li class=\"upload-item\"><label for=\"file-input\" id=\"upload-label\"><span class=\"icon\">➕</span>Upload files</label><input type=\"file\" id=\"file-input\" name=\"files\" multiple onchange=\"uploadFiles()\"></li>"
    );

    // Check if the current path is not the root directory
    if path_str != "." {
        let mut breadcrumb_path = String::new();
        let path_components: Vec<&str> = path_str.split('/').filter(|&c| !c.is_empty()).collect();
        for (index, component) in path_components.iter().enumerate() {
            if index > 0 {
                breadcrumb_path.push('/');
            }
            breadcrumb_path.push_str(component);
            breadcrumb_navigation.push_str(&format!(
                " <a href=\"/{0}\">{1}</a> / ",
                breadcrumb_path, component
            ));
        }

        // Add the "../" link at the top of the directory contents with an up-arrow icon
        let parent_link = format!("/{}/..", path_str.trim_end_matches('/'));
        directory_contents.push_str(&format!(
            "<li class=\"up-directory\"><span class=\"icon up-icon\"></span><a href=\"{}\">../</a></li>",
            parent_link
        ));
    }

    for entry in entries {
        let file_name = entry.file_name().unwrap().to_string_lossy();
        let link_path = format!("{}/{}", path_str, file_name);
        let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
        let icon_class = if is_dir { "folder-icon" } else { "file-icon" };
        directory_contents.push_str(&format!(
            "<li><span class=\"icon {0}\"></span><a href=\"/{1}\">{2}</a></li>",
            icon_class, link_path, file_name
        ));
    }
    (breadcrumb_navigation, directory_contents)
}
