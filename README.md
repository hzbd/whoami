# whoami

A comprehensive, high-performance HTTP request & network information analyzer, built with Rust and Axum.

`whoami` is the ultimate debugging tool designed for containerized environments (like Kubernetes, Docker) and complex network topologies (behind API Gateways, Load Balancers, etc.). It goes far beyond just returning a hostname, providing a deep inspection of every incoming HTTP request and returning a detailed analysis report in JSON format.

[![crates.io](https://img.shields.io/crates/v/whoami.svg)](https://crates.io/crates/whoami)
[![CI](https://github.com/hzbd/whoami/actions/workflows/ci.yml/badge.svg)](https://github.com/hzbd/whoami/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Core Features

*   **Comprehensive Request Inspection**: Captures and returns **all** key information from a request, including its method, URI, HTTP version, IP details, and body.
*   **Intelligent Body Parsing**: Automatically detects the `Content-Type` and parses multiple formats, including:
    *   `application/json`
    *   `application/x-www-form-urlencoded`
    *   `application/xml` (as plain text)
    *   `multipart/form-data` (automatically separates text fields and ignores file content)
    *   `text/plain` and other text-based types
    *   Provides a safe fallback for unknown binary formats.
*   **Proxy-Aware IP Detection**: Accurately identifies and returns the true client IP (`client_ip`) and the direct peer IP (`peer_ip`), even when the service is behind multiple layers of proxies or load balancers.
*   **Zero-Configuration**: A single, self-contained binary with no external dependencies or configuration files needed.
*   **High Performance & Lightweight**: Built on Rust, Tokio, and Axum for a minimal memory footprint and extremely high concurrent request handling capacity.
*   **Container Friendly**: Comes with an optimized multi-stage `Dockerfile` to produce a tiny, secure container image.

## Quick Start

### 1. Build and Run from Source

Ensure you have the Rust toolchain installed.

```bash
# 1. Clone the repository
git clone https://github.com/hzbd/whoami.git
cd whoami
cargo build --release

# 2. Run the service (sudo is required to listen on port 80)
sudo ./target/release/whoami
```

### 2. Using Docker

You can easily build a lightweight Docker image for this project.

```bash
# 1. Build the Docker image
docker build -t whoami-rs .

# 2. Run the container
docker run -d -p 80:80 --name whoami whoami-rs
```

### 3. Test the Service

The service will catch requests on **any** path and with **any** method. Here is an example of a complex `POST` request:

```bash
# Send a POST request with a query parameter, custom header, and a JSON body
curl -X POST "http://127.0.0.1/api/v1/test?param=value" \
  -H "Content-Type: application/json" \
  -H "X-Auth-Header: MyValue123" \
  -d '{"key": "data", "nested": {"array": [1, 2]}}'
```

## API Endpoint

### `ANY /{*path}` (Catch-all)

The service listens for all HTTP methods on all paths. Whether you request `GET /`, `POST /submit`, or `PUT /data/123`, it will return a detailed JSON analysis of that request.

*   **Method**: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`
*   **Path**: `/` (all paths)
*   **Success Response**:
    *   **Code**: `200 OK`
    *   **Content-Type**: `application/json`

### Response Body Structure & Example

Below is a typical response generated from a request made through a proxy server, showcasing all core fields:

```json
{
  "request_id": "a1b2c3d4-e5f6-4a5b-8c9d-0e1f2a3b4c5d",
  "server": "whoami/3.2.0-release",
  "hostname": "backend-container-7f8c9d",
  "at": 1755308000,
  "processing_time_ms": 1.75,
  "ip_details": {
    "client_ip": "203.0.113.75",
    "peer_ip": "10.0.0.1",
    "port": 54321
  },
  "method": "POST",
  "protocol": "HTTP/1.1",
  "uri": "/api/v1/test?param=value",
  "body": {
    "format": "json",
    "content": {
      "key": "data",
      "nested": {
        "array": [1, 2]
      }
    }
  }
}
```
