use std::fs;
use std::path::PathBuf;

pub async fn construct_html(host: &str, path_str: &str, entries: Vec<PathBuf>) -> String {
    let mut html_template = fs::read_to_string("src/web/index.html").unwrap_or_default();
    html_template = html_template.replace("{{host}}", host);
    let (breadcrumb_navigation, directory_contents) =
        generate_directory_contents(path_str, entries);
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
            breadcrumb_navigation.push_str(&format!(
                " <a href=\"/{0}\">{1}</a> / ",
                breadcrumb_path, component
            ));
        }
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
