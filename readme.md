# wex - a web file explorer

## Overview
`wex` is a secure web-based file explorer that enables users to manage files on a server via a web interface. It is ideal for remote file management, offering features like file browsing, downloading, and directory viewing without exposing the full filesystem paths to the user interface.

## Features
- **File Browsing**: Navigate through server directories securely.
- **File Download**: Easily download files through the web interface.
- **Directory Viewing**: View directory contents securely; each item is a clickable link that does not expose the full path.

## Safety and Security
`wex` prioritizes security by ensuring that the full filesystem paths are never exposed outside the `FileManager` struct. This encapsulation helps prevent unauthorized path access and potential security vulnerabilities:

- **Path Handling**: All path operations are securely managed within the `FileManager` struct, ensuring that paths are not directly exposed to the end user or through the web interface.
- **Secure Access**: The system is designed to prevent exposure of sensitive file system details, providing a safe environment for file management.

## Usage
Access the web interface:
- `http://host:port/`: View the current directory.
- `http://host:port/path`: View directory contents or download a specific file.

## Technical Details
`wex` is developed in Rust using the `actix-web` framework for robust web server functionality. The application architecture includes:

- **`fs` Module**: Manages filesystem operations securely, including path parsing, file type determination, directory listing, and file reading.
- **`http` Module**: Handles HTTP server functionalities, such as routing and responding to requests, while ensuring directory listings and file contents are securely served.


## Contributing
We encourage contributions to `wex`. If you wish to contribute, fork the repository, make your changes, and submit a pull request.

## License
`wex` is licensed under the MIT License. For more details, see the LICENSE file.
