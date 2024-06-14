# wex - a web file explorer

## Overview
`wex` is a web-based file explorer designed to allow users to interact with a filesystem through a web interface. This tool is particularly useful for remote management of files on a server, providing capabilities such as viewing, downloading, and managing files and directories.

## Features
- **File Browsing**: Navigate through the directories on the server.
- **File Download**: Download files directly from the web interface.
- **Directory Viewing**: View the contents of directories, with each item in the directory (file or folder) represented as a clickable link.

## Usage
- ```host:port/```: view current directory
- ```host:port/path```: view directory contents, or download file

## Technical Details
`wex` is built using Rust and leverages the `actix-web` framework for the web server functionality. The application is structured into several modules:

- **`fs` Module**: Handles all filesystem operations such as parsing paths, determining file types, listing directory contents, and reading files. This module ensures that file operations are securely handled.
  
- **`http` Module**: Manages all HTTP server-related functionalities, including routing and responding to HTTP requests. This module uses handlers to serve directory listings and file contents based on the request paths.

## Getting Started
To get started with `wex`, clone the repository and build the project using Cargo, Rust's package manager and build system.

```bash
git clone https://example.com/wex.git
cd wex
cargo build --release
```

Run the application:

```bash
cargo run
```

The server will start, and you can access the web interface by navigating to `http://localhost:8080` in your web browser.

## Contributing
Contributions to `wex` are welcome! If you're interested in improving the functionality or adding new features, please fork the repository and submit a pull request.

## License
`wex` is open-sourced under the MIT License. See the LICENSE file for more details.
