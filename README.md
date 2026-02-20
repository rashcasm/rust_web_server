# Rust Web Server

This is a simple web server implemented in Rust. It serves static HTML files and demonstrates basic web server functionality using the Rust programming language.

### Features

- Serves static HTML files
- Custom 404 error page
- Built with Rust for high performance and safety

## Prerequisites

- [Rust](https://www.rust-lang.org/) (Ensure you have the latest stable version installed)

## Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/rust_web_server_rashmin.git
   cd rust_web_server_rashmin
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the server:
   ```bash
   cargo run
   ```

4. Open your browser and navigate to `http://localhost:7878` to view the served HTML files.

## Configuration

- The server serves the `hello.html` file by default.
- If a requested file is not found, the server will return the `404.html` page.
